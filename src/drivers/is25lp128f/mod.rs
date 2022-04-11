use crate::hal::flash::ReadWrite;
use core::ops::{Add, Sub};

pub struct Is25Lp128F {}

#[derive(Clone, Copy)]
pub enum Error {}

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

impl Is25Lp128F {
    pub fn new() -> Self {
        Self {}
    }
}

impl ReadWrite for Is25Lp128F {
    type Error = Error;
    type Address = Address;

    fn label() -> &'static str {
        "is25lp128f"
    }

    fn read(&mut self, _address: Address, _bytes: &mut [u8]) -> nb::Result<(), Error> {
        todo!()
    }

    fn write(&mut self, _address: Address, _bytes: &[u8]) -> nb::Result<(), Error> {
        todo!()
    }

    fn range(&self) -> (Self::Address, Self::Address) {
        todo!()
    }

    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }

    fn write_from_blocks<I: Iterator<Item = [u8; N]>, const N: usize>(
        &mut self,
        _address: Self::Address,
        _blocks: I,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
