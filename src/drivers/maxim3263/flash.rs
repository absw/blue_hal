//! Internal flash controller for the MAX32630.

use core::ops::{Add, Sub};
use core::convert::TryInto;
use crate::{
    hal::flash::ReadWrite,
    utilities::memory::{IterableByOverlaps, Region},
};

const PAGE_SIZE : u32 = 0x2000;
const PAGE_COUNT : u32 = 256;

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
    pub fn pages() -> impl Iterator<Item = Page> { (0..PAGE_COUNT).map(Page) }
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
    fn add(self, rhs: usize) -> Address { Address(self.0 + rhs as u32) }
}

impl Sub<usize> for Address {
    type Output = Self;
    fn sub(self, rhs: usize) -> Address { Address(self.0.saturating_sub(rhs as u32)) }
}

impl Sub<Address> for Address {
    type Output = usize;
    fn sub(self, rhs: Address) -> usize { self.0.saturating_sub(rhs.0) as usize }
}

impl Into<usize> for Address {
    fn into(self) -> usize { self.0 as usize }
}

extern "C" {
    fn FLC_Init() -> i32;
    fn FLC_PageErase(address: u32, erase_code: u8, unlock_key: u8) -> i32;
    fn FLC_Write(address: u32, data: *const u8, length: u32, unlock_key: u8) -> i32;
}

impl Flash {
    pub fn new(flc: max3263x::FLC) -> Self {
        // let error = unsafe { FLC_Init() };
        // assert!(error == 0);

        // Self { flc }

        assert!(
            !flc.ctrl.read().write().bit_is_set()
            && !flc.ctrl.read().mass_erase().bit_is_set()
            && !flc.ctrl.read().page_erase().bit_is_set()
        );

        flc.perform.write(|w| {
            w.auto_clkdiv().set_bit()
            // .en_back2back_rds().set_bit()
            // .en_merge_grab_gnt().set_bit()
            // .auto_tacc().set_bit()
        });

        flc.perform.write(|w| {
            w.en_prevent_fail().set_bit()
        });

        // flc.fckdiv.write(|w| unsafe {
        //     w.fckdiv().bits(96)
        // });

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
        self.flc.ctrl.write(|w| unsafe {
            w
                .flsh_unlock().bits(0b0000)
                .erase_code().bits(0x00)
        });
        assert!(self.flc.ctrl.read().flsh_unlock().bits() == 0);
    }

    /// Enable write and erase operations.
    fn unlock_flash(&mut self) {
        self.flc.ctrl.write(|w| unsafe {
            w.flsh_unlock().bits(0b0010)
        });
        assert!(self.flc.ctrl.read().flsh_unlock().bits() == 2);
    }

    fn erase_page(&mut self, page: Page) -> nb::Result<(), Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        self.clear_errors();
        self.unlock_flash();

        self.flc.ctrl.modify(|_, w| unsafe {
            w.erase_code().bits(0x55)
        });

        self.flc.faddr.write(|w| unsafe {
            w.faddr().bits(page.address().0)
        });

        self.flc.ctrl.modify(|_, w| unsafe {
            w.page_erase().bit(true)
        });

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
        self.flc.intr.write(|w| w.failed_if().bit(false));
        failed
    }

    fn clear_errors(&mut self) {
        self.flc.intr.write(|w| w.failed_if().bit(false));
    }

    fn write_range(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Error> {
        let error = unsafe { FLC_Write(address.0, bytes.as_ptr(), bytes.len() as u32, 0b10) };
        assert!(error == 0);
        Ok(())
        // if self.is_busy() {
        //     return Err(nb::Error::WouldBlock);
        // }

        // let is_start_aligned = address.0 % 4 == 0;
        // let is_end_aligned = bytes.len() % 4 == 0;
        // if !is_start_aligned || !is_end_aligned {
        //     return Err(nb::Error::Other(Error::UnalignedAccess));
        // }

        // self.clear_errors();

        // self.unlock_flash();

        // assert!(self.flc.ctrl.read().write_enable().bit_is_set());

        // // self.flc.ctrl.write(|w| w.auto_incre_mode().set_bit());
        // // MXC_FLC->ctrl |= MXC_F_FLC_CTRL_AUTO_INCRE_MODE;

        // // MXC_FLC->faddr = address;
        // // self.flc.faddr.write(|w| unsafe {
        // //     w.faddr().bits(address.0)
        // // });

        // // /* Set the address to write and enable auto increment */
        // // uint32_t write_cmd = MXC_FLC->ctrl | MXC_F_FLC_CTRL_WRITE;

        // // for (; length > 0; length -= 4) {
        // //     /* Perform the write */
        // //     MXC_FLC->fdata = *ptr++;
        // //     MXC_FLC->ctrl = write_cmd;
        // //     while (FLC_Busy());
        // // }

        // let addresses = (address.0..).step_by(4);
        // let words = bytes.chunks_exact(4).map(|b| u32::from_ne_bytes(b.try_into().unwrap()));

        // for (address, word) in addresses.zip(words) {
        //     let old_value = unsafe { *(address as *const u32) };

        //     self.flc.faddr.write(|w| unsafe { w.bits(address) });
        //     self.flc.fdata.write(|w| unsafe { w.bits(word) });
        //     self.flc.ctrl.write(|w| w.write().set_bit());
        //     self.wait_until_not_busy();
        //     assert!(!self.read_failed_bit(), "Failed to write 0x{:08x} to 0x{:08x} (from 0x{:08x}).", word, address, old_value);
        // }

        // // for word in bytes.chunks_exact(4) {
        // //     let value = u32::from_ne_bytes(word.try_into().unwrap());

        // //     assert!(!self.read_failed_bit());
        // //     self.flc.fdata.write(|w| unsafe {
        // //         w.bits(value)
        // //     });
        // //     assert!(!self.read_failed_bit());
        // //     self.flc.ctrl.write(|w| w.write().set_bit());
        // //     assert!(!self.read_failed_bit());
        // //     self.wait_until_not_busy();
        // //     assert!(!self.read_failed_bit());
        // // }

        // self.lock_flash();

        // if self.read_failed_bit() {
        //     Err(nb::Error::Other(Error::WriteFailed))
        // } else {
        //     Ok(())
        // }
    }
}

impl ReadWrite for Flash {
    type Error = Error;
    type Address = Address;

    fn label() -> &'static str { "maxim3263x flash (internal)" }

    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        let (minimum_address, maximum_address) = self.range();
        let is_valid_address = (address >= minimum_address) && (address + bytes.len() <= maximum_address);
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
        let is_valid_address = (address >= minimum_address) && (address + bytes.len() <= maximum_address);
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

        const MASS_ERASE_CODE : u8 = 0xAA;
        self.flc.ctrl.write(|w| unsafe {
            w.erase_code().bits(MASS_ERASE_CODE)
        });

        self.flc.ctrl.write(|w| w.mass_erase().set_bit());

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
