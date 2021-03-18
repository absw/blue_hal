use core::marker::PhantomData;
use core::any::Any;

use super::{clocks::Clocks, gpio::{
    typestate::{Input, Output},
    *,
}};
use crate::{serial_write, efm32pac, hal::{gpio::{InputPin, OutputPin}, time::Hertz}};
use efm32pac::{UART0, UART1, USART0, USART1, USART2, USART3, USART4, USART5};

mod sealed {
    use super::*;
    pub trait RxPin<USART>: InputPin {}
    pub trait TxPin<USART>: OutputPin {}
}
use sealed::*;

allowed! {
    RxPin<UART0>: [Pf7<Input> Pe1<Input> Pa4<Input> Pc15<Input> Pc5<Input> Pf2<Input> Pe4<Input>]
    RxPin<UART1>: [Pc13<Input> Pf11<Input> Pb10<Input> Pe3<Input> Pe13<Input> Ph12<Input>]
    RxPin<USART0>: [Pe11<Input> Pe6<Input> Pc10<Input> Pe12<Input> Pb8<Input> Pc1<Input> Pg13<Input>]
    RxPin<USART1>: [Pc1<Input> Pd1<Input> Pd6<Input> Pf7<Input> Pc2<Input> Pa0<Input> Pa2<Input>]
    RxPin<USART2>: [Pc3<Input> Pb4<Input> Pa8<Input> Pa14<Input> Pf7<Input> Pf1<Input>]
    RxPin<USART3>: [Pa1<Input> Pe7<Input> Pb7<Input> Pg7<Input> Pg1<Input> Pl13<Input>]
    RxPin<USART4>: [Pb8<Input> Pd10<Input> Pl1<Input> Pl7<Input> Ph5<Input>]
    RxPin<USART5>: [Pe9<Input> Pa7<Input> Pb1<Input> Ph11<Input>]

    TxPin<UART0>: [Pf6<Output> Pe0<Output> Pa3<Output> Pc14<Output> Pc4<Output> Pf1<Output> Pd7<Output>]
    TxPin<UART1>: [Pc12<Output> Pf10<Output> Pb9<Output> Pe2<Output> Pe12<Output> Ph11<Output>]
    TxPin<USART0>: [Pe10<Output> Pe7<Output> Pc11<Output> Pe13<Output> Pb7<Output> Pc0<Output> Pg12<Output>]
    TxPin<USART1>: [Pc0<Output> Pd0<Output> Pd7<Output> Pf6<Output> Pc1<Output> Pf2<Output> Pa14<Output>]
    TxPin<USART2>: [Pc2<Output> Pb3<Output> Pa7<Output> Pa13<Output> Pf6<Output> Pf0<Output>]
    TxPin<USART3>: [Pa0<Output> Pe6<Output> Pb3<Output> Pg6<Output> Pg0<Output> Pl12<Output>]
    TxPin<USART4>: [Pb7<Output> Pd9<Output> Pl0<Output> Pl6<Output> Ph4<Output>]
    TxPin<USART5>: [Pe8<Output> Pa6<Output> Pf15<Output> Ph10<Output>]
}

pub struct Serial<U, TX: TxPin<U>, RX: RxPin<U>> {
    tx: TX,
    rx: RX,
    peripheral: U,
}

impl<U: Any, TX: TxPin<U>, RX: RxPin<U>> Serial<U, TX, RX> {
    const BAUD_RATE: u32 = 115200;
    const OVERSAMPLE: u32 = 16;

    pub fn new(peripheral: U, tx: TX, rx: RX, clocks: &Clocks) -> Self {
        let mut serial = Self { peripheral, tx, rx };
        serial.set_baud_rate(clocks);
        serial
    }

    fn set_baud_rate(&mut self, clocks: &Clocks) {
        let Hertz(frequency) = clocks.get_frequency_hfclk();
        // Operate in u64 to avoid overflow
        let divider = ((256 * frequency as u64) / ((Self::OVERSAMPLE * Self::BAUD_RATE) - 256) as u64) as u32;

        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe { serial_write!(&self.peripheral, clkdiv, |w| { w.bits(divider)}); }
    }

}

#[macro_export(local_inner_macros)]
macro_rules! serial_write_inner {
    ([$($serial:ident)+] $peripheral:expr, $register_name:ident, |$write:ident| $block:block) => {
        $(
            if let Some(p) = ($peripheral as &dyn Any).downcast_ref::<$serial>() {
                p.$register_name.write(|$write| $block);
            }
        )+
    };
}

#[macro_export(local_inner_macros)]
macro_rules! serial_write {
    ($peripheral:expr, $register_name:ident, |$write:ident| $block:block) => {
        serial_write_inner!(
            [UART0 UART1 USART0 USART1 USART2 USART3 USART4 USART5]
            $peripheral, $register_name, |$write| $block
        );
    };
}
