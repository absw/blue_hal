//! SPI driver implementation for the MAX32631.
//!
//! Currently, only 4-wire SPI over the SPIM1 peripheral is supported.

use crate::{
    drivers::max3263::gpio::*,
    hal::spi,
    max32pac::{CLKMAN, SPIM1},
};

/// A pin which may be used as an SPIM1 clock pin.
pub unsafe trait ClkPin {}

/// A pin which may be used as an SPIM1 chip select pin.
pub unsafe trait CsPin {}

/// A pin which may be used as an SPIM1 IO0 (MOSI) pin.
pub unsafe trait Io0Pin {}

/// A pin which may be used as an SPIM1 IO1 (MISO) pin.
pub unsafe trait Io1Pin {}

// Trait implementations for SPI pins.
unsafe impl ClkPin for P10<AF0> {}
unsafe impl CsPin for P13<AF0> {}
unsafe impl Io0Pin for P11<AF0> {}
unsafe impl Io1Pin for P12<AF0> {}

/// The SPI driver.
pub struct Spi<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> {
    spim: SPIM1,
    clock: CK,
    chip_select: CS,
    mosi: IO0,
    miso: IO1,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BadParameter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ClkmanScale(u32);

impl ClkmanScale {
    const MAX_VALUE: u32 = 0x0A;

    pub const fn disabled() -> Self {
        Self(0x00)
    }

    pub const fn auto() -> Self {
        Self(Self::MAX_VALUE)
    }
}

impl core::ops::Sub<u32> for ClkmanScale {
    type Output = Self;

    fn sub(self, right: u32) -> ClkmanScale {
        let value = self.0 - right;
        assert!(value <= Self::MAX_VALUE);
        Self(value)
    }
}

impl core::ops::Add<u32> for ClkmanScale {
    type Output = Self;

    fn add(self, right: u32) -> ClkmanScale {
        let value = self.0 + right;
        assert!(value <= Self::MAX_VALUE);
        Self(value)
    }
}

impl core::ops::AddAssign<u32> for ClkmanScale {
    fn add_assign(&mut self, right: u32) {
        *self = *self + right;
    }
}

impl<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> Spi<CK, CS, IO0, IO1> {
    pub fn new(
        spim: SPIM1,
        _clkman: &mut CLKMAN,
        clock: CK,
        chip_select: CS,
        mosi: IO0,
        miso: IO1,
    ) -> Result<Self, Error> {
        struct SystemConfig<'a> {
            clk_scale: ClkmanScale,
            io_cfg: &'a (),
        }

        struct SpimConfig {
            baud: u32,
            mode: u32,
        }

        let system_config = SystemConfig {
            clk_scale: ClkmanScale::disabled(), // TODO
            io_cfg: &(),
        };

        let config = SpimConfig {
            baud: 15200,
            mode: 3,
        };

        let clock_scale = if system_config.clk_scale == ClkmanScale::auto() {
            if config.baud > (Self::system_core_clock() / 2) {
                return Err(Error::BadParameter);
            }

            let mut value = ClkmanScale::disabled();

            loop {
                let max_baud = (Self::system_core_clock() >> value.0) / 2;

                if (config.baud == max_baud) || (value == ClkmanScale::auto()) {
                    break;
                }

                value += 1;
            }

            value
        } else {
            let max_baud = (Self::system_core_clock() >> (system_config.clk_scale - 1).0) / 2;

            if config.baud > max_baud {
                return Err(Error::BadParameter);
            } else {
                system_config.clk_scale
            }
        };

        Self::clkman_set_clk_scale(clock_scale);

        Self::ioman_config(&system_config.io_cfg)?;

        let spim_clk = Self::sys_get_freq(clock_scale);

        if (spim_clk == 0)
            || ((spim_clk == Self::system_core_clock()) && ((spim_clk / 2) < config.baud))
        {
            return Err(Error::BadParameter);
        }

        // // Initialize state pointers
        // states[spim_num].req = NULL;
        // states[spim_num].head_rem = 0;
        // NOTE: These may be referring to partially stored transfers?

        // Drain FIFOs, then re-enable everything.

        // SPIM1.GEN_CTRL = 0
        // SPIM1.GEN_CTRL = SPI_MSTR_EN | TX_FIFO_EN | RX_FIFO_EN | ENABLE_SCK_FB_MODE

        // Set SPIM mode and page size.
        // SPIM1.MSTR_CFG = ((cfg.mode << SPI_MODE_POS) & SPI_MODE) | PAGE_32B | (0x2 << ACT_DELAY_POS)

        // Set SSEL polarity.
        // SPIM1.SS_SR_POLARITY = (cfg.ssel_pol << SS_POLARITY_POS) & SS_POLARITY

        // Disable feedback clocks in modes 1 and 2.
        if config.mode == 1 || config.mode == 2 {
            // SPIM1.GEN_CTRL &= ~ENABLE_SCK_FB_MODE
            // SPIM1.MSTR_CFG |= 1 << SDIO_SAMPLE_POINTS_POS
        }

        // Calculate the high/low clock settings.
        let clocks = if (spim_clk / 2) > config.baud {
            // SPIM1.GEN_CTRL &= ~ENABLE_SCK_FB_MODE

            let value = spim_clk / (2 * config.baud);

            if (value == 0) || (value > 0x10) {
                return Err(Error::BadParameter);
            } else if value == 0x10 {
                0
            } else {
                value
            }
        } else {
            1
        };

        // SPIM1.MSTR_CFG |= ((clocks << SCK_HI_CLK_POS) & SCK_HI_CLK) | ((clocks << SCK_LO_CLK_POS) & SCK_LO_CLK)

        Ok(Self {
            spim,
            clock,
            chip_select,
            mosi,
            miso,
        })
    }

    fn system_core_clock() -> u32 {
        todo!()
    }

    fn clkman_set_clk_scale(_: ClkmanScale) {
        todo!()
    }

    fn ioman_config(_: &()) -> Result<(), Error> {
        todo!()
    }

    fn sys_get_freq(clock_scale: ClkmanScale) -> u32 {
        if clock_scale == ClkmanScale::disabled() {
            0
        } else {
            let divisor: u32 = 1 << (clock_scale.0 - 1);
            Self::system_core_clock() / divisor
        }
    }
}

impl<CK: ClkPin, CS: CsPin, IO0: Io0Pin, IO1: Io1Pin> spi::FullDuplex<u8>
    for Spi<CK, CS, IO0, IO1>
{
    type Error = Error;

    fn transmit(&mut self, _word: Option<u8>) -> nb::Result<(), Self::Error> {
        todo!()
    }

    fn receive(&mut self) -> nb::Result<u8, Self::Error> {
        todo!()
    }
}

// trait SpimRegister {
//     fn get_clock_scale(&self, clkman: &CLKMAN) -> u8;
//     fn set_clock_scake(&mut self, clkman: &mut CLKMAN, value: u8);
//     fn disable_feedback_mode(&mut self);
// }

// impl SpimRegister for SPIM0 {
//     fn get_clock_scale(&self, clkman: &CLKMAN) -> u8 {
//         clkman.sys_clk_ctrl_11_spi0.read().spi0_clk_scale().bits()
//     }

//     fn set_clock_scake(&mut self, clkman: &mut CLKMAN, value: u8) {
//         clkman.sys_clk_ctrl_11_spi0.write(|w| w.spi0_clk_scale().bits(value))
//     }

//     fn disable_feedback_mode(&mut self) {
//         self.gen_ctrl.modify(|_, w| w.enable_sck_fb_mode().clear())
//     }

//     fn reset(&mut self);
// }

// impl SpimRegister for SPIM1 {
//     fn get_clock_scale(&self, clkman: &CLKMAN) -> u8 {
//         clkman.sys_clk_ctrl_12_spi1.read().spi1_clk_scale().bits()
//     }

//     fn set_clock_scake(&mut self, clkman: &mut CLKMAN, value: u8) {
//         clkman.sys_clk_ctrl_12_spi1.write(|w| w.spi1_clk_scale().bits(value))
//     }

//     fn disable_feedback_mode(&mut self) {
//         self.gen_ctrl.modify(|_, w| w.enable_sck_fb_mode().clear())
//     }

//     fn reset(&mut self) {
//         self.gen_ctrl <= 0;
//         // Write register SPIM.GEN_CTRL: 0
//         // Write register SPIM.GEN_CTRL:
//         //     MXC_F_SPIM_GEN_CTRL_SPI_MSTR_EN |
//         //     MXC_F_SPIM_GEN_CTRL_TX_FIFO_EN |
//         //     MXC_F_SPIM_GEN_CTRL_RX_FIFO_EN |
//         //     MXC_F_SPIM_GEN_CTRL_ENABLE_SCK_FB_MODE;
//     }
// }

// impl SpimRegister for SPIM2 {
//     fn get_clock_scale(&self, clkman: &CLKMAN) -> u8 {
//         clkman.sys_clk_ctrl_13_spi2.read().spi2_clk_scale().bits()
//     }

//     fn set_clock_scake(&mut self, clkman: &mut CLKMAN, value: u8) {
//         clkman.sys_clk_ctrl_13_spi2.write(|w| w.spi2_clk_scale().bits(value))
//     }

//     fn disable_feedback_mode(&mut self) {
//         self.gen_ctrl.modify(|_, w| w.enable_sck_fb_mode().clear())
//     }
// }
