//! Internal flash controller for MAX3263 family.

use crate::{
    hal::flash::ReadWrite,
    utilities::memory::{IterableByOverlaps, Region},
};
use core::convert::TryInto;
use core::ops::{Add, Sub};

const PAGE_SIZE: u32 = KB!(8);
const PAGE_COUNT: u32 = 256;

pub struct Flash {
    flc: max3263x::FLC,
}

#[derive(Copy, Clone, Debug)]
struct Map;

#[derive(Copy, Clone, Debug)]
struct Page(pub u32);

#[derive(Copy, Clone, Debug)]
pub enum Error {
    AddressOutOfRange,
    PageEraseFailed,
    MassEraseFailed,
    WriteFailed,
    UnalignedAccess,
}

#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Address(pub u32);

impl Map {
    pub fn pages() -> impl Iterator<Item = Page> {
        (0..PAGE_COUNT).map(Page)
    }
}

impl Page {
    pub fn address(&self) -> Address {
        Address(PAGE_SIZE * self.0)
    }
}

impl Region<Address> for Map {
    fn contains(&self, address: Address) -> bool {
        address < Address(PAGE_SIZE * PAGE_COUNT)
    }
}

impl Region<Address> for Page {
    fn contains(&self, address: Address) -> bool {
        let start = Address(PAGE_SIZE * self.0);
        (address >= start) && (address < start + PAGE_SIZE as usize)
    }
}

impl Add<usize> for Address {
    type Output = Self;
    fn add(self, rhs: usize) -> Address {
        Address(self.0 + rhs as u32)
    }
}

impl Sub<usize> for Address {
    type Output = Self;
    fn sub(self, rhs: usize) -> Address {
        Address(self.0.saturating_sub(rhs as u32))
    }
}

impl Sub<Address> for Address {
    type Output = usize;
    fn sub(self, rhs: Address) -> usize {
        self.0.saturating_sub(rhs.0) as usize
    }
}

impl Into<usize> for Address {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Flash {
    pub fn new(flc: max3263x::FLC) -> Self {
        flc.perform.write(|w| {
            w.en_back2back_rds()
                .set_bit()
                .en_merge_grab_gnt()
                .set_bit()
                .auto_tacc()
                .set_bit()
                .auto_clkdiv()
                .set_bit()
                .en_prevent_fail()
                .set_bit()
        });

        Self { flc }
    }

    // Returns whether the flash controller is performing a read, write, or erase operation.
    fn is_busy(&self) -> bool {
        self.flc.ctrl.read().write().bit_is_set()
            || self.flc.ctrl.read().mass_erase().bit_is_set()
            || self.flc.ctrl.read().page_erase().bit_is_set()
    }

    fn wait_until_not_busy(&self) {
        while self.is_busy() {}
    }

    /// Disable write and erase operations.
    fn lock_flash(&mut self) {
        self.flc
            .ctrl
            .modify(|_, w| unsafe { w.flsh_unlock().bits(0b0000).erase_code().bits(0x00) });
    }

    /// Enable write and erase operations.
    fn unlock_flash(&mut self) {
        self.flc
            .ctrl
            .modify(|_, w| unsafe { w.flsh_unlock().bits(0b0010) });
    }

    fn erase_page(&mut self, page: Page) -> nb::Result<(), Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        self.clear_errors();
        self.unlock_flash();

        self.flc
            .ctrl
            .modify(|_, w| unsafe { w.erase_code().bits(0x55) });

        self.flc
            .faddr
            .write(|w| unsafe { w.faddr().bits(page.address().0) });

        self.flc.ctrl.modify(|_, w| w.page_erase().bit(true));

        self.wait_until_not_busy();
        self.lock_flash();

        if self.read_failed_bit() {
            Err(nb::Error::Other(Error::PageEraseFailed))
        } else {
            Ok(())
        }
    }

    /// Return the state of the failed bit, clearing it if set.
    fn read_failed_bit(&mut self) -> bool {
        let failed = self.flc.intr.read().failed_if().bit_is_set();
        self.flc.intr.modify(|_, w| w.failed_if().bit(false));
        failed
    }

    fn clear_errors(&mut self) {
        self.flc.intr.modify(|_, w| w.failed_if().bit(false));
    }

    fn write_range(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        let is_start_aligned = address.0 % 4 == 0;
        let is_end_aligned = bytes.len() % 4 == 0;
        if !is_start_aligned || !is_end_aligned {
            return Err(nb::Error::Other(Error::UnalignedAccess));
        }

        self.clear_errors();
        self.unlock_flash();

        let addresses = (address.0..).step_by(4);
        let words = bytes
            .chunks_exact(4)
            .map(|b| u32::from_ne_bytes(b.try_into().unwrap()));

        for (address, word) in addresses.zip(words) {
            self.flc.faddr.write(|w| unsafe { w.bits(address) });
            self.flc.fdata.write(|w| unsafe { w.bits(word) });
            self.flc.ctrl.modify(|_, w| w.write().set_bit());
            self.wait_until_not_busy();
        }

        self.lock_flash();

        if self.read_failed_bit() {
            Err(nb::Error::Other(Error::WriteFailed))
        } else {
            Ok(())
        }
    }
}

impl ReadWrite for Flash {
    type Error = Error;
    type Address = Address;

    fn label() -> &'static str {
        "MAX3263 flash (internal)"
    }

    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        let (minimum_address, maximum_address) = self.range();
        let is_valid_address =
            (address >= minimum_address) && (address + bytes.len() <= maximum_address);
        if !is_valid_address {
            return Err(nb::Error::Other(Error::AddressOutOfRange));
        }

        let source = address.0 as *const u8;
        let destination = bytes.as_mut_ptr();
        let size = bytes.len();

        unsafe {
            core::intrinsics::copy_nonoverlapping(source, destination, size);
        }

        Ok(())
    }

    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        let is_start_aligned = address.0 & 0b11 == 0;
        let is_end_aligned = bytes.len() & 0b11 == 0;
        if !is_start_aligned || !is_end_aligned {
            return Err(nb::Error::Other(Error::UnalignedAccess));
        }

        let (minimum_address, maximum_address) = self.range();
        let is_valid_address =
            (address >= minimum_address) && (address + bytes.len() <= maximum_address);
        if !is_valid_address {
            return Err(nb::Error::Other(Error::AddressOutOfRange));
        }

        for (block, page, address) in Map::pages().overlaps(bytes, address) {
            let page_data = &mut [0u8; PAGE_SIZE as usize];
            nb::block!(self.read(page.address(), page_data))?;

            let offset_into_page = address.0.saturating_sub(page.address().0) as usize;
            page_data
                .iter_mut()
                .skip(offset_into_page)
                .zip(block)
                .for_each(|(byte, input)| *byte = *input);
            nb::block!(self.erase_page(page))?;
            nb::block!(self.write_range(page.address(), page_data))?;
        }

        Ok(())
    }

    fn range(&self) -> (Self::Address, Self::Address) {
        (Address(0x0000_0000), Address(PAGE_SIZE * PAGE_COUNT))
    }

    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        self.clear_errors();
        self.unlock_flash();

        const MASS_ERASE_CODE: u8 = 0xAA;
        self.flc
            .ctrl
            .modify(|_, w| unsafe { w.erase_code().bits(MASS_ERASE_CODE) });

        self.flc.ctrl.modify(|_, w| w.mass_erase().set_bit());

        self.wait_until_not_busy();

        self.lock_flash();

        let failed = self.read_failed_bit();

        if failed {
            Err(nb::Error::Other(Error::MassEraseFailed))
        } else {
            Ok(())
        }
    }

    fn write_from_blocks<I: Iterator<Item = [u8; N]>, const N: usize>(
        &mut self,
        address: Self::Address,
        blocks: I,
    ) -> Result<(), Self::Error> {
        const TRANSFER_SIZE: usize = KB!(4);
        assert!(TRANSFER_SIZE % N == 0);
        let mut transfer_array = [0x00u8; TRANSFER_SIZE];
        let mut memory_index = 0usize;

        for block in blocks {
            let slice = &mut transfer_array
                [(memory_index % TRANSFER_SIZE)..((memory_index % TRANSFER_SIZE) + N)];
            slice.clone_from_slice(&block);
            memory_index += N;

            if memory_index % TRANSFER_SIZE == 0 {
                nb::block!(self.write(address + (memory_index - TRANSFER_SIZE), &transfer_array))?;
                transfer_array.iter_mut().for_each(|b| *b = 0x00u8);
            }
        }
        let remainder = &transfer_array[0..(memory_index % TRANSFER_SIZE)];
        nb::block!(self.write(address + (memory_index - remainder.len()), &remainder))?;
        Ok(())
    }
}
