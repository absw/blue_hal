use core::marker::PhantomData;

use crate::utilities::safety::LimitedU8;

// Mode typestates
pub struct Input;
pub struct Output;
pub struct AlternateFunction<const INDEX: u8>;

pub struct Pin<MODE, const PORT: char, const INDEX: u8> {
    _marker: PhantomData<MODE>,
}

impl <MODE, const PORT: char, const INDEX: u8> Pin<MODE, PORT, INDEX> {
    fn new() -> Self { Self { _marker: Default::default() } }
}

const FIRST_PORT: char = 'A';
const LAST_PORT: char = 'L';
const TOTAL_PORTS: usize = 1 + LAST_PORT as usize - FIRST_PORT as usize;
const fn is_valid_port(port: char) -> bool { port >= FIRST_PORT && port <= LAST_PORT }
const fn is_valid_index(index: u8) -> bool { index < 16 }

pub struct Gpio {
    claimed: [u16; TOTAL_PORTS]
}

impl Gpio {
    fn new() -> Self {
        Self { claimed: [0u16; TOTAL_PORTS] }
    }

    fn is_claimed<const PORT: char, const INDEX: u8>(&self) -> bool {
        let entry = PORT as usize - 'A' as usize;
        self.claimed[entry] >> INDEX & 0b1 == 0b1
    }

    fn set_as_claimed<const PORT: char, const INDEX: u8>(&mut self) {
        let entry = PORT as usize - 'A' as usize;
        self.claimed[entry] |= 0b1 << INDEX;
    }

    pub fn claim_pin_as_input<const PORT: char, const INDEX: u8>(&mut self) -> Option<Pin<Input, PORT, INDEX>> {
        if self.is_claimed::<PORT, INDEX>() || !is_valid_port(PORT) || !is_valid_index(INDEX) {
            return None;
        }

        self.set_as_claimed::<PORT, INDEX>();
        Some(Pin::new())
    }
    pub fn claim_pin_as_output<const PORT: char, const INDEX: u8>(&mut self) -> Option<Pin<Output, PORT, INDEX>> {
        if self.is_claimed::<PORT, INDEX>() ||!is_valid_port(PORT) || !is_valid_index(INDEX) {
            return None;
        }

        self.set_as_claimed::<PORT, INDEX>();
        Some(Pin::new())
    }
    pub fn claim_pin_with_alternate_function<const AF_INDEX: u8, const PORT: char, const INDEX: u8>(&mut self
    ) -> Option<Pin<AlternateFunction<AF_INDEX>, PORT, INDEX>> {
        if self.is_claimed::<PORT, INDEX>() || !is_valid_port(PORT) || !is_valid_index(INDEX) {
            return None;
        }

        self.set_as_claimed::<PORT, INDEX>();
        Some(Pin::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pins_can_only_be_claimed_once() {
        let mut gpio = Gpio::new();
        assert!(gpio.claim_pin_as_output::<'A', 3>().is_some());
        assert!(gpio.claim_pin_as_output::<'A', 3>().is_none());

        assert!(gpio.claim_pin_as_output::<'C', 8>().is_some());
        assert!(gpio.claim_pin_as_output::<'C', 8>().is_none());
        assert!(gpio.claim_pin_as_output::<'C', 8>().is_none());
    }
}
