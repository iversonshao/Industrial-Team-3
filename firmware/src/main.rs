mod display;
mod mqtt;
mod network;
mod piezo;

use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use log::{error, info};

use crate::display::Display;
use crate::piezo::{Piezo, Tone};

/// This configuration is picked up at compile time by `build.rs` from the
/// file `cfg.toml`. The constant `CONFIG` is auto-generated by `toml_config`.
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    aws_endpoint: &'static str,
    #[default("")]
    aws_privkey: &'static str,
    #[default("")]
    aws_cert: &'static str,
}

/// Entry point to our application.
///
/// It sets up a Wi-Fi connection to the Access Point given in the
/// configuration, then blinks the RGB LED green/blue.
///
/// If the LED goes solid red, then it was unable to connect to your Wi-Fi
/// network.
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();   
    esp_idf_svc::log::EspLogger::initialize_default();
    unsafe { esp_idf_svc::sys::nvs_flash_init() };

    let mut peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // hardware i2c bus 0, sda pin 19, scl pin 18
    let mut display =
        Display::init(peripherals.i2c0, peripherals.pins.gpio19, peripherals.pins.gpio18);
    
    let mut piezo = 
        Piezo::init(peripherals.pins.gpio21, peripherals.ledc.timer0, peripherals.ledc.channel0);

    info!("Hello, world!");

    piezo.sound(Tone::E5, 220, 20);
    piezo.sound(Tone::A5, 220, 20);
    piezo.sound(Tone::E5, 220, 20);
    
    display.text_demo("connecting...");

    let app_config = CONFIG;

    let _wifi = loop {
        // Connect to the Wi-Fi network
        match network::wifi_conn(
            CONFIG.wifi_ssid,
            CONFIG.wifi_psk,
            &mut peripherals.modem,
            sysloop.clone(),
        ) {
            Ok(inner) => break inner,
            Err(err) => {
                error!("Could not connect to Wi-Fi network: {:?}, trying again...", err);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };

    display.text_demo("hiiiii :3");
    std::thread::sleep(std::time::Duration::from_secs(3));

    loop {
        display.veryhappy_anim();
    }

    loop {
        // Wait...
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!("Hello, world!");
    }
}
