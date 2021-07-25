//#![deny(unsafe_code, warnings, clippy::all)]
#![no_main]
#![no_std]

use panic_abort as _;
use stm32f4xx_hal as hal;

use crate::display::Display;
use crate::hal::{i2c::I2c, prelude::*, stm32};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use ds323x::{NaiveDate, NaiveDateTime};
use ssd1306::I2CDisplayInterface;

mod display;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    // LED on PC13
    let mut led = gpioc.pc13.into_push_pull_output();
    led.set_high().unwrap();

    // I2C1, SSD1306 display
    // PB6, SCL1
    // PB7, SDA1
    let disp_scl = gpiob.pb6.into_alternate_af4_open_drain();
    let disp_sda = gpiob.pb7.into_alternate_af4_open_drain();
    let disp_i2c = I2c::new(dp.I2C1, (disp_scl, disp_sda), 400.khz(), clocks);
    let disp_iface = I2CDisplayInterface::new(disp_i2c);
    let mut display = Display::new(disp_iface).unwrap();

    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    let dt: NaiveDateTime = NaiveDate::from_ymd(2021, 7, 25).and_hms(5, 42, 11);

    // TODO - timer, switch display modes every ~5sec
    let mut temp: f32 = 0.0;
    let mut humid: f32 = 0.0;
    loop {
        led.toggle().unwrap();

        temp += 10.1;
        humid += 5.1;
        display.draw_sensor_readings(temp, humid).unwrap();
        delay.delay_ms(3000_u32);

        display.draw_date(&dt.date()).unwrap();
        delay.delay_ms(3000_u32);

        display.draw_time(&dt.time()).unwrap();
        delay.delay_ms(3000_u32);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
