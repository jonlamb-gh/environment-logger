use crate::hal::{rcc::Clocks, stm32::SYST};
use core::sync::atomic::{AtomicU32, Ordering::SeqCst};
use cortex_m::peripheral::syst::SystClkSource;
use ds323x::{Datelike, Timelike};
use embedded_time::{clock, fraction::Fraction, Clock, Instant};

/// 32-bit second clock
#[derive(Debug)]
#[repr(transparent)]
pub struct SystemClock(AtomicU32);

impl SystemClock {
    pub const fn new() -> Self {
        SystemClock(AtomicU32::new(0))
    }

    pub fn enable_systick_interrupt(&self, mut syst: SYST, clocks: &Clocks) {
        log::debug!("Enable SystemClock hclk freq {} Hz", clocks.hclk().0);

        // Generate an interrupt once a second, HCLK/8
        syst.set_clock_source(SystClkSource::External);
        syst.set_reload(clocks.hclk().0 / 8);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        // So the SYST can't be stopped or reset
        drop(syst);
    }

    pub fn inc_from_interrupt(&self) {
        self.0.fetch_add(1, SeqCst);
    }

    pub fn get_raw(&self) -> u32 {
        self.0.load(SeqCst)
    }

    pub fn now(&self) -> Instant<Self> {
        Instant::new(self.get_raw())
    }
}

impl Clock for SystemClock {
    type T = u32;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1);

    fn try_now(&self) -> Result<Instant<Self>, clock::Error> {
        Ok(Instant::new(self.get_raw()))
    }
}

// NOTE: only to provide a dummy embedded_sdmmc::TimeSource for the FAT32 fs
// don't care about fs datetime so it's just using the relative system time
// seconds
pub struct SystemClockRef<'a> {
    pub base_datetime: ds323x::NaiveDateTime,
    pub sys_clock: &'a SystemClock,
}

impl<'a> embedded_sdmmc::TimeSource for SystemClockRef<'a> {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        let now_sec = self.sys_clock.get_raw();
        let now_dur = core::time::Duration::from_secs(now_sec as _);
        let time_since_boot = chrono::Duration::from_std(now_dur).unwrap();
        let dt = self.base_datetime + time_since_boot;
        let date = dt.date();
        let time = dt.time();
        embedded_sdmmc::Timestamp {
            year_since_1970: (date.year() - 1970) as _,
            zero_indexed_month: date.month0() as _,
            zero_indexed_day: date.day0() as _,
            hours: time.hour() as _,
            minutes: time.minute() as _,
            seconds: time.second() as _,
        }
    }
}
