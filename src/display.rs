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

// TODO update to 64 once new hw arrives
type DispSize = DisplaySize128x32;

const SENSOR_READING_FONT: MonoFont<'_> = profont::PROFONT_14_POINT;
const DATETIME_FONT: MonoFont<'_> = profont::PROFONT_24_POINT;

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
    line_buf: String<LINE_BUF_CAP>,
}

impl<DI> Display<DI>
where
    DI: WriteOnlyDataCommand,
{
    pub fn new(di: DI) -> Result<Self, DisplayError> {
        let mut drv =
            Ssd1306::new(di, DispSize {}, DisplayRotation::Rotate0).into_buffered_graphics_mode();
        drv.init()?;
        drv.set_display_on(true)?;
        drv.clear();
        drv.flush()?;
        drv.set_brightness(Brightness::BRIGHTEST)?;
        Ok(Display {
            drv,
            line_buf: String::new(),
        })
    }

    pub fn draw_view<'a>(&mut self, view: View) -> Result<(), DisplayError> {
        match view {
            View::Time { data } => self.draw_time(data),
            View::Date { data } => self.draw_date(data),
            View::SensorReadings { data } => self.draw_sensor_readings(data),
            View::SystemStatus { data } => self.draw_system_status(data),
        }
    }

    fn draw_time(&mut self, time: &NaiveTime) -> Result<(), DisplayError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&DATETIME_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y = (DisplaySize128x32::HEIGHT / 2) as i32;
        let (_is_pm, hour) = time.hour12();

        self.drv.clear();

        self.line_buf.clear();
        write!(&mut self.line_buf, " {:02}:{:02}", hour, time.minute(),)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, pos_y),
            text_style,
            Baseline::Middle,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_date(&mut self, date: &NaiveDate) -> Result<(), DisplayError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&DATETIME_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y = (DisplaySize128x32::HEIGHT / 2) as i32;

        self.drv.clear();

        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            " {:02} {} {}",
            date.day(),
            MONTHS[date.month0().clamp(0, 11) as usize],
            date.year(),
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, pos_y),
            text_style,
            Baseline::Middle,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_sensor_readings(&mut self, data: &bme680::FieldData) -> Result<(), DisplayError> {
        let temp = util::celsius_to_fahrenheit(data.temperature_celsius()).clamp(0.0, 99.0);
        let humid = data.humidity_percent().clamp(0.0, 99.0);

        let text_style = MonoTextStyleBuilder::new()
            .font(&SENSOR_READING_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y =
            DisplaySize128x32::HEIGHT as i32 - SENSOR_READING_FONT.character_size.height as i32;

        self.drv.clear();

        self.line_buf.clear();
        write!(&mut self.line_buf, "Temp   {:02.0} F", temp)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        write!(&mut self.line_buf, "Humid  {:02.0} %", humid)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, pos_y),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }

    fn draw_system_status(&mut self, data: &SystemStatus) -> Result<(), DisplayError> {
        // TODO
        // need to clamp some, like uptime

        let text_style = MonoTextStyleBuilder::new()
            .font(&SENSOR_READING_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y =
            DisplaySize128x32::HEIGHT as i32 - SENSOR_READING_FONT.character_size.height as i32;

        self.drv.clear();

        self.line_buf.clear();
        write!(&mut self.line_buf, "Uptime {}", data.uptime_sec)
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
            "Alarm  {}",
            if data.alarm_on { "ON" } else { "OFF" }
        )
        .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::new(0, pos_y),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.drv.flush()?;

        Ok(())
    }
}
