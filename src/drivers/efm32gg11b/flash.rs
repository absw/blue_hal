use core::ops::{Add, Sub};

use bytemuck::cast_slice;
use cortex_m_semihosting::hprintln;
use efm32gg11b::{CMU, MSC};

use crate::{hal::flash::ReadWrite, utilities::memory::Region, utilities::memory::IterableByOverlaps};

pub struct Flash {
    msc: MSC
}

#[derive(Copy, Clone, Debug)]
pub struct Map;

#[derive(Copy, Clone, Debug)]
pub struct Page(pub u16);


impl Map {
    pub fn pages() -> impl Iterator<Item=Page> { (0..count::PAGES as u16).map(Page) }
}

impl Page {
    pub fn address(&self) -> Address { Address(self.0 as u32 * size::PAGE as u32)}
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    MemoryNotReachable,
    MemoryIsLocked,
    InvalidAddress,
    MisalignedAccess,
}

impl Region<Address> for Map {
    fn contains(&self, address: Address) -> bool {
        address < Address(size::PAGE as u32 * count::PAGES as u32)
    }
}
impl Region<Address> for Page {
    fn contains(&self, address: Address) -> bool {
        let start = Address((size::PAGE * self.0 as usize) as u32);
        (address >= start) && (address < start + size::PAGE)
    }
}

#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct Address(pub u32);

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

impl Flash {
    pub fn new(msc: MSC) -> Self {
         const MSC_UNLOCK_CODE: u32 = 0x1B71;
         // Safety: Unsafe access here is required only to write
         // multiple bits at once to the same register. We must ensure
         // that we write bits that leave the peripheral in a known and
         // correct state.
         unsafe { msc.lock.write(|w| w.bits(MSC_UNLOCK_CODE)); }
         msc.writectrl.write(|w| w.wren().set_bit());
         Self { msc }
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn is_busy(&self) -> bool { self.msc.status.read().busy().bit_is_set() }

    #[link_section = ".data"] // Must be executed from RAM
    fn wait_until_not_busy(&self) { while self.is_busy() {} }

    #[link_section = ".data"] // Must be executed from RAM
    fn wait_until_ready_to_write(&self) { while self.msc.status.read().wdataready().bit_is_clear() {} }

    #[link_section = ".data"] // Must be executed from RAM
    fn erase_page(&mut self, page: Page) -> nb::Result<(), Error> {
        if self.is_busy() { return Err(nb::Error::WouldBlock); }
        self.load_address(page.address())?;
        self.msc.writecmd.write(|w| w.erasepage().set_bit());
        self.wait_until_not_busy();
        Ok(())
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn load_address(&self, Address(value): Address) -> nb::Result<(), Error> {
        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe { self.msc.addrb.write(|w| w.bits(value)) }
        self.msc.writecmd.write(|w| w.laddrim().set_bit());
        self.verify_status()
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn verify_status(&self) -> nb::Result<(), Error> {
        let error = self.msc.status.read().invaddr().bit_is_set().then_some(Error::InvalidAddress)
            .or(self.msc.status.read().locked().bit_is_set().then_some(Error::MemoryIsLocked));

        if let Some(error) = error {
            Err(nb::Error::Other(error))
        } else {
            Ok(())
        }
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn write_page(
        &mut self,
        bytes: &[u8],
        page: Page,
    ) -> nb::Result<(), Error> {
        if bytes.len() != size::PAGE {
            return Err(nb::Error::Other(Error::MisalignedAccess));
        }
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }
        // We know this to be aligned, so it can't fail.
        let words: &[u32] = cast_slice(bytes);

        self.load_address(page.address())?;
        for word in words {
            self.wait_until_ready_to_write();
            // Safety: Unsafe required to write the entire word at once to a register.
            unsafe { self.msc.wdata.write(|w| w.bits(*word)) }
            self.msc.writecmd.write(|w| w.writeonce().set_bit());
            self.wait_until_not_busy();
        }

        Ok(())
    }
}

impl Drop for Flash {
    fn drop(&mut self) {
         // Safety: Unsafe access here is required only to write
         // multiple bits at once to the same register. We must ensure
         // that we write bits that leave the peripheral in a known and
         // correct state.
        unsafe { self.msc.lock.write(|w| w.bits(0));}
        self.msc.writectrl.write(|w| w.wren().clear_bit());
    }
}

impl ReadWrite for Flash {
    type Error = Error;

    type Address = Address;

    fn label() -> &'static str {
        "efm32gg11b flash (Internal)"
    }

    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        let inside_map = Map.contains(address) && Map.contains(address + bytes.len());
        if !inside_map {
            Err(nb::Error::Other(Error::MemoryNotReachable))
        } else {
            let base = address.0 as *const u8;
            for (index, byte) in bytes.iter_mut().enumerate() {
                // NOTE(Safety) we are reading directly from raw memory locations,
                // which is inherently unsafe.
                *byte = unsafe { *(base.add(index)) };
            }
            Ok(())
        }
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        let Address(address_value) = address;
        let correctly_aligned_start = address_value & 0b11 == 0;
        let correctly_aligned_end = bytes.len() & 0b11 == 0;
        if !correctly_aligned_end || !correctly_aligned_start {
            return Err(nb::Error::Other(Error::MisalignedAccess))
        }
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        for (block, page, address) in Map::pages().overlaps(bytes, address) {
            let page_data = &mut [0u8; size::PAGE];
            nb::block!(self.read(page.address(), page_data))?;
            let offset_into_page = address.0.saturating_sub(page.address().0) as usize;
            page_data
                .iter_mut()
                .skip(offset_into_page)
                .zip(block)
                .for_each(|(byte, input)| *byte = *input);
            nb::block!(self.erase_page(page))?;
            nb::block!(self.write_page(page_data, page))?;
        }

        Ok(())
    }

    fn range(&self) -> (Self::Address, Self::Address) {
        (Address(0), Address(0) + count::PAGES * size::PAGE)
    }

    #[link_section = ".data"] // Must be executed from RAM
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        if self.is_busy() { return Err(nb::Error::WouldBlock); }
        const MSC_MASS_ERASE_CODE: u32 = 0x0000631A;
        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe { self.msc.masslock.write(|w| w.bits(MSC_MASS_ERASE_CODE)) }
        self.msc.writecmd.write(|w| w.erasemain0().set_bit());
        self.wait_until_not_busy();
        self.msc.writecmd.write(|w| w.erasemain1().set_bit());
        self.wait_until_not_busy();
        unsafe { self.msc.masslock.write(|w| w.bits(0)) }
        Ok(())
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
        nb::block!(self.write(address + (memory_index - remainder.len()), &remainder))
    }
}


mod size {
    use super::count;
    pub const PAGE: usize = KB!(4);
}

mod count {
    pub const PAGES: usize = 512;
}

