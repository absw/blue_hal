//! Interface to a SPI peripheral.

/// Allows full duplex communication with a generic `WORD` as a unit
/// of communication.
pub trait FullDuplex<WORD> {
    type Error;

    /// Transmit an optional word. Pass `None` to send an empty (implementation
    /// defined) message, allowing you to receive a word in exchange.
    fn transmit(&mut self, word: Option<WORD>) -> nb::Result<(), Self::Error>;

    /// Receive a word.
    ///
    /// Must be called after transmit (full duplex operation)
    fn receive(&mut self) -> nb::Result<WORD, Self::Error>;
}
