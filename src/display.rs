use crate::system_status::SystemStatus;
use crate::util;
use core::fmt::Write;
use display_interface::DisplayError;
use ds323x::{Datelike, NaiveDate, NaiveTime, Timelike};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyleBuilder};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use heapless::String;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

type DispSize = DisplaySize128x64;

const DATE_FONT: MonoFont<'_> = profont::PROFONT_24_POINT;
const TIME_FONT: MonoFont<'_> = profont::PROFONT_24_POINT;
const SENSOR_READING_FONT: MonoFont<'_> = profont::PROFONT_14_POINT;
const SYS_STATS_FONT: MonoFont<'_> = profont::PROFONT_14_POINT;

const LINE_BUF_CAP: usize = 64;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub enum View<'a> {
    Time { data: &'a NaiveTime },
    Date { data: &'a NaiveDate },
    SensorReadings { data: &'a bme680::FieldData },
    SystemStatus { data: &'a SystemStatus },
}

pub struct Display<DI> {
    drv: Ssd1306<DI, DispSize, BufferedGraphicsMode<DispSize>>,
    brightness: Brightness,
    line_buf: String<LINE_BUF_CAP>,
}

impl<DI> Display<DI>
where
    DI: WriteOnlyDataCommand,
{
    pub fn new(di: DI) -> Result<Self, DisplayError> {
        let brightness = Brightness::BRIGHTEST;
        let mut drv =
            Ssd1306::new(di, DispSize {}, DisplayRotation::Rotate0).into_buffered_graphics_mode();
        drv.init()?;
        drv.set_display_on(true)?;
        drv.clear();
        drv.flush()?;
        drv.set_brightness(brightness)?;
        Ok(Display {
            drv,
            brightness,
            line_buf: String::new(),
        })
    }

    /// 8 PM ..= 8 AM => Brightness::DIMMEST, else Brightness::BRIGHTEST
    pub fn update_brightness(&mut self, time: &NaiveTime) -> Result<(), DisplayError> {
        let hr24 = time.hour();
        let range0 = 20..=23;
        let range1 = 0..=7;
        let brightness = if range0.contains(&hr24) || range1.contains(&hr24) {
            Brightness::DIMMEST
        } else {
            Brightness::BRIGHTEST
        };
        if brightness != self.brightness {
            self.drv.set_brightness(brightness)?;
            self.brightness = brightness;
        }
        Ok(())
    }

    pub fn draw_view(&mut self, view: View) -> Result<(), DisplayError> {
        match view {
            View::Time { data } => self.draw_time(data),
            View::Date { data } => self.draw_date(data),
            View::SensorReadings { data } => self.draw_sensor_readings(data),
            View::SystemStatus { data } => self.draw_system_status(data),
        }
    }

    fn draw_time(&mut self, time: &NaiveTime) -> Result<(), DisplayError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&TIME_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y = (DispSize::HEIGHT / 2) as i32;
        let (_is_pm, hour) = time.hour12();

        self.drv.clear();

        self.line_buf.clear();
        write!(&mut self.line_buf, "{:02}:{:02}", hour, time.minute(),)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(24, pos_y),
            text_style,
            Baseline::Middle,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_date(&mut self, date: &NaiveDate) -> Result<(), DisplayError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&DATE_FONT)
            .text_color(BinaryColor::On)
            .build();
        let mid = (DispSize::HEIGHT / 2) as i32;

        self.drv.clear();

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            " {:02} {}",
            date.day(),
            MONTHS[date.month0().clamp(0, 11) as usize],
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(&mut self.line_buf, "  {}", date.year())
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, mid),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_sensor_readings(&mut self, data: &bme680::FieldData) -> Result<(), DisplayError> {
        let temp = util::celsius_to_fahrenheit(data.temperature_celsius()).clamp(0.0, 99.0);
        let humid = data.humidity_percent().clamp(0.0, 99.0);
        let pressure = data.pressure_hpa();
        let gas = if data.gas_valid() {
            data.gas_resistance_ohm()
        } else {
            0
        };

        let dh = (DispSize::HEIGHT / 4) as i32;
        let text_style = MonoTextStyleBuilder::new()
            .font(&SENSOR_READING_FONT)
            .text_color(BinaryColor::On)
            .build();

        self.drv.clear();

        self.line_buf.clear();
        write!(&mut self.line_buf, "Temp     {:02.0} F", temp)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(&mut self.line_buf, "Humidity {:02.0} %", humid)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(&mut self.line_buf, "Pressure {:04.0}", pressure)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, 2 * dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(&mut self.line_buf, "Gas    {:06.0}", gas)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, 3 * dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_system_status(&mut self, data: &SystemStatus) -> Result<(), DisplayError> {
        let dh = (DispSize::HEIGHT / 4) as i32;
        let text_style = MonoTextStyleBuilder::new()
            .font(&SYS_STATS_FONT)
            .text_color(BinaryColor::On)
            .build();

        self.drv.clear();

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            "UT {}",
            data.uptime_sec.clamp(0, 999999999)
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            "CNT {}",
            data.record_count.clamp(0, 99999999)
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            "ALRM {}  CON {}",
            data.alarm,
            util::DisplayBool::from(data.storage_connected)
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, 2 * dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            "FULL {}  ERR {}",
            util::DisplayBool::from(data.storage_full),
            util::DisplayBool::from(data.storage_error)
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, 3 * dh),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }
}
