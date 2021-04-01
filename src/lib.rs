//! # Blue Hal
//!
//! Rusty drivers and HAL developed in-house at Bluefruit
#![feature(never_type)]
#![feature(bool_to_option)]
#![feature(associated_type_bounds)]
#![feature(min_specialization)]
#![feature(stmt_expr_attributes)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(target_arch = "arm", no_std)]

#[cfg(feature = "stm32f407")]
pub use stm32f4::stm32f407 as stm32pac;
#[cfg(feature = "stm32f412")]
pub use stm32f4::stm32f412 as stm32pac;
#[cfg(feature = "stm32f429")]
pub use stm32f4::stm32f429 as stm32pac;
#[cfg(feature = "stm32f469")]
pub use stm32f4::stm32f469 as stm32pac;

#[cfg(feature = "efm32gg11b_any")]
pub use efm32gg11b as efm32pac;

#[macro_use]
pub mod utilities {
    pub mod bitwise;
    pub mod iterator;
    pub mod memory;
    pub mod guard;
    pub mod buffer;
    pub mod xmodem;
    pub mod safety;
    mod macros;
}

pub use paste;
pub mod hal;
pub mod drivers;
