use crate::hal::hal::{digital::v2::OutputPin, spi::FullDuplex};
use crate::system_clock::SystemClock;
use embedded_sdmmc::{Controller, Error, Mode, SdMmcError, SdMmcSpi, TimeSource, VolumeIdx};
use embedded_time::{duration::Minutes, Instant};

const WRITE_INTERVAL: Minutes = Minutes(15_u32);
const VOLUME_IDX: VolumeIdx = VolumeIdx(0);
const FILENAME: &str = "RECORDS.CSV";

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct InitializedStateData {
    card_size_bytes: u64,
    last_write: Instant<SystemClock>,
}

pub struct FileSystem<SPI: FullDuplex<u8>, CS: OutputPin, T: TimeSource>
where
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug,
{
    ctrl: Controller<SdMmcSpi<SPI, CS>, T>,
    data: Option<InitializedStateData>,
}

impl<SPI, CS, T> FileSystem<SPI, CS, T>
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug,
    T: TimeSource,
{
    pub fn new(spi: SPI, cs: CS, timesource: T) -> Result<Self, Error<SdMmcError>> {
        let dev = SdMmcSpi::new(spi, cs);
        let ctrl = Controller::new(dev, timesource);
        Ok(FileSystem { ctrl, data: None })
    }

    /// Call this when card is connected
    pub fn init(&mut self, now: &Instant<SystemClock>) -> Result<(), Error<SdMmcError>> {
        if self.data.is_none() {
            let dev = self.ctrl.device();
            dev.init().map_err(Error::DeviceError)?;
            let card_size_bytes = dev.card_size_bytes().map_err(Error::DeviceError)?;
            self.data.replace(InitializedStateData {
                card_size_bytes,
                last_write: *now,
            });
        }
        Ok(())
    }

    /// Call this when card is disconnected
    pub fn deinit(&mut self) {
        if let Some(_data) = self.data.take() {
            let dev = self.ctrl.device();
            dev.deinit();
        }
    }

    pub fn is_init(&self) -> bool {
        self.data.is_some()
    }

    /// Returns true if did write
    pub fn write(
        &mut self,
        now: &Instant<SystemClock>,
        buffer: &[u8],
    ) -> Result<bool, Error<SdMmcError>> {
        if let Some(data) = &mut self.data {
            if let Some(dur) = now.checked_duration_since(&data.last_write) {
                if dur >= WRITE_INTERVAL.into() {
                    data.last_write = *now;
                    let mut volume = self.ctrl.get_volume(VOLUME_IDX)?;
                    let root_dir = self.ctrl.open_root_dir(&volume)?;
                    let mut file = self.ctrl.open_file_in_dir(
                        &mut volume,
                        &root_dir,
                        FILENAME,
                        Mode::ReadWriteCreateOrAppend,
                    )?;
                    self.ctrl.write(&mut volume, &mut file, buffer)?;
                    self.ctrl.close_file(&volume, file)?;
                    self.ctrl.close_dir(&volume, root_dir);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
