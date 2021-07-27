use crate::hal::{rcc::Clocks, stm32::SYST};
use core::sync::atomic::{AtomicU32, Ordering::SeqCst};
use cortex_m::peripheral::syst::SystClkSource;
use embedded_time::{clock, fraction::Fraction, Clock, Instant};

/// 32-bit second clock
#[derive(Debug)]
pub struct SystemClock(AtomicU32);

unsafe impl Send for SystemClock {}
unsafe impl Sync for SystemClock {}

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
