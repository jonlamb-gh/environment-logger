use crate::util;
use bme680::FieldData;
use core::fmt::Write;
use ds323x::{Datelike, NaiveDateTime, Timelike};
use heapless::String;

const STRING_CAP: usize = 64;

// TODO - csv, serde stuff, maybe bincode but csv prefered
// might be able to keep NaiveDateTime instead of string by importing chrono
// with serde features
#[derive(Debug)]
pub struct Record {
    /// ds323x::NaiveDateTime encoded as a ISO 8601 combined date and time
    /// (without timezone) string.
    /// YYYY-MM-DDThh:mm:ss
    pub timestamp: String<STRING_CAP>,

    /// Temperature in degree fahrenheit (°F)
    pub temperature: f32,

    /// Humidity in % relative humidity
    pub humidity: f32,

    /// Pressure in hectopascal (hPA)
    pub pressure: f32,

    /// Gas resistance, present if the valid bit is set on the BME689
    pub gas_resistance: Option<u32>,
}

impl Record {
    // TODO - error type
    pub fn new(datetime: &NaiveDateTime, data: &FieldData) -> Self {
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
        .unwrap();

        Record {
            timestamp,
            temperature: util::celsius_to_fahrenheit(data.temperature_celsius()),
            humidity: data.humidity_percent(),
            pressure: data.pressure_hpa(),
            gas_resistance: if data.gas_valid() {
                data.gas_resistance_ohm().into()
            } else {
                None
            },
        }
    }
}
