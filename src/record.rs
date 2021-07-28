use crate::util;
use bme680::FieldData;
use core::fmt::Write;
use ds323x::{Datelike, NaiveDateTime, Timelike};
use heapless::String;

// TODO - if these get big, put them in the bss section instead of on the stack
const TIMESTAMP_STRING_CAP: usize = 32;
const CSV_LINE_STRING_CAP: usize = TIMESTAMP_STRING_CAP + (4 * 16);

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Could not format string")]
    StringFormatting,
}

#[derive(Debug)]
pub struct Record {
    /// ds323x::NaiveDateTime encoded as a ISO 8601 combined date and time
    /// (without timezone) string.
    /// YYYY-MM-DDThh:mm:ss
    pub timestamp: String<TIMESTAMP_STRING_CAP>,

    /// Temperature in degree fahrenheit (°F)
    pub temperature: f32,

    /// Humidity in % relative humidity
    pub humidity: f32,

    /// Pressure in hectopascal (hPA)
    pub pressure: f32,

    /// Gas resistance, present if the valid bit is set on the BME689
    /// If not valid, value 0 is used
    pub gas_resistance: Option<u32>,
}

// TODO - probably don't need to have intermediate state, just convert to csv
// string
impl Record {
    pub fn new(datetime: &NaiveDateTime, data: &FieldData) -> Result<Self, Error> {
        let mut timestamp = String::new();

        let date = datetime.date();
        let time = datetime.time();
        write!(
            &mut timestamp,
            "{}-{:02}-{:02}T{:02}:{:02}:{:02}",
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
        )
        .map_err(|_| Error::StringFormatting)?;

        Ok(Record {
            timestamp,
            temperature: util::celsius_to_fahrenheit(data.temperature_celsius()),
            humidity: data.humidity_percent(),
            pressure: data.pressure_hpa(),
            gas_resistance: if data.gas_valid() {
                data.gas_resistance_ohm().into()
            } else {
                None
            },
        })
    }

    pub fn to_csv_line(&self) -> Result<String<CSV_LINE_STRING_CAP>, Error> {
        let mut s = String::new();
        write!(
            &mut s,
            "{},{},{},{},{}",
            self.timestamp,
            self.temperature,
            self.humidity,
            self.pressure,
            self.gas_resistance.unwrap_or(0)
        )
        .map_err(|_| Error::StringFormatting)?;
        Ok(s)
    }
}
