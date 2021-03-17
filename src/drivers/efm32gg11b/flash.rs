use core::ops::{Add, Sub};

use efm32gg11b::MSC;

use crate::{hal::flash::ReadWrite, utilities::memory::Region};

pub struct Flash {
    msc: MSC
}

pub struct Map;
pub struct Page(u16);

impl Map {
    pub fn pages() -> impl Iterator<Item=Page> { (0..count::PAGES as u16).map(Page) }
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    MemoryNotReachable,
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

    fn is_busy(&self) -> bool { self.msc.status.read().busy().bit_is_set() }
    fn wait_until_not_busy(&self) { while self.is_busy() {} }
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

    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        todo!()
    }

    fn range(&self) -> (Self::Address, Self::Address) {
        (Address(0), Address(0) + count::PAGES * size::PAGE)
    }

    fn erase(&mut self) -> nb::Result<(), Self::Error> {
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
        todo!()
    }
}


mod size {
    use super::count;
    pub const PAGE: usize = KB!(4);
    pub const MEMORY: usize = PAGE * count::PAGES;
}

mod count {
    pub const PAGES: usize = 512;
}

