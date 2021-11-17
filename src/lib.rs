#![no_std]
#![no_main]

use embedded_sdmmc::{TimeSource, Timestamp};

pub mod clock {
    use crate::{TimeSource, Timestamp};

    //Keeps tracks of when files are made
    pub struct FakeClock;

    impl TimeSource for FakeClock {
        fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
            Timestamp {
                year_since_1970: 30,
                zero_indexed_month: 2,
                zero_indexed_day: 4,
                hours: 8,
                minutes: 31,
                seconds: 2,
            }
        }
    }
}

pub mod sd {

    use core::fmt::Debug;
    use embedded_sdmmc::BlockDevice;
    use embedded_sdmmc::Directory;
    use embedded_sdmmc::TimeSource;
    use embedded_sdmmc::Volume;
    use embedded_sdmmc::VolumeIdx;

    //Make the sdmmc_spi
    pub fn make_sdmmcspi<SPI, CS>(spi: SPI, cs: CS) -> embedded_sdmmc::SdMmcSpi<SPI, CS>
    where
        SPI: embedded_hal::spi::FullDuplex<u8>,
        CS: embedded_hal::digital::v2::OutputPin,
        <SPI as embedded_hal::spi::FullDuplex<u8>>::Error: Debug,
    {
        embedded_sdmmc::SdMmcSpi::new(spi, cs)
    }

    //returns a controller to handle
    //all file handling.
    pub fn controller<D, T>(block_device: D, fake_clock: T) -> embedded_sdmmc::Controller<D, T>
    where
        D: BlockDevice,
        T: TimeSource,
        <D as BlockDevice>::Error: Debug,
    {
        embedded_sdmmc::Controller::new(block_device, fake_clock)
    }

    //Returns the volume of the controller
    pub fn get_volume<D, T>(controller: &mut embedded_sdmmc::Controller<D, T>) -> Volume
    where
        D: BlockDevice,
        T: TimeSource,
        <D as BlockDevice>::Error: Debug,
    {
        //Usually is always 0
        controller.get_volume(VolumeIdx(0)).unwrap()
    }

    //Open the root directory of the controller
    pub fn root_dir<D, T>(
        controller: &mut embedded_sdmmc::Controller<D, T>,
        volume: &Volume,
    ) -> Directory
    where
        D: BlockDevice,
        T: TimeSource,
        <D as BlockDevice>::Error: Debug,
    {
        controller.open_root_dir(volume).unwrap()
    }

    //Open file given a name
    //inside the controller's volume and directory
    pub fn open_file<D, T>(
        controller: &mut embedded_sdmmc::Controller<D, T>,
        dir: &Directory,
        volume: &mut Volume,
        file_name: &'static str,
    ) -> embedded_sdmmc::File
    where
        D: BlockDevice,
        T: TimeSource,
        <D as BlockDevice>::Error: Debug,
    {
        controller
            .open_file_in_dir(
                volume,
                &dir,
                file_name,
                embedded_sdmmc::Mode::ReadWriteAppend,
            )
            .unwrap()
    }

    //Write into the file given
    //File must be in controller's volume.
    pub fn write_into_file<D, T>(
        controller: &mut embedded_sdmmc::Controller<D, T>,
        file: &mut embedded_sdmmc::File,
        volume: &mut Volume,
        buffer: &[u8],
    ) -> usize
    where
        D: BlockDevice,
        T: TimeSource,
        <D as BlockDevice>::Error: Debug,
    {
        controller.write(volume, file, buffer).unwrap()
    }
}
