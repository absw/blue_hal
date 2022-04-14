use core::marker::PhantomData;
use crate::hal::spi;

pub unsafe trait ClkPin {}
pub unsafe trait CsPin {}
pub unsafe trait Io0Pin {}
pub unsafe trait Io1Pin {}

pub struct Spi<PINS> {
    // TODO: Require PINS type parameter for later use.
    _phantom: PhantomData<PINS>,
}

pub enum Error {}

#[macro_export(local_inner_macros)]
macro_rules! enable_qspi {
    () => {
        #[cfg(feature = "max32631")]
        seal_pins!(blue_hal::drivers::max3263::spi::ClkPin: [P10]);
        #[cfg(feature = "max32631")]
        seal_pins!(blue_hal::drivers::max3263::spi::CsPin: [P13]);
        #[cfg(feature = "max32631")]
        seal_pins!(blue_hal::drivers::max3263::spi::Io0Pin: [P11]);
        #[cfg(feature = "max32631")]
        seal_pins!(blue_hal::drivers::max3263::spi::Io1Pin: [P12]);
    }
}

impl<PINS> Spi<PINS> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<PINS> spi::FullDuplex<u8> for Spi<PINS> {
    type Error = Error;

    fn transmit(&mut self, _word: Option<u8>) -> nb::Result<(), Self::Error> {
        todo!()
    }

    fn receive(&mut self) -> nb::Result<u8, Self::Error> {
        todo!()
    }
}
