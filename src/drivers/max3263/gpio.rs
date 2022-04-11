use core::marker::PhantomData;

/// Declares multiple pin structs named by the provided identifiers. Each struct expects a type
/// parameter bound by the `AlternativeFunction` trait.
macro_rules! define_pin_structs {
    ($($name:ident),*) => {
        $(
            pub struct $name<AF: AlternativeFunction> {
                _phantom: PhantomData<AF>,
            }
        )*
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

pub struct AF0 {}
pub struct AF1 {}
pub struct AF2 {}
pub struct AF3 {}
pub struct AF4 {}

/// Trait of types which represent alternative functions. The trait itself serves no purpose but to
/// restrict which types can be used to represent the selected alternative function of a pin.
pub unsafe trait AlternativeFunction {}

unsafe impl AlternativeFunction for AF0 {}
unsafe impl AlternativeFunction for AF4 {}
unsafe impl AlternativeFunction for AF1 {}
unsafe impl AlternativeFunction for AF2 {}
unsafe impl AlternativeFunction for AF3 {}
