//! Top level error

use crate::hal::i2c;
use display_interface::DisplayError;

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Infallible")]
    Infallible(#[error(source)] core::convert::Infallible),

    #[error(display = "Failed to take stm32::Peripherals")]
    TakeDevicePeripherals,

    #[error(display = "Failed to take cortex_m::Peripherals")]
    TakeCorePeripherals,

    #[error(display = "Display error")]
    Display(#[error(source)] DisplayError),

    #[error(display = "RTC error")]
    Rtc(#[error(source)] ds323x::Error<i2c::Error, ()>),
}
