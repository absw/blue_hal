use core::marker::PhantomData;
use efm32pac::{CMU, GPIO};

use crate::{
    construct_gpio, efm32pac, gpio_modify, gpio_read, gpio_struct, gpio_write,
    hal::gpio::{InputPin, OutputPin, TogglePin},
    matrix,
};

/// Pin mode typestates
pub mod typestate {
    /// Pin is disabled or not yet configured.
    pub struct NotConfigured;
    /// Pin is enabled as an input.
    pub struct Input;
    /// Pin is enabled as a push-pull output.
    pub struct Output;
}
use typestate::*;

// Define a Gpio struct with 15 pins and ports A to L
matrix! {
    gpio_struct
    [(a 'A') (b 'B') (c 'C') (d 'D') (e 'E') (f 'F') (g 'G') (h 'H') (i 'I') (j 'J') (k 'K') (l 'L')]
    [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15]
}

impl Gpio {
    pub fn new(_: GPIO, pac_cmu: &mut CMU) -> Self {
        // Enable GPIO clock
        pac_cmu.hfbusclken0.write(|w| w.gpio().set_bit());
        matrix! { construct_gpio [a b c d e f g h i j k l] [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15] }
    }
}

/// Generic pin. Modes are defined via typestate.
// The pin structure is a zero sized, pure compile time construct that allows us exclusive
// access to that section of the GPIO register block; hence why this struct has nothing
// inside. It exists only to help the type system reason about ownership of a particular pin.
pub struct Pin<MODE, const PORT: char, const INDEX: u8> {
    _marker: PhantomData<MODE>,
}

/// Bit representation of pin modes.
enum Mode {
    Disabled = 0,
    Input = 1,
    PushPullOutput = 4,
    Mask = 0b1111,
}

impl<MODE, const PORT: char, const INDEX: u8> Pin<MODE, PORT, INDEX> {

    // Inner constructor. Only the GPIO module can call it to ensure
    // there only exists one of each pin.
    fn new() -> Self { Self { _marker: Default::default() } }

    /// Sets the pin to input mode.
    pub fn as_input(self) -> Pin<Input, PORT, INDEX> {
        Self::set_mode(Mode::Input);
        Pin::new()
    }

    /// Sets the pin to output mode.
    pub fn as_output(self) -> Pin<Output, PORT, INDEX> {
        Self::set_mode(Mode::PushPullOutput);
        Pin::new()
    }

    /// Disables the pin.
    pub fn as_disabled(self) -> Pin<NotConfigured, PORT, INDEX> {
        Self::set_mode(Mode::Disabled);
        Pin::new()
    }

    fn set_mode(mode: Mode) {
        let offset = (4 * INDEX) % 32;
        // Safety: We can interact with the GPIO register block, as we promise
        // we only modify bits related to this specific pin and port, and nothing
        // else has control over those bits.
        let high_register_threshold = 8u8;
        let mask = Mode::Mask as u32;
        let mode = mode as u32;

        // Safety: See above. Also, unsafe access is required to write multiple bits,
        // as the compiler cannot guarantee the bits written are the correct ones not
        // to leave the chip in an undefined state. Must be verified my manual review.
        unsafe {
            if INDEX >= high_register_threshold {
                gpio_modify!(PORT, modeh, |r, w| {
                    w.bits((r.bits() & !(mask << offset)) | (mode << offset))
                });
            } else {
                gpio_modify!(PORT, model, |r, w| {
                    w.bits((r.bits() & !(mask << offset)) | (mode << offset))
                });
            };
        }

    }
}

impl<const PORT: char, const INDEX: u8> OutputPin for Pin<Output, PORT, INDEX> {
    fn set_low(&mut self) {
        // Safety: We can interact with the GPIO register block, as we promise
        // we only modify bits related to this specific pin and port, and nothing
        // else has control over those bits.
        unsafe { gpio_modify!(PORT, dout, |r, w| { w.bits(r.bits() & !(1 << INDEX)) }) }
    }

    fn set_high(&mut self) {
        // Safety: We can interact with the GPIO register block, as we promise
        // we only modify bits related to this specific pin and port, and nothing
        // else has control over those bits.
        unsafe { gpio_modify!(PORT, dout, |r, w| {
            w.bits(r.bits() | (1 << INDEX)) }) }
    }
}

impl<const PORT: char, const INDEX: u8> TogglePin for Pin<Output, PORT, INDEX> {
    fn toggle(&mut self) {
        unsafe { gpio_write!(PORT, douttgl, |w| { w.bits(1 << INDEX) }) }
    }
}

impl<const PORT: char, const INDEX: u8> InputPin for Pin<Input, PORT, INDEX> {
    fn is_high(&self) -> bool {
        unsafe { gpio_read!(PORT, din, |r| { r.bits() >> INDEX & 1 == 1 }) }
    }

    fn is_low(&self) -> bool { !self.is_high() }
}

/// Reads from the specific GPIO register associated to a given port
#[macro_export(local_inner_macros)]
macro_rules! gpio_read {
    ($port:ident, $register_name:ident, |$read:ident| $block:block) => {
        gpio_read_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I' 'J' 'K' 'L']
            [a b c d e f g h i j k l]
            $port, $register_name, |$read| $block
        );
    };
}

/// Modifies the specific GPIO register associated to a given port
#[macro_export(local_inner_macros)]
macro_rules! gpio_modify {
    ($port:ident, $register_name:ident, |$read:ident, $write:ident| $block:block) => {
        gpio_modify_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I' 'J' 'K' 'L']
            [a b c d e f g h i j k l]
            $port, $register_name, |$read, $write| $block
        );
    };
}

/// Writes to the specific GPIO register associated to a given port
#[macro_export(local_inner_macros)]
macro_rules! gpio_write {
    ($port:ident, $register_name:ident, |$write:ident| $block:block) => {
        gpio_write_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I' 'J' 'K' 'L']
            [a b c d e f g h i j k l]
            $port, $register_name, |$write| $block
        );
    };
}

/// Defines a GPIO struct
#[macro_export(local_inner_macros)]
macro_rules! gpio_struct {
    ($( ($letter:tt $character:tt) $number:tt )*) => { paste::item! {
        pub struct Gpio {
            $(
                pub [<p $letter $number>]: Pin<NotConfigured, $character, $number>,
            )+
        }
    }}
}

/// Initialises the members of a GPIO struct
#[macro_export(local_inner_macros)]
macro_rules! construct_gpio {
    ($($letter:ident $number: literal)*) => { paste::item! {
        Self {
            $([<p $letter $number>]: Pin::new(),)*
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_read_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident, |$read:ident| $block:block) => {
        paste::item! {
            #[allow(unused_braces)]
            match PORT {
                $($character => { let $read = (*GPIO::ptr()).[<p $letter _ $register_name>].read(); $block }, )+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_modify_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident, |$read:ident, $write:ident| $block:block) => {
        paste::item! {
            #[allow(unused_braces)]
            match PORT {
                $($character => { (*GPIO::ptr()).[<p $letter _ $register_name>].modify(|$read, $write| $block) })+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_write_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident, |$write:ident| $block:block) => {
        paste::item! {
            #[allow(unused_braces)]
            match PORT {
                $($character => { (*GPIO::ptr()).[<p $letter _ $register_name>].write(|$write| $block) })+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}
