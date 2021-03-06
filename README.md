# environment-logger

Displays and optionally logs data from a BME680 sensor to an SD card, housed in a solid block of Cherry wood.

![sensor_readings.jpg](images/sensor_readings.jpg)

![under.jpg](images/under.jpg)

## Plotting

```bash
./plotter/plot-records /path/to/RECORDS.CSV
```

![env_logger_plots.png](images/env_logger_plots.png)

## Dependencies

* [cargo-embed](https://github.com/probe-rs/cargo-embed)
```bash
cargo install cargo-embed
```

## Build and Upload

```bash
cargo embed --release
```

## Hardware

* [STM32F4x1 MiniF4 - STM32F411 BlackPill](https://github.com/WeActTC/MiniSTM32F4x1)
  - [pinout](https://raw.githubusercontent.com/WeActTC/MiniSTM32F4x1/master/images/STM32F4x1_PinoutDiagram_RichardBalint.png)
  - [STM32F411CEU6 refman](https://www.st.com/resource/en/reference_manual/dm00119316-stm32f411xc-e-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf)
  - [datasheet](https://www.st.com/resource/en/datasheet/stm32f411ce.pdf)
* [SSD1306 128x64 OLED Display Module, White, GeeekPi](https://www.amazon.com/gp/product/B0833PF7ML/ref=ppx_yo_dt_b_asin_title_o00_s00?ie=UTF8&psc=1)
* [Adafruit BME680 Sensor](https://www.adafruit.com/product/3660)
* [Adafruit Micro SD Card Breakout Board](https://www.adafruit.com/product/4682)
* [Adafruit DS3231 RTC](https://www.adafruit.com/product/3013)
* [Piezo buzzer, 5 kHz square wave](https://www.amazon.com/gp/product/B085XQM69Z/ref=ppx_yo_dt_b_search_asin_title?ie=UTF8&psc=1)
* [AMS1117-3.3 Regulator](https://www.amazon.com/gp/product/B07CP4P5XJ/ref=ppx_yo_dt_b_asin_title_o00_s00?ie=UTF8&psc=1)

### Pins

| Description       | GPIO  | AF    |
| :---              | :--:  | :--:  |
| On-board LED      | PC13  | DOUT  |
| On-board Button   | PA0   | DIN   |
| Debug serial out  | PA2   | TX2   |
| SSD1306 I2C       | PB6   | SCL1  |
| SSD1306 I2C       | PB7   | SDA1  |
| BME680 I2C        | PB10  | SCL2  |
| BME680 I2C        | PB3   | SDA2  |
| DS3231 I2C        | PA8   | SCL3  |
| DS3231 I2C        | PB4   | SDA3  |
| SD SPI            | PA15  | NSS1  |
| SD SPI            | PA5   | SCK1  |
| SD SPI            | PA6   | MISO1 |
| SD SPI            | PA7   | MOSI1 |
| SD DET            | PC15  | DIN   |
| Buzzer            | PA10  | DOUT, T1_CH3  |
