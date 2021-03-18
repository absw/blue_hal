use efm32gg11b::CMU;
use time::Hertz;
use crate::hal::time;

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

    fn get_frequency_hfclk(&self) -> time::Hertz {
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

    fn get_frequency_hfperclk(&self) -> time::Hertz {
        let hfclk_frequency = self.get_frequency_hfclk();
        let scale = 1 + self.cmu.hfperpresc.read().bits();

        Hertz(hfclk_frequency.0 / scale)
    }

    fn get_frequency(&self, clock: Clock) -> time::Hertz {
        match clock {
            Clock::HfClk => self.get_frequency_hfclk(),
            Clock::HfPerClk => self.get_frequency_hfperclk(),
        }
    }
}
