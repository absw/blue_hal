//! Driver implementations for all supported platforms. They offer
//! a safe API, and are
//! [typestate](https://rust-embedded.github.io/book/static-guarantees/typestate-programming.html)
//! based whenever possible.
#![macro_use]

/// Drivers for the stm32f4 family of microcontrollers.
#[cfg(feature = "stm32f4_any")]
#[macro_use]
pub mod stm32f4 {
    pub mod flash;
    pub mod gpio;
    #[cfg(any(feature = "stm32f412", feature = "stm32f446"))]
    pub mod qspi;
    pub mod rcc;
    pub mod serial;
    pub mod spi;
    pub mod systick;
}

#[cfg(feature = "efm32gg11b_any")]
#[macro_use]
pub mod efm32gg11b {
    #[macro_use]
    pub mod gpio;
    #[macro_use]
    pub mod serial;
    pub mod flash;
    pub mod clocks;
}

#[cfg(feature = "max3263_any")]
#[macro_use]
pub mod max3263 {
    pub mod flash;
}

#[cfg(feature = "max32631")]
pub mod is25lp128f;

pub mod led;

/// Drivers for the Micron manufacturer (e.g. external flash).
#[cfg(feature = "stm32f412")]
pub mod micron {
    /// N25Q128A external flash chip
    pub mod n25q128a_flash;
}
