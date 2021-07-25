use crate::hal::hal::blocking::i2c::{Write, WriteRead};
use ds323x::{ic::DS3231, interface::I2cInterface, Ds323x, Error, NaiveDateTime, Rtcc};

pub struct Rtc<I2C> {
    drv: Ds323x<I2cInterface<I2C>, DS3231>,
}

impl<I2C, E> Rtc<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, Error<E, ()>> {
        let mut drv = Ds323x::new_ds3231(i2c);
        drv.disable()?;
        drv.disable_square_wave()?;
        drv.disable_32khz_output()?;
        drv.disable_alarm1_interrupts()?;
        drv.disable_alarm2_interrupts()?;
        drv.enable()?;
        Ok(Rtc { drv })
    }

    pub fn get_datetime(&mut self) -> Result<NaiveDateTime, Error<E, ()>> {
        self.drv.get_datetime()
    }
}
