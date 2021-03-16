use core::marker::PhantomData;
use core::ops::Deref;
use efm32pac::{CMU, GPIO, cmu, generic::Writable, gpio};

use crate::{construct_gpio, efm32pac, gpio_struct};

use crate::utilities::safety::LimitedU8;

// Mode typestates
pub struct NotConfigured;
pub struct Input;
pub struct Output;
pub struct AlternateFunction<const INDEX: u8>;

pub struct Pin<MODE, const PORT: char, const INDEX: u8> {
    _marker: PhantomData<MODE>,
}

macro_rules! modify_register {
    ($gpio:ident, $port:ident, $register_name:ident, |$read:ident, $write:ident| $closure:block) => { paste::item! {
        match PORT {
            'A' => (*$gpio).[<pa_ $register_name>].modify(|$read, $write| $closure),
            'B' => (*$gpio).[<pb_ $register_name>].modify(|$read, $write| $closure),
            'C' => (*$gpio).[<pc_ $register_name>].modify(|$read, $write| $closure),
            'D' => (*$gpio).[<pd_ $register_name>].modify(|$read, $write| $closure),
            'E' => (*$gpio).[<pe_ $register_name>].modify(|$read, $write| $closure),
            'F' => (*$gpio).[<pf_ $register_name>].modify(|$read, $write| $closure),
            'G' => (*$gpio).[<pg_ $register_name>].modify(|$read, $write| $closure),
            'H' => (*$gpio).[<ph_ $register_name>].modify(|$read, $write| $closure),
            'I' => (*$gpio).[<pi_ $register_name>].modify(|$read, $write| $closure),
            'J' => (*$gpio).[<pj_ $register_name>].modify(|$read, $write| $closure),
            'K' => (*$gpio).[<pk_ $register_name>].modify(|$read, $write| $closure),
            'L' => (*$gpio).[<pl_ $register_name>].modify(|$read, $write| $closure),
            _ => panic!("Unexpected port"),
        }
    }}
}

impl <MODE, const PORT: char, const INDEX: u8> Pin<MODE, PORT, INDEX> {
    fn new() -> Self { Self { _marker: Default::default() } }

    /// Set to input mode.
    pub fn as_input(self) -> Pin<Input, PORT, INDEX>  {
        // Safety: We can interact with the GPIO register block, as we promise
        // we only modify bits related to this specific pin and port, and nothing
        // else has control over those bits.
        let gpio = GPIO::ptr();
        let high_register_threshold = 8u8;
        let offset = (4 * INDEX) % 32;
        let mode_mask = 0b1111;
        let mode = 0b0001;

        // Safety: See above. Also, unsafe access is required to write multiple bits,
        // as the compiler cannot guarantee the bits written are the correct ones not
        // to leave the chip in an undefined state. Must be verified my manual review.
        if INDEX >= high_register_threshold {
            unsafe { modify_register!(gpio, PORT, modeh, |r, w| { w.bits((r.bits() & !(mode_mask << offset)) | (mode << offset))}); }
        } else {
            unsafe { modify_register!(gpio, PORT, model, |r, w| { w.bits((r.bits() & !(mode_mask << offset)) | (mode << offset))}); }
        };
        Pin::<Input, PORT, INDEX> { _marker: Default::default() }
    }
}

impl<const PORT: char, const INDEX: u8> Default for Pin<NotConfigured, PORT, INDEX> {
    fn default() -> Self { Self { _marker: Default::default() } }
}

gpio_struct!(a: 'A', b: 'B', c: 'C', d: 'D', e: 'E', f: 'F', g: 'G', h: 'H', i: 'I', j: 'J', k: 'K', l: 'L',);

impl Gpio {
    pub fn new(pac_gpio: GPIO, pac_cmu: &mut CMU) -> Self {
        pac_cmu.hfbusclken0.write(|w| w.gpio().set_bit());
        construct_gpio!(pac_gpio, a b c d e f g h i j k l)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_struct_inner {
    ($($letter:ident $character:literal $number: literal,)+) => { paste::item! {
        pub struct Gpio {
            pac_gpio: GPIO,
            $(
                pub [<p $letter $number>]: Pin<NotConfigured, $character, $number>,
            )+
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_struct {
    ($($l:ident: $c:literal,)+) => {
        gpio_struct_inner!(
           $($l $c 0,)+ $($l $c 1,)+ $($l $c 2,)+ $($l $c 3,)+
           $($l $c 4,)+ $($l $c 5,)+ $($l $c 6,)+ $($l $c 7,)+
           $($l $c 8,)+ $($l $c 9,)+ $($l $c 10,)+ $($l $c 11,)+
           $($l $c 12,)+ $($l $c 13,)+ $($l $c 14,)+ $($l $c 15,)+
        );
    }
}

#[macro_export(local_inner_macros)]
macro_rules! construct_gpio_inner {
    ($pac_gpio:ident, $($letter:ident $number: literal,)+) => { paste::item! {
        Self {
            $pac_gpio,
            $([<p $letter $number>]: Default::default(),)+
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! construct_gpio {
    ($pac_gpio:ident, $($l:ident)+) => {
        construct_gpio_inner!(
           $pac_gpio,
           $($l 0,)+ $($l 1,)+ $($l 2,)+ $($l 3,)+
           $($l 4,)+ $($l 5,)+ $($l 6,)+ $($l 7,)+
           $($l 8,)+ $($l 9,)+ $($l 10,)+ $($l 11,)+
           $($l 12,)+ $($l 13,)+ $($l 14,)+ $($l 15,)+
        );
    }
}

#[macro_export(local_inner_macros)]
macro_rules! ports_a_to_l {
    () => {a: 'A', b: 'B', c: 'C', d: 'D', e: 'E', f: 'F', g: 'G', h: 'H', i: 'I', j: 'J', k: 'K', l: 'L',}
}
