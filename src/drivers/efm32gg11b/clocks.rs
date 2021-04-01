use efm32gg11b::{CMU, MSC};
use time::Hertz;
use crate::hal::time::{self, MegaHertz};

const HFRCO_STARTUP_FREQUENCY: time::MegaHertz = time::MegaHertz(19);

pub struct Clocks {
    cmu: CMU,
}

pub enum Clock {
    HfClk,
    HfPerClk,
}

enum HfClkClockSource {
    Hfrco = 1,
    Hfxo = 2,
    Lfrco = 3,
    Lfxo = 4,
    Hfrcodiv2 = 5,
    Ushfrco = 6,
    Clkin0 = 7,
}

impl Clocks {
    /// Currently hardcoded at 72mhz HFRCO
    pub fn new(cmu: CMU, msc: &mut MSC) -> Self {
        let mut clocks = Self {
            cmu,
        };

        let frequency = MegaHertz(72);
        let device_information_page = 0x0FE081B0;
        let calibration_information_offset = match frequency {
            MegaHertz(72) => 0x0C0,
            _ => todo!("Calibration for frequencies below 72mhz unimplemented"),
        };

        let calibration_register = (device_information_page + calibration_information_offset) as *const u32;

        // Safety: We need to access a raw memory location defined in the `efm32gg11b` reference
        // manual sections 4.6 and 4.7. This section is always defined, and user readable.
        let calibration = unsafe { *calibration_register };

        Self::set_flash_wait_states(msc, frequency);
        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe {
            clocks.cmu.hfperpresc.write(|w| w.bits(3));
            clocks.cmu.hfperprescb.write(|w| w.bits(3));
            clocks.cmu.hfperprescc.write(|w| w.bits(3));
        }
        clocks.wait_for_hfrco_sync();

        unsafe { clocks.cmu.hfrcoctrl.write(|w| w.bits(calibration));}
        clocks.set_prescalers(frequency);
        clocks
    }

    fn set_prescalers(&mut self, frequency: MegaHertz) {
        assert_eq!(frequency, MegaHertz(72));
        let hertz: Hertz = frequency.into();

        let mut divisor = (hertz.0 + 50_000_000u32 - 1u32) / 50_000_000u32;
        if divisor > 0 { divisor -= 1 };

        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe { self.cmu.hfperpresc.write(|w| w.bits(divisor));}
        unsafe { self.cmu.hfperprescc.write(|w| w.bits(divisor));}

        let mut divisor = (hertz.0 + 72_000_000u32 - 1u32) / 72_000_000u32;
        if divisor > 0 { divisor -= 1 };
        unsafe { self.cmu.hfperprescb.write(|w| w.bits(divisor));}
    }

    fn wait_for_hfrco_sync(&self) { while self.cmu.syncbusy.read().hfrcobsy().bit_is_set() {} }

    fn set_flash_wait_states(msc: &mut MSC, frequency: MegaHertz) {
        const MSC_UNLOCK_CODE: u32 = 0x1B71;
        assert_eq!(frequency, MegaHertz(72));
        // Safety: Unsafe access here is required only to write
        // multiple bits at once to the same register. We must ensure
        // that we write bits that leave the peripheral in a known and
        // correct state.
        unsafe {
            msc.lock.write(|w| w.bits(MSC_UNLOCK_CODE));
            msc.ctrl.modify(|_, w| w.waitmode().set_bit());
            msc.readctrl.modify(|_, w| w.mode().ws3());
            msc.lock.write(|w| w.bits(0));
        }
    }

    fn get_hf_clock_source(&self) -> HfClkClockSource {
        let status = self.cmu.hfclkstatus.read();

        match status.selected() {
            s if s.is_hfrco() => HfClkClockSource::Hfrco,
            s if s.is_hfxo() => HfClkClockSource::Hfxo,
            s if s.is_lfxo() => HfClkClockSource::Lfxo,
            s if s.is_lfrco() => HfClkClockSource::Lfrco,
            s if s.is_hfrcodiv2() => HfClkClockSource::Hfrcodiv2,
            s if s.is_ushfrco() => HfClkClockSource::Ushfrco,
            s if s.is_clkin0() => HfClkClockSource::Clkin0,
            _ => panic!(),
        }

    }

    pub fn get_frequency_hfclk(&self) -> time::Hertz {
        if !matches!(self.get_hf_clock_source(), HfClkClockSource::Hfrco) {
            unimplemented!("Only HfClk<-Hfrco sourced clock calculations implemented");
        }

        match self.get_hf_clock_source() {
            HfClkClockSource::Hfrco => {
                let scale = 1 + self.cmu.hfpresc.read().bits();
                let Hertz(value) = HFRCO_STARTUP_FREQUENCY.into();
                Hertz(value / scale)
            },
            _ => unimplemented!()
        }
    }

    fn _get_frequency_hfperclk(&self) -> time::Hertz {
        let hfclk_frequency = self.get_frequency_hfclk();
        let scale = 1 + self.cmu.hfperpresc.read().bits();

        Hertz(hfclk_frequency.0 / scale)
    }

    fn _get_frequency(&self, clock: Clock) -> time::Hertz {
        match clock {
            Clock::HfClk => self.get_frequency_hfclk(),
            Clock::HfPerClk => self._get_frequency_hfperclk(),
        }
    }
}
