use super::{
    flash,
    serial::{Read, TimeoutRead, Write},
    time,
    update_signal::{UpdateSignal, UpdateSignalResult},
};

pub type NullAddress = usize;

pub struct NullSerial;

#[derive(Copy, Clone, Debug)]
pub struct NullError;

impl Read for NullSerial {
    type Error = NullError;
    fn read(&mut self) -> nb::Result<u8, Self::Error> { unimplemented!() }
}

impl Write for NullSerial {
    type Error = NullError;

    fn write_str(&mut self, _: &str) -> Result<(), Self::Error> { unimplemented!() }
}

impl TimeoutRead for NullSerial {
    type Error = NullError;
    fn read<T: Copy + Into<super::time::Milliseconds>>(&mut self, _: T) -> Result<u8, Self::Error> {
        unimplemented!()
    }
}

pub struct NullFlash;

impl flash::ReadWrite for NullFlash {
    type Error = NullError;
    type Address = NullAddress;

    fn label() -> &'static str {
        unimplemented!();
    }
    fn read(&mut self, _: Self::Address, _: &mut [u8]) -> nb::Result<(), Self::Error> {
        unimplemented!()
    }
    fn write(&mut self, _: Self::Address, _: &[u8]) -> nb::Result<(), Self::Error> {
        unimplemented!()
    }
    fn range(&self) -> (Self::Address, Self::Address) { unimplemented!() }
    fn erase(&mut self) -> nb::Result<(), Self::Error> { unimplemented!() }
    fn write_from_blocks<I: Iterator<Item = [u8; N]>, const N: usize>(
        &mut self,
        _: Self::Address,
        _: I,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

pub struct NullSystick;

#[derive(Copy, Clone, Debug)]
pub struct NullInstant;

impl core::ops::Sub for NullInstant {
    type Output = time::Milliseconds;
    fn sub(self, _: Self) -> Self::Output { time::Milliseconds(0) }
}

/// Addition between any Millisecond-convertible type and the current tick.
impl<T: Into<time::Milliseconds>> core::ops::Add<T> for NullInstant {
    type Output = Self;
    fn add(self, _: T) -> Self { Self {} }
}

impl time::Now for NullSystick {
    type I = NullInstant;
    fn now() -> Self::I { unimplemented!() }
}

pub struct NullUpdateSignal;

impl UpdateSignal for NullUpdateSignal {
    fn should_update(&self) -> UpdateSignalResult { UpdateSignalResult::Any }
}
