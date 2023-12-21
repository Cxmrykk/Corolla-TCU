use esp_idf_svc::hal as esp_idf_hal;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::delay::FreeRtos;
use std::io::{Write, stdout};

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut gpio40 = PinDriver::input(peripherals.pins.gpio40)?; // LPS
    let mut gpio41 = PinDriver::input(peripherals.pins.gpio41)?; // SS1
    let mut gpio42 = PinDriver::input(peripherals.pins.gpio42)?; // SS2
    let mut gpio43 = PinDriver::input(peripherals.pins.gpio43)?; // TCS
    let mut gpio44 = PinDriver::input(peripherals.pins.gpio44)?; // LPS GND
    let mut gpio45 = PinDriver::input(peripherals.pins.gpio45)?; // VSS
    let mut gpio46 = PinDriver::input(peripherals.pins.gpio46)?; // RPM
    //let mut gpio15 = PinDriver::input(peripherals.pins.gpio15)?; // TPS

    // Set pull down mode for all GPIO
    gpio40.set_pull(Pull::Down)?;
    gpio41.set_pull(Pull::Down)?;
    gpio42.set_pull(Pull::Down)?;
    gpio43.set_pull(Pull::Down)?;
    gpio44.set_pull(Pull::Down)?;
    gpio45.set_pull(Pull::Down)?;
    gpio46.set_pull(Pull::Down)?;
    //gpio15.set_pull(Pull::Down)?;

    loop {
        let buffer: [u8; 1] = [
            0x00
            | (gpio40.is_high() as u8)
            | (gpio41.is_high() as u8) << 1
            | (gpio42.is_high() as u8) << 2
            | (gpio43.is_high() as u8) << 3
            | (gpio44.is_high() as u8) << 4
            | (gpio44.is_high() as u8) << 5
            | (gpio46.is_high() as u8) << 6
            //| (gpio15.is_high() as u8) << 7
        ];

        stdout().write_all(&buffer)?;
        
        FreeRtos::delay_ms(10);
    }
}
