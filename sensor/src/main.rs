use std::{
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, LazyLock, RwLock,
    },
    thread,
};

use embedded_graphics::prelude::*;

use epd_waveshare::{epd2in9_v2::*, prelude::*};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::Delay,
        gpio::{self, PinDriver},
        peripherals,
        spi::{self, config::DriverConfig, SpiDeviceDriver, SpiDriver},
    },
    nvs::EspDefaultNvsPartition,
    sntp::EspSntp,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use max31855_rs::Max31855;
use u8g2_fonts::{
    fonts,
    types::{HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

static TEMP: AtomicI32 = AtomicI32::new(-25000);

static WIFI_STATUS: LazyLock<Arc<RwLock<String>>> =
    LazyLock::new(|| Arc::new(RwLock::new("Not available".to_string())));

// TODO: avoid unwrap, handle gracefully
// TODO: int temp https://github.com/esp-rs/esp-idf-hal/blob/master/examples/temperature_sensor.rs

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("main: Starting smart-hot-water-tank");

    let peripherals = peripherals::Peripherals::take()?;

    // display
    let rst = PinDriver::output(peripherals.pins.gpio10)?;
    let dc = PinDriver::output(peripherals.pins.gpio11)?;
    let busy = PinDriver::input(peripherals.pins.gpio22)?;

    // spi
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio6;
    let mosi = peripherals.pins.gpio7;
    let miso = peripherals.pins.gpio2;
    // display
    let cs0 = peripherals.pins.gpio19;
    // amplifier
    let cs1 = peripherals.pins.gpio18;

    let config = spi::config::Config::default();

    let spi_driver = Arc::new(SpiDriver::new(
        spi,
        sclk,
        mosi,
        Some(miso),
        &DriverConfig::default(),
    )?);

    // display
    let spi0 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs0), &config)?;
    // amplifier
    let spi1 = SpiDeviceDriver::new(spi_driver.clone(), Some(cs1), &config)?;

    // wifi
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    thread::Builder::new()
        .name("display".to_string())
        .stack_size(8192)
        .spawn(|| {
            let display_res = display(spi0, busy, dc, rst);
            match display_res {
                Ok(_) => log::info!("main: Display thread exited successfully"),
                Err(e) => log::error!("main: Display thread exited with an error. {:?}", e),
            }
        })?;

    thread::Builder::new()
        .name("sensor".to_string())
        .stack_size(8192)
        .spawn(|| {
            let sensor_res = sensor(spi1);
            match sensor_res {
                Ok(_) => log::info!("main: Sensor thread exited successfully"),
                Err(e) => log::error!("main: Sensor thread exited with an error. {:?}", e),
            }
        })?;

    thread::Builder::new()
        .name("network".to_string())
        .stack_size(8192)
        .spawn(|| {
            let sensor_res = network(wifi);
            match sensor_res {
                Ok(_) => log::info!("main: Network thread exited successfully"),
                Err(e) => log::error!("main: Network thread exited with an error. {:?}", e),
            }
        })?;

    loop {
        thread::park();
    }
}

fn display<'a>(
    mut spi0: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>,
    busy: PinDriver<'a, gpio::Gpio22, gpio::Input>,
    dc: PinDriver<'a, gpio::Gpio11, gpio::Output>,
    rst: PinDriver<'a, gpio::Gpio10, gpio::Output>,
) -> anyhow::Result<()> {
    let mut delay = Delay::new_default();

    let mut epd = Epd2in9::new(&mut spi0, busy, dc, rst, &mut delay, None).unwrap();
    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);
    display.clear(Color::White).unwrap();

    epd.update_and_display_frame(&mut spi0, display.buffer(), &mut delay)
        .unwrap();

    epd.update_old_frame(&mut spi0, display.buffer(), &mut delay)
        .unwrap();

    loop {
        delay.delay_ms(1000);

        display.clear(Color::White).unwrap();

        // ip
        {
            let ip = WIFI_STATUS.try_read();
            match ip {
                Ok(ip) => {
                    write_ip(&mut display, ip.to_string()).unwrap();
                }
                Err(_) => {
                    log::error!("display: IP: Lock not available");
                }
            }
        }
        // temp
        {
            let temp = TEMP.load(Ordering::SeqCst) as f32 / 100_f32;

            let integer_part = temp.trunc() as i32;
            let fractional_part = (temp.fract() * 100_f32) as i32;

            write_temp(&mut display, integer_part, fractional_part).unwrap();
        }

        epd.update_and_display_new_frame(&mut spi0, display.buffer(), &mut delay)
            .unwrap();
    }
}

fn sensor<'a>(spi1: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>) -> anyhow::Result<()> {
    let mut max = Max31855::new(spi1);

    let delay = Delay::new_default();

    loop {
        delay.delay_ms(1000);

        let data = max.read().unwrap();
        let thermo_c = data.thermo_temperature();
        let thermo_i = (thermo_c * 100_f32) as i32;

        TEMP.store(thermo_i, Ordering::SeqCst);

        log::info!("sensor: Thermo: {}°C", thermo_c);
    }
}

fn network<'a>(mut wifi: BlockingWifi<EspWifi<'a>>) -> anyhow::Result<()> {
    let delay = Delay::new_default();

    // TODO: gracefully handle timeouts or disconnects
    connect_wifi(&mut wifi)?;

    let _sntp = EspSntp::new_default()?;
    log::info!("network: SNTP initialized");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    log::info!("network: Wifi connected with IP: {:?}", ip_info);

    loop {
        delay.delay_ms(1000);
        let mut write_lock = WIFI_STATUS.write().unwrap();
        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
        *write_lock = ip_info.ip.to_string();
        drop(write_lock);
    }
}

fn write_temp(display: &mut Display2in9, int: i32, frac: i32) -> anyhow::Result<()> {
    let font = FontRenderer::new::<fonts::u8g2_font_logisoso58_tf>();

    // render temp, int and frac part. int part should be larger then the frac part
    let text = &*format!("{}.{:02}°C", int, frac);

    font.render_aligned(
        text,
        Point::new(280, 64),
        VerticalPosition::Center,
        HorizontalAlignment::Right,
        u8g2_fonts::types::FontColor::Transparent(Color::Black),
        display,
    )
    .unwrap();

    Ok(())
}

fn connect_wifi<'a>(wifi: &mut BlockingWifi<EspWifi<'a>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2WPA3Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    log::info!("wifi: Wifi started");

    wifi.connect()?;
    log::info!("wifi: Wifi connected");

    wifi.wait_netif_up()?;
    log::info!("wifi: Wifi netif up");

    Ok(())
}

fn write_ip(display: &mut Display2in9, ip: String) -> anyhow::Result<()> {
    let font = FontRenderer::new::<fonts::u8g2_font_profont17_tr>();
    let icon = FontRenderer::new::<fonts::u8g2_font_streamline_interface_essential_wifi_t>();

    let icon_box = icon
        .render_aligned(
            '\u{0030}',
            Point::new(0, 128),
            VerticalPosition::Bottom,
            HorizontalAlignment::Left,
            u8g2_fonts::types::FontColor::Transparent(Color::Black),
            display,
        )
        .unwrap()
        .unwrap();

    font.render_aligned(
        &*ip,
        icon_box.bottom_right().unwrap() + Point::new(5, 0),
        VerticalPosition::Bottom,
        HorizontalAlignment::Left,
        u8g2_fonts::types::FontColor::Transparent(Color::Black),
        display,
    )
    .unwrap();

    Ok(())
}
