// TODO
//#![deny(unsafe_code, warnings, clippy::all)]
#![no_main]
#![no_std]

use panic_abort as _;
use stm32f4xx_hal as hal;

use crate::alarm::Alarm;
use crate::atomic_button_state::AtomicButtonState;
use crate::display::{Display, View};
use crate::error::Error;
use crate::hal::{
    delay::Delay,
    gpio::gpioa::PA0,
    gpio::{Edge, Input, PullUp},
    i2c::I2c,
    interrupt,
    prelude::*,
    stm32,
    timer::Timer,
    watchdog::IndependentWatchdog,
};
use crate::rtc::Rtc;
use crate::sensor::{DelayWrapper, Sensor};
use crate::system_clock::SystemClock;
use crate::system_status::SystemStatus;
use crate::view_mode_switcher::{ViewMode, ViewModeSwitcher};
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use ssd1306::I2CDisplayInterface;

mod alarm;
mod atomic_button_state;
mod display;
mod error;
mod file_system;
mod record;
mod rtc;
mod sensor;
mod system_clock;
mod system_status;
mod util;
mod view_mode_switcher;

static SYS_CLOCK: SystemClock = SystemClock::new();
static BUTTON: AtomicButtonState = AtomicButtonState::new();
static BUTTON_GPIO: Mutex<RefCell<Option<PA0<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

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
    let mut dp = stm32::Peripherals::take().ok_or(Error::TakeDevicePeripherals)?;
    let cp = cortex_m::peripheral::Peripherals::take().ok_or(Error::TakeCorePeripherals)?;

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();
    let mut syscfg = dp.SYSCFG.constrain();

    let mut watchdog = IndependentWatchdog::new(dp.IWDG);
    watchdog.start(5000_u32.ms());

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    // On-board LED on PC13
    let mut led = gpioc.pc13.into_push_pull_output();
    led.set_high();

    // On-board button on PA0
    let mut btn = gpioa.pa0.into_pull_up_input();
    btn.make_interrupt_source(&mut syscfg);
    btn.enable_interrupt(&mut dp.EXTI);
    btn.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

    // General purpose delay timer from TIM5
    let delay = Delay::tim5(dp.TIM5, &clocks);
    let mut delay = DelayWrapper { delay };

    // Alarm buzzer, PA10, PWM on T1_CH3
    let pwm_channels = gpioa.pa10.into_alternate();
    let buzzer = Timer::new(dp.TIM1, &clocks).pwm(pwm_channels, 5_u32.khz());
    let mut alarm = Alarm::new(buzzer);

    // Short beep on power up
    led.set_low();
    alarm.set_on_off(true);
    delay.delay_ms(200_u8);
    alarm.set_on_off(false);
    led.set_high();
    alarm.set_monitoring(true);

    // I2C1, SSD1306 display
    // PB6, SCL1
    // PB7, SDA1
    let disp_scl = gpiob.pb6.into_alternate().set_open_drain();
    let disp_sda = gpiob.pb7.into_alternate().set_open_drain();
    let disp_i2c = I2c::new(dp.I2C1, (disp_scl, disp_sda), 400.khz(), clocks);
    let disp_iface = I2CDisplayInterface::new(disp_i2c);
    let mut display = Display::new(disp_iface)?;

    // I2C3, DS3231 RTC
    // PA8, SCL3
    // PB4, SDA3
    let rtc_scl = gpioa.pa8.into_alternate().set_open_drain();
    let rtc_sda = gpiob.pb4.into_alternate().set_open_drain();
    let rtc_i2c = I2c::new(dp.I2C3, (rtc_scl, rtc_sda), 100.khz(), clocks);
    let mut rtc = Rtc::new(rtc_i2c)?;

    // I2C2, BME680 sensor
    // PB10, SCL2
    // PB3, SDA2
    let bme_scl = gpiob.pb10.into_alternate().set_open_drain();
    let bme_sda = gpiob.pb3.into_alternate().set_open_drain();
    let bme_i2c = I2c::new(dp.I2C2, (bme_scl, bme_sda), 100.khz(), clocks);
    let mut sensor = Sensor::new(bme_i2c, &SYS_CLOCK.now(), &mut delay)?;

    SYS_CLOCK.enable_systick_interrupt(cp.SYST, &clocks);
    watchdog.feed();

    let mut status = SystemStatus::default();
    let mut view_mode_switcher = ViewModeSwitcher::new(SYS_CLOCK.now());

    free(|cs| {
        BUTTON_GPIO.borrow(cs).replace(Some(btn));
    });

    // Enable interrupts
    stm32::NVIC::unpend(stm32::Interrupt::EXTI0);
    unsafe {
        stm32::NVIC::unmask(stm32::Interrupt::EXTI0);
    };

    // TODO
    // int/polling for SD detect pin state change

    let mut sensor_data = None;

    loop {
        cortex_m::asm::wfi();
        watchdog.feed();
        led.toggle();

        let dt = rtc.get_datetime()?;

        status.uptime_sec = SYS_CLOCK.get_raw();
        let now = SYS_CLOCK.now();

        if BUTTON.get_and_clear() {
            alarm.set_monitoring(!alarm.monitoring());
            view_mode_switcher.set_mode(ViewMode::SystemStatus, &now);
        }

        // TODO - warm-up period before alarm monitoring
        if let Some(new_sensor_data) = sensor.poll(&now, &mut delay)? {
            alarm.check_temperature_f(util::celsius_to_fahrenheit(
                new_sensor_data.temperature_celsius(),
            ));

            sensor_data.replace(new_sensor_data);
        }

        status.alarm = alarm.status();

        let view_mode = view_mode_switcher.mode(&now);
        match view_mode {
            ViewMode::Time => {
                display.draw_view(View::Time { data: &dt.time() })?;
            }
            ViewMode::Date => {
                display.draw_view(View::Date { data: &dt.date() })?;
            }
            ViewMode::SensorReadings => {
                if let Some(sensor_data) = &sensor_data {
                    display.draw_view(View::SensorReadings { data: &sensor_data })?;
                } else {
                    view_mode_switcher.skip(&now);
                }
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

#[interrupt]
fn EXTI0() {
    free(|cs| {
        let mut btn_ref = BUTTON_GPIO.borrow(cs).borrow_mut();
        if let Some(ref mut btn) = btn_ref.deref_mut() {
            btn.clear_interrupt_pending_bit();
            BUTTON.set();
        }
    });
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
