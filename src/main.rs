// TODO
//#![deny(unsafe_code, warnings, clippy::all)]
#![no_main]
#![no_std]

use panic_abort as _;
use stm32f4xx_hal as hal;

use crate::alarm::Alarm;
use crate::display::{Display, View};
use crate::error::Error;
use crate::hal::{
    delay::Delay, i2c::I2c, prelude::*, stm32, timer::Timer, watchdog::IndependentWatchdog,
};
use crate::system_clock::SystemClock;
use crate::system_status::SystemStatus;
use crate::view_mode_switcher::{ViewMode, ViewModeSwitcher};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use ds323x::{NaiveDate, NaiveDateTime};
use ssd1306::I2CDisplayInterface;

mod alarm;
mod display;
mod error;
mod fs;
mod record;
mod rtc;
mod system_clock;
mod system_status;
mod util;
mod view_mode_switcher;

static SYS_CLOCK: SystemClock = SystemClock::new();

#[entry]
fn main() -> ! {
    if let Err(e) = do_main() {
        log::error!("{}", e);
    }

    // Let the watchdog reset
    loop {
        cortex_m::asm::nop();
    }
}

fn do_main() -> Result<(), Error> {
    let dp = stm32::Peripherals::take().ok_or(Error::TakeDevicePeripherals)?;
    let cp = cortex_m::peripheral::Peripherals::take().ok_or(Error::TakeCorePeripherals)?;

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

    let mut watchdog = IndependentWatchdog::new(dp.IWDG);
    watchdog.start(5000_u32.ms());

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    // LED on PC13
    let mut led = gpioc.pc13.into_push_pull_output();
    led.set_high();

    // General purpose delay timer from TIM5
    let mut delay = Delay::tim5(dp.TIM5, &clocks);

    // Alarm buzzer, PA10, PWM on T1_CH3
    let pwm_channels = gpioa.pa10.into_alternate();
    let buzzer = Timer::new(dp.TIM1, &clocks).pwm(pwm_channels, 5_u32.khz());
    let mut alarm = Alarm::new(buzzer);

    // I2C1, SSD1306 display
    // PB6, SCL1
    // PB7, SDA1
    let disp_scl = gpiob.pb6.into_alternate().set_open_drain();
    let disp_sda = gpiob.pb7.into_alternate().set_open_drain();
    let disp_i2c = I2c::new(dp.I2C1, (disp_scl, disp_sda), 400.khz(), clocks);
    let disp_iface = I2CDisplayInterface::new(disp_i2c);
    let mut display = Display::new(disp_iface)?;

    SYS_CLOCK.enable_systick_interrupt(cp.SYST, &clocks);
    watchdog.feed();

    // Short beep on power up
    led.set_high();
    alarm.enable();
    delay.delay_ms(200_u32);
    alarm.disable();
    led.set_low();

    let mut status = SystemStatus::default();
    let mut view_mode_switcher = ViewModeSwitcher::new(SYS_CLOCK.now());

    // TODO
    // add an optional error/warn display page for EOF/IO error, etc
    // draw a small SD icon on the display if SD card is present
    //   - just do polling on this GPIO for state change

    let dt: NaiveDateTime = NaiveDate::from_ymd(2021, 7, 25).and_hms(5, 42, 11);
    let mut forged_sensor_data = bme680::FieldData {
        status: 0xFF,
        gas_index: 0,
        meas_index: 0,
        temperature: 0,
        pressure: 0,
        humidity: 0,
        gas_resistance: 0,
    };

    loop {
        cortex_m::asm::wfi();
        watchdog.feed();
        led.toggle();

        let now = SYS_CLOCK.now();

        status.uptime_sec = SYS_CLOCK.get_raw();
        status.alarm_on = alarm.is_on();

        let view_mode = view_mode_switcher.mode(&now);

        forged_sensor_data.temperature += 105;
        forged_sensor_data.humidity += 1005;

        match view_mode {
            ViewMode::Time => {
                display.draw_view(View::Time { data: &dt.time() })?;
            }
            ViewMode::Date => {
                display.draw_view(View::Date { data: &dt.date() })?;
            }
            ViewMode::SensorReadings => {
                display.draw_view(View::SensorReadings {
                    data: &forged_sensor_data,
                })?;
            }
            ViewMode::SystemStatus => {
                display.draw_view(View::SystemStatus { data: &status })?;
            }
        }
    }
}

#[exception]
fn SysTick() {
    SYS_CLOCK.inc_from_interrupt();
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
