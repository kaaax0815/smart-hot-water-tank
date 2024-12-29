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
    timer::{config, TimerDriver},
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

    // correctly logged
    log::info!("Hello, world!");

    ThreadSpawnConfiguration::default().set().unwrap();

    let peripherals = peripherals::Peripherals::take().unwrap();

    // correctly logged
    log::info!("Hello, world!");

    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio4).unwrap();
    let busy = PinDriver::input(peripherals.pins.gpio22).unwrap();

    // not logged
    log::info!("Hello, world!");

    let sclk = peripherals.pins.gpio6;
    let mosi = peripherals.pins.gpio7;
    let miso = peripherals.pins.gpio2;
    let cs0 = peripherals.pins.gpio16;
    let cs1 = peripherals.pins.gpio17;

    let config = spi::config::Config::default();

    // doesn't work properly
    let spi_driver =
        Arc::new(SpiDriver::new(spi, sclk, mosi, Some(miso), &DriverConfig::default()).unwrap());

    let spi0 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs0), &config).unwrap();
    let spi1 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs1), &config).unwrap();

    let timer0 = TimerDriver::new(peripherals.timer00, &config::Config::default()).unwrap();
    let timer1 = TimerDriver::new(peripherals.timer10, &config::Config::default()).unwrap();

    // not logged
    log::info!("Hello, world!");

    thread::Builder::new()
        .name("display".to_string())
        .spawn(|| {
            let display_res = block_on(display(spi0, busy, dc, rst, timer0));
            match display_res {
                Ok(_) => log::info!("Display thread exited successfully"),
                Err(_) => log::error!("Display thread exited with an error"),
            }
        })
        .unwrap();

    thread::Builder::new()
        .name("sensor".to_string())
        .spawn(|| {
            let sensor_res = block_on(sensor(spi1, timer1));
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
    dc: PinDriver<'a, gpio::Gpio4, gpio::Output>,
    rst: PinDriver<'a, gpio::Gpio5, gpio::Output>,
    mut timer: TimerDriver<'a>,
) -> Result<(), &'a str> {
    let mut delay = Delay::new_default();

    let mut epd = Epd2in9::new(&mut spi0, busy, dc, rst, &mut delay, None).unwrap();
    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);

    loop {
        timer.delay(1000000).await.unwrap();

        display.clear(Color::White).unwrap();

        let temp = TEMP.load(Ordering::Relaxed) as f32 / 100_f32;

        write_text(&mut display, format!("{}", temp).as_str(), 10, 30).unwrap();

        epd.update_frame(&mut spi0, &display.buffer(), &mut delay)
            .unwrap();
        epd.display_frame(&mut spi0, &mut delay).unwrap();

        // Set the EPD to sleep
        epd.sleep(&mut spi0, &mut delay).unwrap();
    }
}

async fn sensor<'a>(
    spi1: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>,
    mut timer: TimerDriver<'a>,
) -> Result<(), &'a str> {
    let mut max = Max31855::new(spi1);

    loop {
        timer.delay(1000000).await.unwrap();

        let data = max.read().unwrap();
        let thermo_c = data.thermo_temperature();
        TEMP.store((thermo_c * 100_f32) as i32, Ordering::Relaxed);
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
