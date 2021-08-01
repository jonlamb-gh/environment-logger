//! Top level error

use crate::hal::{i2c, serial};
use crate::record;

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Failed to take stm32::Peripherals")]
    TakeDevicePeripherals,

    #[error(display = "Failed to take cortex_m::Peripherals")]
    TakeCorePeripherals,

    #[error(display = "Display error")]
    Display(#[error(source)] display_interface::DisplayError),

    #[error(display = "RTC error")]
    Rtc(#[error(source)] ds323x::Error<i2c::Error, ()>),

    #[error(display = "Sensor error")]
    Sensor(#[error(source)] bme680::Error<i2c::Error, i2c::Error>),

    #[error(display = "Record error")]
    Record(#[error(source)] record::Error),

    #[error(display = "File system error")]
    FileSystem(#[error(source)] embedded_sdmmc::Error<embedded_sdmmc::SdMmcError>),

    #[error(display = "SerialConfig error")]
    SerialConfig(#[error(source)] serial::config::InvalidConfig),

    #[error(display = "Formatting error")]
    Formatting(#[error(source)] core::fmt::Error),
}
