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
        let gpioc = dp.GPIOC.split();

        let mut success_led = gpiob.pb15.into_push_pull_output();
        success_led.set_high();
        let mut failure_led = gpioc.pc6.into_open_drain_output();
        failure_led.set_high();
        let mut orange_led = gpioe.pe1.into_open_drain_output();
        orange_led.set_high();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let gpioa = dp.GPIOA.split();

        // SPI1_SCK  PA5 D13
        // SPI1_MISO PA6 D12
        // SPI1_MOSI PA7 D11

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
            800.khz(), //100-800
            clocks,
        );

        //STM32F412 PB8 D9
        //STM32F401 PB10
        //

        //Buffer to contain any data we want to
        //store in a file.
        //TODO Currently it saves empty bytes
        //after reading from file. We need
        //to make it into a string
        let mut buffer = [0u8; 4096];

        //CS PB8 D9
        let cs = gpiob.pb8.into_push_pull_output();

        let sdmmc_spi = SdMmcSpi::new(spi, cs);

        let fake_clock = FakeClock {}; //Fake clock

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

        //What the volume we will use.
        //Usually is 0
        let mut first_volume = match controller.get_volume(VolumeIdx(0)) {
            Ok(v) => v,
            Err(_) => {
                orange_led.set_low();

                panic!("Yikes");
            }
        };

        //Opens the root directory according volume given.
        let mut root_dir = controller.open_root_dir(&first_volume).unwrap();

        //Opens a file according to the: Volume, Directory, NAMEOFFILE
        //and for last what MODE we open the file as.
        let mut read_file = controller
            .open_file_in_dir(
                &mut first_volume,
                &mut root_dir,
                "_Lalala.txt",
                embedded_sdmmc::Mode::ReadOnly,
            )
            .unwrap();

        //Method to read file and put data into buffer.
        controller
            .read(&mut first_volume, &mut read_file, &mut buffer)
            .unwrap();

        //Closes file specified.
        //MAKE SURE TO DO AFTER USING FILE
        controller.close_file(&first_volume, read_file).unwrap();

        //This opens an output file or creates it if not found.
        let mut output_file = controller
            .open_file_in_dir(
                &mut first_volume,
                &mut root_dir,
                "OutPut.txt",
                embedded_sdmmc::Mode::ReadWriteCreate,
            )
            .unwrap();

        //Writes to the output file with the buffer we give it.
        controller
            .write(&mut first_volume, &mut output_file, &buffer)
            .unwrap();

        //Close file.
        controller.close_file(&first_volume, output_file).unwrap();

        //Closes directory
        controller.close_dir(&first_volume, root_dir);

        success_led.set_low();

        // let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);
    }

    panic!(":)");

    loop {}
}
