#![no_main]
#![no_std]

use embedded_hal::{
    digital::v2::OutputPin,
    spi::{Mode, Phase, Polarity},
};
use embedded_sdmmc::{Controller, SdMmcSpi, TimeSource, Timestamp, VolumeIdx};
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

struct FakeClock;

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

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let gpioe = dp.GPIOE.split();

        let gpiob = dp.GPIOB.split();

        let mut success_led = gpiob.pb15.into_push_pull_output();
        success_led.set_high();
        let mut failure_led = gpioe.pe2.into_open_drain_output();
        failure_led.set_high();
        let mut orange_led = gpioe.pe1.into_open_drain_output();
        orange_led.set_high();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(80.mhz()).freeze();

        let gpioa = dp.GPIOA.split();

        // SPI1_SCK  PA5
        // SPI1_MISO PA6
        // SPI1_MOSI PA7

        let sck = gpioa.pa5.into_alternate();
        let miso = gpioa.pa6.into_alternate();
        let mosi = gpioa.pa7.into_alternate();

        let pins = (sck, miso, mosi);

        let spi = crate::hal::spi::Spi::new(
            dp.SPI1,
            pins,
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            200.khz(),
            clocks,
        );

        let cs = gpiob.pb8.into_push_pull_output();

        let sdmmc_spi = SdMmcSpi::new(spi, cs);

        let fake_clock = FakeClock {};

        let mut controller = Controller::new(sdmmc_spi, fake_clock);

        // Init SD card
        match controller.device().init() {
            Ok(_) => {
                success_led.set_low();
            }
            Err(_) => {
                failure_led.set_low();

                panic!("Yikes");
            }
        }

        let mut first_volume = match controller.get_volume(VolumeIdx(0)) {
            Ok(v) => v,
            Err(_) => {
                orange_led.set_low();

                panic!("Yikes");
            }
        };

        let mut root_dir = controller.open_root_dir(&first_volume).unwrap();

        let mut test_file = controller
            .open_file_in_dir(
                &mut first_volume,
                &mut root_dir,
                "TEST5.txt",
                embedded_sdmmc::Mode::ReadWriteCreateOrTruncate,
            )
            .unwrap();

        controller
            .write(
                &mut first_volume,
                &mut test_file,
                "Hello darkness my old friend".as_bytes(),
            )
            .unwrap();

        controller.close_file(&first_volume, test_file).unwrap();

        controller.close_dir(&first_volume, root_dir);

        success_led.set_low();

        // let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);
    }

    panic!(":)");

    loop {}
}
