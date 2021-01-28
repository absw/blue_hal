/// Data used during a QSPI operation.
pub enum Data<'a> {
    WriteNone,
    Read(&'a mut [u8]),
    Write(&'a [u8]),
}

/// Builder class for QSPI operations.
pub struct QSPICommand<'a> {
    instruction: Option<u8>,
    address: Option<u32>,
    data: Data<'a>,
    dummy_cycles: u8,
}

impl<'a> Default for QSPICommand<'a> {
    fn default() -> Self {
        Self {
            instruction: None,
            address: None,
            data: Data::WriteNone,
            dummy_cycles: 0,
        }
    }
}

impl<'a> QSPICommand<'a> {
    pub fn with_instruction(self, instruction: u8) -> Self {
        Self { instruction: Some(instruction), ..self }
    }

    pub fn with_address(self, address: u32) -> Self {
        Self { address: Some(address), ..self }
    }

    pub fn with_data(self, data: Data<'a>) -> Self {
        Self { data, ..self }
    }

    pub fn with_read_data(self, data: &'a mut [u8]) -> Self {
        Self { data: Data::Read(data), ..self }
    }

    pub fn with_write_data(self, data: &'a mut [u8]) -> Self {
        Self { data: Data::Write(data), ..self }
    }

    pub fn with_dummy_cycles(self, dummy_cycles: u8) -> Self {
        Self { dummy_cycles, ..self }
    }

    pub fn instruction(&self) -> Option<u8> { self.instruction }

    pub fn address(&self) -> Option<u32> { self.address }

    pub fn data_ref(&self) -> &Data { &self.data }
    pub fn data_mut(&mut self) -> &mut Data<'a> { &mut self.data }
    pub fn data(self) -> Data<'a> { self.data }

    pub fn dummy_cycles(&self) -> u8 { self.dummy_cycles }
}

/// Quad SPI configured in Indirect mode.
///
/// Indirect mode forces all communication to occur through writes
/// and reads to QSPI registers.
pub trait Indirect {
    type Error;

    fn execute_command(&mut self, command: &mut QSPICommand) -> nb::Result<(), Self::Error>;
}
