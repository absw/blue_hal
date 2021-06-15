//! Update signal implementation.
use crate::hal::update_signal::*;

// TODO: Replace with useful implementation.

pub struct FixedUpdateSignal {
    value: bool,
}

impl FixedUpdateSignal {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl UpdateSignal for FixedUpdateSignal {
    fn should_update(&self) -> UpdateSignalResult {
        if self.value {
            UpdateSignalResult::Any
        } else {
            UpdateSignalResult::None
        }
    }
}