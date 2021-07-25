// TODO pick fonts
// https://docs.rs/embedded-graphics/0.7.1/embedded_graphics/mono_font/index.html

use core::fmt::Write;
use display_interface::DisplayError;
use embedded_graphics::mono_font::{ascii as fonts, MonoFont, MonoTextStyleBuilder};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use heapless::String;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

type DispSize = DisplaySize128x32;

const BIG_FONT: MonoFont<'_> = fonts::FONT_9X15_BOLD;

const LINE_BUF_CAP: usize = 64;

pub struct Display<DI>
where
    DI: WriteOnlyDataCommand,
{
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

    // TODO don't need this
    /*
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.drv.clear();
        self.drv.flush()?;
        Ok(())
    }
    */

    pub fn draw_sensor_readings(&mut self, temp: f32, humid: f32) -> Result<(), DisplayError> {
        let temp = temp.clamp(0.0, 100.0);
        let humid = humid.clamp(0.0, 100.0);

        self.drv.clear();

        let text_style = MonoTextStyleBuilder::new()
            .font(&BIG_FONT)
            .text_color(BinaryColor::On)
            .build();

        self.line_buf.clear();
        write!(&mut self.line_buf, "Temp   {:.1} F", temp)
            .map_err(|_| DisplayError::InvalidFormatError)?;
        Text::with_baseline(
            self.line_buf.as_str(),
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut self.drv)?;

        self.line_buf.clear();
        let pos_y = DisplaySize128x32::HEIGHT as i32 - BIG_FONT.character_size.height as i32;
        write!(&mut self.line_buf, "Humid  {:.1} %", humid)
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
