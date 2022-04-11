use core::marker::PhantomData;
use crate::hal::qspi;

pub unsafe trait ClkPin {}
pub unsafe trait CsPin {}
pub unsafe trait Io0Pin {}
pub unsafe trait Io1Pin {}

pub struct Qspi<PINS> {
    // TODO: Require PINS type parameter for later use.
    _phantom: PhantomData<PINS>,
}

pub enum Error {}

#[macro_export(local_inner_macros)]
macro_rules! enable_qspi { () => {
    // TODO: Add pins to this:
    #[cfg(feature = "max32631")]
    seal_pins!(blue_hal::drivers::max3263::qspi::ClkPin: [P10]);
    #[cfg(feature = "max32631")]
    seal_pins!(blue_hal::drivers::max3263::qspi::CsPin: [P13]);
    #[cfg(feature = "max32631")]
    seal_pins!(blue_hal::drivers::max3263::qspi::Io0Pin: [P11]);
    #[cfg(feature = "max32631")]
    seal_pins!(blue_hal::drivers::max3263::qspi::Io1Pin: [P12]);
}}

impl<PINS> Qspi<PINS> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<PINS> qspi::Indirect for Qspi<PINS> {
    type Error = Error;

    fn write(
        &mut self,
        _instruction: Option<u8>,
        _address: Option<u32>,
        _data: Option<&[u8]>,
        _dummy_cycles: u8,
    ) -> nb::Result<(), Error> {
        todo!()
    }

    fn read(
        &mut self,
        _instruction: Option<u8>,
        _address: Option<u32>,
        _data: &mut [u8],
        _dummy_cycles: u8,
    ) -> nb::Result<(), Error> {
        todo!()
    }
}
