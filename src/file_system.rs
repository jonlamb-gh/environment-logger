// TODO
// Mode::ReadWriteCreateOrAppend

use crate::hal::hal::{digital::v2::OutputPin, spi::FullDuplex};
use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Controller, Error, Mode, SdMmcError, SdMmcSpi,
    TimeSource, Timestamp, VolumeIdx,
};

const VOLUME_IDX: VolumeIdx = VolumeIdx(0);
const FILENAME: &str = "records.txt";

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct InitializedStateData {
    card_size_bytes: u64,
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
    pub fn init(&mut self) -> Result<(), Error<SdMmcError>> {
        let dev = self.ctrl.device();
        dev.init().map_err(|e| Error::DeviceError(e))?;
        let card_size_bytes = dev.card_size_bytes().map_err(|e| Error::DeviceError(e))?;
        self.data.replace(InitializedStateData { card_size_bytes });
        Ok(())
    }

    /// Call this when card is disconnected
    pub fn deinit(&mut self) {
        let dev = self.ctrl.device();
        dev.deinit();
    }

    // option/result, may not have been init'd
    pub fn is_full(&mut self) -> bool {
        // if file exists, check the DirEntry.size minus a block size or something
        todo!()
    }
}

// https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/develop/examples/create_test.rs

// TODO
// https://docs.rs/embedded-sdmmc/0.3.0/embedded_sdmmc/struct.DirEntry.html#structfield.size
// check if file exist, if so, see if the card is full
