use esp_idf_svc::hal as esp_idf_hal;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::task::block_on;
use std::io::{Write, stdout};

const START_MARKER: u8 = 0x01; // SOH
const END_MARKER: u8 = 0x0A; // CARRIAGE RETURN (line termination sends buffer to stdout, converts to line feed)

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    // Assign GPIO pins
    let peripherals = Peripherals::take()?;
    let mut gpio40 = PinDriver::input(peripherals.pins.gpio40)?; // LPS
    let mut gpio41 = PinDriver::input(peripherals.pins.gpio41)?; // SS1
    let mut gpio42 = PinDriver::input(peripherals.pins.gpio42)?; // SS2
    let mut gpio43 = PinDriver::input(peripherals.pins.gpio43)?; // TCS
    let mut gpio44 = PinDriver::input(peripherals.pins.gpio44)?; // LPS GND
    let mut gpio45 = PinDriver::input(peripherals.pins.gpio45)?; // VSS
    let mut gpio46 = PinDriver::input(peripherals.pins.gpio46)?; // RPM
    //let mut gpio15 = PinDriver::input(peripherals.pins.gpio15)?; // TPS

    // Set mode for GPIO pins
    gpio40.set_pull(Pull::Floating)?;
    gpio41.set_pull(Pull::Floating)?;
    gpio42.set_pull(Pull::Floating)?;
    gpio43.set_pull(Pull::Floating)?;
    gpio44.set_pull(Pull::Floating)?;
    gpio45.set_pull(Pull::Floating)?;
    gpio46.set_pull(Pull::Floating)?;
    //gpio15.set_pull(Pull::Floating)?;

    let mut stdout = stdout();
    let mut buffer: [u8; 10] = [0; 10];
    buffer[0] = START_MARKER;
    buffer[9] = END_MARKER;

    block_on(async {
        loop {
            for i in 1..=8 {
                buffer[i] = 0x00
                | (gpio40.is_high() as u8)
                | (gpio41.is_high() as u8) << 1
                | (gpio42.is_high() as u8) << 2
                | (gpio43.is_high() as u8) << 3
                | (gpio44.is_high() as u8) << 4
                | (gpio45.is_high() as u8) << 5
                | (gpio46.is_high() as u8) << 6
                //| (gpio15.is_high() as u8) << 7
                ;
            }

            stdout.write_all(&buffer)?;
            stdout.flush()?;
        }
    })
}
