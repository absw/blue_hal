use crate::hal::qspi::{self, Mode, Indirect, QSPICommand};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct CommandRecord {
    pub instruction: Option<u8>,
    pub address: Option<u32>,
    pub data: Option<Vec<u8>>,
    pub length_requested: usize,
    pub dummy_cycles: u8,
}

impl CommandRecord {
    pub fn contains(&self, data: &[u8]) -> bool {
        if let Some(stored) = &self.data {
            data.len() == stored.len() && stored.iter().zip(data.iter()).all(|(a, b)| a == b)
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct MockQspi {
    pub mode: Mode,
    pub command_records: Vec<CommandRecord>,
    pub to_read: VecDeque<Vec<u8>>,
}

impl MockQspi {
    pub fn clear(&mut self) {
        self.command_records.clear();
        self.to_read.clear();
    }
}

impl Indirect for MockQspi {
    type Error = ();

    fn execute_command(&mut self, command: &mut QSPICommand) -> nb::Result<(), Self::Error> {
        let (length_requested, data) = match command.data_ref() {
            qspi::Data::WriteNone => (0, None),
            qspi::Data::Write(data) => (0, Some(data.to_vec())),
            qspi::Data::Read(data) => (data.len(), Some(data.to_vec())),
        };

        self.command_records.push(CommandRecord {
            instruction: command.instruction(),
            address: command.address(),
            data,
            length_requested,
            dummy_cycles: command.dummy_cycles(),
        });

        if let qspi::Data::Read(data) = command.data_mut() {
            data.iter_mut()
                .zip(self.to_read.pop_front().unwrap_or_default())
                .for_each(|(o, i)| *o = i);
        }

        Ok(())
    }
}
