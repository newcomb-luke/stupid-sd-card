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

use stupid_sd_card::sd;

use crate::hal::{pac, prelude::*};

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
        //  let mut buffer1 = [0u8; 4096];

        //CS PB8 D9
        let cs = gpiob.pb8.into_push_pull_output();

        let sdmmc_spi = sd::make_sdmmcspi(spi, cs);

        let fake_clock = stupid_sd_card::clock::FakeClock; //Fake clock

        let mut controller = sd::controller(sdmmc_spi, fake_clock);

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
        let mut volume = sd::get_volume(&mut controller);
        //Opens the root directory according volume given.
        let root_dir = sd::root_dir(&mut controller, &mut volume);

        let file_name: &'static str = "offset.txt";

        //Opens a file according to the: Volume, Directory, NAMEOFFILE
        //and for last what MODE we open the file as.
        //This opens an output file or creates it if not found.
        let mut output_file = sd::open_file(&mut controller, &root_dir, &mut volume, file_name);
        //Writes to the output file with the buffer we give it.

        sd::write_into_file(
            &mut controller,
            &mut output_file,
            &mut volume,
            "Testing writing sd mod".as_bytes(),
        );
        //Close file.

        controller.close_file(&volume, output_file).unwrap();

        //Closes directory
        controller.close_dir(&volume, root_dir);

        success_led.set_low();

        // let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);
    }

    panic!(":)");

    loop {}
}
