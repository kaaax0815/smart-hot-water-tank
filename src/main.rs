use embedded_graphics::{
 mono_font::{ascii::FONT_6X12, MonoTextStyle}, prelude::*, text::Text
};

use epd_waveshare::{epd2in9_v2::*, prelude::*};

use esp_idf_svc::hal::{delay::Delay, gpio::PinDriver, peripherals, spi::{self, SpiDeviceDriver, SpiDriverConfig}};

#[allow(dead_code)]
const HEIGHT: u32 = 128;
#[allow(dead_code)]
const WIDTH: u32 = 296;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = peripherals::Peripherals::take().unwrap();

    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio4).unwrap();
    let busy = PinDriver::input(peripherals.pins.gpio22).unwrap();
   
    let sclk = peripherals.pins.gpio6;
    let mosi = peripherals.pins.gpio7;
    let miso = peripherals.pins.gpio2;
    let cs = peripherals.pins.gpio16;

    let mut delay = Delay::new_default();

    let config = spi::config::Config::default();

    let mut spi0 = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Some(miso),
        Some(cs),
        &SpiDriverConfig::default(),
        &config,
    ).unwrap();

    let mut epd = Epd2in9::new(&mut spi0, busy, dc, rst, &mut delay, None).unwrap();

    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);
    display.clear(Color::White).unwrap();

    write_text(&mut display, "Hello, World!", 10, 10).unwrap();

    epd.update_frame(&mut spi0, &display.buffer(), &mut delay).unwrap();
    epd.display_frame(&mut spi0, &mut delay).unwrap();

    // Set the EPD to sleep
    epd.sleep(&mut spi0, &mut delay).unwrap();
}

fn write_text<'a>(display: &'a mut Display2in9, text: &'a str, x: i32, y: i32) -> Result<(), &'a str> {
    let style = MonoTextStyle::new(&FONT_6X12, Color::Black);

    Text::new(text, Point::new(x, y), style)
        .draw(display).unwrap();

    return Ok(());
}
