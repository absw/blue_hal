//! SPI driver implementation for the MAX32631.
//!
//! Currently, only 4-wire SPI over the SPIM1 peripheral is supported.

use core::marker::PhantomData;
use crate::hal::spi;
use crate::drivers::max3263::gpio::*;

/// A pin which may be used as an SPI clock pin.
pub unsafe trait ClkPin {}

/// A pin which may be used as an SPI chip select pin.
pub unsafe trait CsPin {}

/// A pin which may be used as an SPI IO0 (MOSI) pin.
pub unsafe trait Io0Pin {}

/// A pin which may be used as an SPI IO1 (MISO) pin.
pub unsafe trait Io1Pin {}

// Trait implementations for SPI pins.
unsafe impl ClkPin for P10<AF0> {}
unsafe impl CsPin for P13<AF0> {}
unsafe impl Io0Pin for P11<AF0> {}
unsafe impl Io1Pin for P12<AF0> {}

pub struct Spi<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> {
    // TODO: Require PINS type parameter for later use.
    _phantom: PhantomData<(CK, CS, IO0, IO1)>,
}

pub enum Error {}

impl<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> Spi<CK, CS, IO0, IO1> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> spi::FullDuplex<u8> for Spi<CK, CS, IO0, IO1> {
    type Error = Error;

    fn transmit(&mut self, _word: Option<u8>) -> nb::Result<(), Self::Error> {
        todo!()
    }

    fn receive(&mut self) -> nb::Result<u8, Self::Error> {
        todo!()
    }
}
