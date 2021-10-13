#![no_std]
#![no_main]

use core::fmt::Debug;

use embedded_sdmmc::{BlockDevice, Controller, TimeSource, Timestamp};

struct FakeClock;

impl TimeSource for FakeClock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 31,
            zero_indexed_month: 2,
            zero_indexed_day: 4,
            hours: 8,
            minutes: 31,
            seconds: 2,
        }
    }
}
pub struct Generic_Controller<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    <D as BlockDevice>::Error: Debug,
{
    block_device: D,
    time_source: T,
}

impl<D, T> Generic_Controller<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    <D as BlockDevice>::Error: Debug,
{
    pub fn new(block_device: D, time_source: T) -> Generic_Controller<D, T> {
        Generic_Controller {
            block_device,
            time_source,
        }
    }
}
