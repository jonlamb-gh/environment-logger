use crate::hal::hal::PwmPin;
use crate::util;
use bme680::FieldData;
use core::fmt;
use embedded_time::duration::Minutes;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AlarmStatus {
    /// Alarm is not activing monitoring
    NotMonitoring,
    /// Alarm is on
    On,
    /// Alarm is off
    Off,
}

impl Default for AlarmStatus {
    fn default() -> Self {
        AlarmStatus::Off
    }
}

impl fmt::Display for AlarmStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlarmStatus::NotMonitoring => f.write_str("X"),
            AlarmStatus::On => f.write_str("Y"),
            AlarmStatus::Off => f.write_str("N"),
        }
    }
}

pub struct Alarm<PWM> {
    pwm: PWM,
    monitoring: bool,
    on: bool,
}

impl<Whatev> Alarm<Whatev> {
    /// Wait this much time before monitoring the alarm (if monitoring enabled)
    pub const WARM_UP_DELAY: Minutes = Minutes(10);
}

impl<PWM> Alarm<PWM>
where
    PWM: PwmPin<Duty = u16>,
{
    // [68, 72] °F ([20, 22.2] °C) is ideal
    pub const TEMP_F_MIN: f32 = 66.0;
    pub const TEMP_F_MAX: f32 = 74.0;

    pub fn new(mut pwm: PWM) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.set_duty(max_duty / 2);
        pwm.disable();
        Alarm {
            pwm,
            monitoring: true,
            on: false,
        }
    }

    pub fn monitoring(&self) -> bool {
        self.monitoring
    }

    pub fn set_monitoring(&mut self, monitoring: bool) {
        self.set_on_off(false);
        self.monitoring = monitoring;
    }

    pub fn set_on_off(&mut self, on: bool) {
        if on {
            self.pwm.enable();
        } else {
            self.pwm.disable();
        }
        self.on = on;
    }

    pub fn status(&self) -> AlarmStatus {
        if self.monitoring {
            if self.on {
                AlarmStatus::On
            } else {
                AlarmStatus::Off
            }
        } else {
            AlarmStatus::NotMonitoring
        }
    }

    pub fn check_temperature(&mut self, data: &FieldData) {
        if self.monitoring {
            let temp_f = util::celsius_to_fahrenheit(data.temperature_celsius());
            if !(Self::TEMP_F_MIN..=Self::TEMP_F_MAX).contains(&temp_f) {
                self.set_on_off(true);
            } else {
                self.set_on_off(false)
            }
        } else {
            self.set_on_off(false)
        }
    }
}
