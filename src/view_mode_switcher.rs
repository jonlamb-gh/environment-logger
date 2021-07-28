use crate::system_clock::SystemClock;
use embedded_time::{duration::Seconds, Instant};

const VIEW_DURATION: Seconds = Seconds(5_u32);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ViewMode {
    Time,
    Date,
    SensorReadings,
    SystemStatus,
}

impl ViewMode {
    fn next(self) -> Self {
        match self {
            ViewMode::Time => ViewMode::Date,
            ViewMode::Date => ViewMode::SensorReadings,
            ViewMode::SensorReadings => ViewMode::SystemStatus,
            ViewMode::SystemStatus => ViewMode::Time,
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Time
    }
}

pub struct ViewModeSwitcher {
    mode: ViewMode,
    last_transition: Instant<SystemClock>,
}

impl ViewModeSwitcher {
    pub fn new(now: Instant<SystemClock>) -> Self {
        ViewModeSwitcher {
            mode: Default::default(),
            last_transition: now,
        }
    }

    pub fn set_mode(&mut self, mode: ViewMode, now: &Instant<SystemClock>) {
        self.mode = mode;
        self.last_transition = *now;
    }

    pub fn mode(&mut self, now: &Instant<SystemClock>) -> ViewMode {
        if let Some(dur) = now.checked_duration_since(&self.last_transition) {
            if dur >= VIEW_DURATION.into() {
                self.last_transition = *now;
                self.mode = self.mode.next();
            }
        }
        self.mode
    }
}
