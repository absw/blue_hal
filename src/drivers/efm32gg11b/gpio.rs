use core::marker::PhantomData;
use core::ops::Deref;
use efm32pac::{CMU, GPIO, cmu, generic::Writable, gpio};

use crate::{construct_gpio, efm32pac, gpio_modify, matrix, hal::gpio::OutputPin, gpio_struct};

// Mode typestates
pub struct NotConfigured;
pub struct Input;
pub struct Output;
pub struct AlternateFunction<const INDEX: u8>;

pub struct Pin<MODE, const PORT: char, const INDEX: u8> {
    _marker: PhantomData<MODE>,
}

enum Mode {
    Disabled = 0,
    Input = 1,
    PushPullOutput = 4,
    Mask = 0b1111,
}

impl <MODE, const PORT: char, const INDEX: u8> Pin<MODE, PORT, INDEX> {
    // Inner constructor. Only the GPIO module can call it to ensure
    // there only exists one of each pin.
    fn new() -> Self { Self { _marker: Default::default() } }

    pub fn as_input(self) -> Pin<Input, PORT, INDEX>  {
        Self::set_mode(Mode::Input);
        Pin::new()
    }

    pub fn as_output(self) -> Pin<Output, PORT, INDEX>  {
        Self::set_mode(Mode::PushPullOutput);
        Pin::new()
    }

    pub fn as_disabled(self) -> Pin<NotConfigured, PORT, INDEX>  {
        Self::set_mode(Mode::Disabled);
        Pin::new()
    }

    pub fn as_alternate_function<const AF: u8>(self) -> Pin<AlternateFunction<AF>, PORT, INDEX>  {
        unimplemented!()
    }

    fn set_mode(mode: Mode) {
        // Safety: We can interact with the GPIO register block, as we promise
        // we only modify bits related to this specific pin and port, and nothing
        // else has control over those bits.
        let gpio = GPIO::ptr();
        let high_register_threshold = 8u8;
        let offset = (4 * INDEX) % 32;
        let mask = Mode::Mask as u32;
        let mode = mode as u32;

        // Safety: See above. Also, unsafe access is required to write multiple bits,
        // as the compiler cannot guarantee the bits written are the correct ones not
        // to leave the chip in an undefined state. Must be verified my manual review.
        if INDEX >= high_register_threshold {
            unsafe { gpio_modify!(gpio, PORT, modeh, |r, w| { w.bits((r.bits() & !(mask << offset)) | (mode << offset))}); }
        } else {
            unsafe { gpio_modify!(gpio, PORT, model, |r, w| { w.bits((r.bits() & !(mask << offset)) | (mode << offset))}); }
        };
    }
}

impl <const PORT: char, const INDEX: u8> OutputPin for Pin<Output, PORT, INDEX> {
    fn set_low(&mut self) {
        todo!()
    }

    fn set_high(&mut self) {
        todo!()
    }
}

matrix! {
    gpio_struct
    [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15]
    [a:'A' b:'B' c:'C' d:'D' e:'E' f:'F' g:'G' h:'H' i:'I' j:'J' k: 'K' l:'L']
}

impl Gpio {
    pub fn new(_: GPIO, pac_cmu: &mut CMU) -> Self {
        pac_cmu.hfbusclken0.write(|w| w.gpio().set_bit());
        construct_gpio!(a b c d e f g h i j k l)
    }
}

//#[macro_export(local_inner_macros)]
//macro_rules! gpio_struct {
//    ($($letter:ident $character:literal $number: literal,)+) => { paste::item! {
//        pub struct Gpio {
//            $(
//                pub [<p $letter $number>]: Pin<NotConfigured, $character, $number>,
//            )+
//        }
//    }}
//}
//
//#[macro_export(local_inner_macros)]
//macro_rules! gpio_struct {
//    ($($l:ident: $c:literal,)+) => {
//        gpio_struct!(
//           $($l $c 0,)+ $($l $c 1,)+ $($l $c 2,)+ $($l $c 3,)+
//           $($l $c 4,)+ $($l $c 5,)+ $($l $c 6,)+ $($l $c 7,)+
//           $($l $c 8,)+ $($l $c 9,)+ $($l $c 10,)+ $($l $c 11,)+
//           $($l $c 12,)+ $($l $c 13,)+ $($l $c 14,)+ $($l $c 15,)+
//        );
//    }
//}

#[macro_export(local_inner_macros)]
macro_rules! construct_gpio_inner {
    ($($letter:ident $number: literal,)+) => { paste::item! {
        Self {
            $([<p $letter $number>]: Pin::new(),)+
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! construct_gpio {
    ($($l:ident)+) => {
        construct_gpio_inner!(
           $($l 0,)+ $($l 1,)+ $($l 2,)+ $($l 3,)+
           $($l 4,)+ $($l 5,)+ $($l 6,)+ $($l 7,)+
           $($l 8,)+ $($l 9,)+ $($l 10,)+ $($l 11,)+
           $($l 12,)+ $($l 13,)+ $($l 14,)+ $($l 15,)+
        );
    }
}




#[macro_export(local_inner_macros)]
macro_rules! gpio_modify {
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
            _ => core::panic!("Unexpected port"),
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! gpio_struct {
    ($( $letter:tt $character:tt $number:tt )*) => { paste::item! {
        pub struct Gpio {
            $(
                pub [<p $letter $number>]: Pin<NotConfigured, $character, $number>,
            )+
        }
    }}
}

#[macro_export(local_inner_macros)]
macro_rules! matrix {
    ( $inner_macro:ident [$($n:tt)+] $ms:tt) => ( matrix! { $inner_macro $($n $ms)* });
    ( $inner_macro:ident $( $n:tt [$($m:tt:$character:tt)*] )* ) =>
        ( $inner_macro! { $( $( $m $character $n )* )* } );
}

