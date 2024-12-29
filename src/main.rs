use std::{
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::*,
    text::Text,
};

use epd_waveshare::{epd2in9_v2::*, prelude::*};

use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{self, PinDriver},
    peripherals,
    spi::{self, config::DriverConfig, SpiDeviceDriver, SpiDriver},
    task::{block_on, thread::ThreadSpawnConfiguration},
};

use max31855_rs::Max31855;

#[allow(dead_code)]
const HEIGHT: u32 = 128;
#[allow(dead_code)]
const WIDTH: u32 = 296;

const TEMP: AtomicI32 = AtomicI32::new(-25000);

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello from esp32");

    ThreadSpawnConfiguration::default().set().unwrap();

    let peripherals = peripherals::Peripherals::take().unwrap();

    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio10).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio11).unwrap();
    let busy = PinDriver::input(peripherals.pins.gpio22).unwrap();

    let sclk = peripherals.pins.gpio6;
    let mosi = peripherals.pins.gpio7;
    let miso = peripherals.pins.gpio2;
    let cs0 = peripherals.pins.gpio19;
    let cs1 = peripherals.pins.gpio18;

    let config = spi::config::Config::default();

    // doesn't work properly
    let spi_driver =
        Arc::new(SpiDriver::new(spi, sclk, mosi, Some(miso), &DriverConfig::default()).unwrap());

    let spi0 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs0), &config).unwrap();
    let spi1 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs1), &config).unwrap();

    thread::Builder::new()
        .name("display".to_string())
        .spawn(|| {
            let display_res = block_on(display(spi0, busy, dc, rst));
            match display_res {
                Ok(_) => log::info!("Display thread exited successfully"),
                Err(_) => log::error!("Display thread exited with an error"),
            }
        })
        .unwrap();

    thread::Builder::new()
        .name("sensor".to_string())
        .spawn(|| {
            let sensor_res = block_on(sensor(spi1));
            match sensor_res {
                Ok(_) => log::info!("Sensor thread exited successfully"),
                Err(_) => log::error!("Sensor thread exited with an error"),
            }
        })
        .unwrap();

    loop {
        thread::park();
    }
}

async fn display<'a>(
    mut spi0: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>,
    busy: PinDriver<'a, gpio::Gpio22, gpio::Input>,
    dc: PinDriver<'a, gpio::Gpio11, gpio::Output>,
    rst: PinDriver<'a, gpio::Gpio10, gpio::Output>,
) -> Result<(), &'a str> {
    let mut delay = Delay::new_default();

    let mut epd = Epd2in9::new(&mut spi0, busy, dc, rst, &mut delay, None).unwrap();
    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);

    epd.update_old_frame(&mut spi0, &display.buffer(), &mut delay)
        .unwrap();

    loop {
        delay.delay_ms(1000);

        display.clear(Color::White).unwrap();

        let temp = TEMP.load(Ordering::Relaxed) as f32 / 100_f32;

        write_text(&mut display, format!("{}", temp).as_str(), 10, 30).unwrap();

        epd.update_and_display_new_frame(&mut spi0, &display.buffer(), &mut delay)
            .unwrap();

        // Set the EPD to sleep
        epd.sleep(&mut spi0, &mut delay).unwrap();
    }
}

async fn sensor<'a>(spi1: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>) -> Result<(), &'a str> {
    let mut max = Max31855::new(spi1);

    let delay = Delay::new_default();

    loop {
        delay.delay_ms(1000);

        let data = max.read().unwrap();
        let thermo_c = data.thermo_temperature();
        TEMP.store((thermo_c * 100_f32) as i32, Ordering::Relaxed);

        log::info!("Thermocouple temperature: {}°C", thermo_c);
    }
}

fn write_text<'a>(
    display: &'a mut Display2in9,
    text: &'a str,
    x: i32,
    y: i32,
) -> Result<(), &'a str> {
    let style = MonoTextStyle::new(&FONT_10X20, Color::Black);

    Text::new(text, Point::new(x, y), style)
        .draw(display)
        .unwrap();

    return Ok(());
}
