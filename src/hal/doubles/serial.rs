use crate::hal::{serial, time};

#[derive(Copy, Clone, Debug)]
pub struct SerialStubError;
pub struct SerialStub;

impl serial::Write for SerialStub {
    type Error = SerialStubError;
    fn write_str(&mut self, _s: &str) -> Result<(), Self::Error> { Ok(()) }
}

impl serial::Read for SerialStub {
    type Error = SerialStubError;
    fn read(&mut self) -> nb::Result<u8, Self::Error> { Ok(0) }
}

impl serial::TimeoutRead for SerialStub {
    type Error = SerialStubError;

    fn read<T: Copy + Into<time::Milliseconds>>(&mut self, _timeout: T) -> Result<u8, Self::Error> {
        Ok(0)
    }
}
