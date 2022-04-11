use core::marker::PhantomData;

pub struct AF0 {}
pub struct AF1 {}
pub struct AF2 {}
pub struct AF3 {}
pub struct AF4 {}

/// Declares multiple pin structs named by the provided identifiers. See `declare_pin_struct`.
macro_rules! define_pin_structs {
    ($($name:ident),*) => {
        $(
            define_pin_struct!($name);
        )*
    }
}

/// Declare an empty struct named `name` which takes a type parameter `AF`.
macro_rules! define_pin_struct {
    ($name:ident) => {
        pub struct $name<AF> {
            _phantom: PhantomData<AF>,
        }
    }
}

// Struct definitions for all pins.
define_pin_structs!(
    P00, P01, P02, P03, P04, P05, P06, P07,
    P10, P11, P12, P13, P14, P15, P16, P17,
    P20, P21, P22, P23, P24, P25, P26, P27,
    P30, P31, P32, P33, P34, P35, P36, P37,
    P40, P41, P42, P43, P44, P45, P46, P47,
    P50, P51, P52, P53, P54, P55, P56, P57,
    P60, P61, P62, P63, P64, P65, P66, P67,
    P70, P71, P72, P73, P74, P75, P76, P77,
    P80, P81
);
