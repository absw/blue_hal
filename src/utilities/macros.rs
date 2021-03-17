//! General purpose convenience macros.
#![macro_use]

/// Define and export a specific port module (transparently pulls
/// its namespace to the current one).
///
/// Used mostly to conveniently fit the module declaration and reexport
/// under a single configuration flag.
///
/// # Example
/// ```ignore
/// #[cfg(feature = "stm32_any")]
/// port!(stm32);
/// // Expands into:
/// pub mod stm32;
/// pub use self::stm32::*;
///
/// #[cfg(feature = "stm32_any")]
/// port!(stm32::flash as mcu_flash);
/// // Expands into:
/// pub mod stm32 { pub mod flash };
/// pub use self::stm32::flash as mcu_flash;
/// ```
#[macro_export]
macro_rules! port {
    ($mod:ident) => {
        pub mod $mod;
        pub use self::$mod::*;
    };
    ($mod:ident as $name:ident) => {
        pub mod $mod;
        pub use self::$mod as $name;
    };
    ($outer:ident::$inner:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner::*;
    };
    ($outer:ident::$inner:ident as $name:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner as $name;
    };
    ($outer:ident: [$($inner:ident,)+]) => {
        pub mod $outer {
        $(
            pub mod $inner;
        )+
        }
        $(
            pub use self::$outer::$inner;
        )+
    };
    ($outer:ident: [$($inner:ident as $name:ident)+,]) => {
        pub mod $outer {
        $(
            pub mod $inner;
        )+
        }
        $(
            pub use self::$outer::$inner as $name;
        )+
    };
}

#[macro_export]
macro_rules! structs {
    ($($struct:ident)+) => {
        $(struct $struct; )+
    };
    ($($struct:ident)+, implementing $trait:ident) => {
        pub trait $trait {}
        $(struct $struct; impl $trait for $struct {})+
    };
}

/// Calls an inner macro by expanding the outer arguments in matrix form.
///
/// # Example:
///
/// ```ignore
/// matrix!(my_macro, [a b c] [1 2 3]);
/// ```
///
/// Is equivalent to:
/// ```ignore
/// my_macro!(a 1 a 2 a 3 b 1 b 2 b 3 c 1 c 2 c 3);
/// ```
///
#[macro_export]
macro_rules! matrix {
    ( $inner_macro:ident [$($n:tt)+] $ms:tt) => ( matrix! { $inner_macro $($n $ms)* });
    ( $inner_macro:ident $( $n:tt [$($m:tt)*] )* ) =>
        ( $inner_macro! { $( $( $n $m )* )* } );
}


/// Allowed pins for a particular function
#[allow(unused)]
macro_rules! allowed {
    ($($function:ty: [$($pin:ty)+])*) => { $($(impl $function for $pin {})+)* }
;}
