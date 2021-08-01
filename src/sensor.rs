use crate::hal::hal::blocking::delay::DelayMs;
use crate::hal::hal::blocking::i2c::{Read, Write};
use crate::system_clock::SystemClock;
use bme680::{
    Bme680, Error, FieldData, FieldDataCondition, I2CAddress, IIRFilterSize, OversamplingSetting,
    PowerMode, SettingsBuilder,
};
use embedded_time::{duration::Seconds, Instant};

const POLLING_INTERVAL: Seconds = Seconds(15_u32);

// stm32f4xx Timer only impls Delay<u16>, bme680 wants Delay<u8>
pub struct DelayWrapper<D: DelayMs<u16>> {
    pub delay: D,
}

impl<D: DelayMs<u16>> DelayMs<u8> for DelayWrapper<D> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay.delay_ms(ms as _);
    }
}

pub struct Sensor<I2C, D> {
    drv: Bme680<I2C, D>,
    last_polled: Instant<SystemClock>,
}

impl<I2C, D> Sensor<I2C, D>
where
    I2C: Read + Write,
    D: DelayMs<u8>,
{
    pub fn new(
        i2c: I2C,
        now: &Instant<SystemClock>,
        delay: &mut D,
    ) -> Result<Self, Error<<I2C as Read>::Error, <I2C as Write>::Error>> {
        let mut drv = Bme680::init(i2c, delay, I2CAddress::Secondary)?;
        let settings = SettingsBuilder::new()
            .with_humidity_oversampling(OversamplingSetting::OS2x)
            .with_pressure_oversampling(OversamplingSetting::OS4x)
            .with_temperature_oversampling(OversamplingSetting::OS8x)
            .with_temperature_filter(IIRFilterSize::Size3)
            .with_gas_measurement(core::time::Duration::from_millis(1500), 320, 25)
            .with_temperature_offset(-0.56)
            .with_run_gas(true)
            .build();
        drv.set_sensor_settings(delay, settings)?;
        drv.set_sensor_mode(delay, PowerMode::ForcedMode)?;
        Ok(Sensor {
            drv,
            last_polled: *now,
        })
    }

    pub fn poll(
        &mut self,
        now: &Instant<SystemClock>,
        delay: &mut D,
    ) -> Result<Option<FieldData>, Error<<I2C as Read>::Error, <I2C as Write>::Error>> {
        if let Some(dur) = now.checked_duration_since(&self.last_polled) {
            if dur >= POLLING_INTERVAL.into() {
                self.last_polled = *now;
                self.drv.set_sensor_mode(delay, PowerMode::ForcedMode)?;
                let (data, state) = self.drv.get_sensor_data(delay)?;
                if state == FieldDataCondition::NewData {
                    return Ok(Some(data));
                }
            }
        }
        Ok(None)
    }
}
