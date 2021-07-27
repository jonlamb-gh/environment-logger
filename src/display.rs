use core::fmt::Write;
use display_interface::DisplayError;
use ds323x::{Datelike, NaiveDate, NaiveTime, Timelike};
use embedded_graphics::mono_font::{ascii as fonts, MonoFont, MonoTextStyleBuilder};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use heapless::String;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

type DispSize = DisplaySize128x32;

//const SENSOR_READING_FONT: MonoFont<'_> = fonts::FONT_9X15_BOLD;
//const DATETIME_FONT: MonoFont<'_> = fonts::FONT_10X20;
const SENSOR_READING_FONT: MonoFont<'_> = profont::PROFONT_14_POINT;
const DATETIME_FONT: MonoFont<'_> = profont::PROFONT_24_POINT;

const LINE_BUF_CAP: usize = 64;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// TODO - (maybe optional) error/status view mode, or just global struct details
// along the bottom of the display
// uptime, io/fs error, fs record count, alarm on
//
// maybe combine Time and Date, make date a smaller font
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ViewMode {
    Time,
    Date,
    SensorReadings,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Time
    }
}

pub enum View<'a> {
    Time { time: &'a NaiveTime },
    Date { date: &'a NaiveDate },
    SensorReadings { data: &'a bme680::FieldData },
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

    pub fn draw_sensor_readings(&mut self, temp: f32, humid: f32) -> Result<(), DisplayError> {
        let temp = temp.clamp(0.0, 99.0);
        let humid = humid.clamp(0.0, 99.0);

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

    pub fn draw_date(&mut self, date: &NaiveDate) -> Result<(), DisplayError> {
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

    pub fn draw_time(&mut self, time: &NaiveTime) -> Result<(), DisplayError> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&DATETIME_FONT)
            .text_color(BinaryColor::On)
            .build();
        let pos_y = (DisplaySize128x32::HEIGHT / 2) as i32;
        let (is_pm, hour) = time.hour12();

        self.drv.clear();

        // TODO - overflows the screen a little with the current font size
        self.line_buf.clear();
        write!(
            &mut self.line_buf,
            "{:02}:{:02} {}",
            hour,
            time.minute(),
            if is_pm { "PM" } else { "AM" },
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
}
