// Basic RUST tutorial
// Crate: A package in RUST
// Modules: files or folders within a crate that organize code

// #![deny(warnings)]
#![no_main]
#![no_std]

// Imports
// pac is a module being imported from crate (peripheral access crate)
// prelude is another module being imported, the asterix means import everything that's public inside it

// I2C is a struct
// {...} Means you can grab multiple items from one crate
// :: imports modules like folder paths

// Import example
// # Python
// from mylib import i2c
// from mylib.i2c import I2c

// # RUST
// use mylib::i2c;
// use mylib::i2c::I2c;
mod led;  // Tells the compiler to look for a file called led.rs
use led::Led;

mod chip;
use chip::BME680;

use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*};
use rtt_target::{rtt_init_log, rprintln};
use log::{info, LevelFilter};

// const GREEN: &str = "\x1b[32m";
// const RED: &str = "\x1b[31m";
// const RESET: &str = "\x1b[0m";

#[entry]
fn main() -> ! {

    // Set up RTT log backend
    rtt_init_log!(LevelFilter::Info);

    // Initialize printing
    rprintln!("Program start");
    rprintln!();

    let cp = cortex_m::Peripherals::take().unwrap();  // cp is the core peripherals
    let dp = pac::Peripherals::take().unwrap();  // dp is the device peripherals

    // Constrain and Freeze power
    info!("Setting up power...");
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.ldo().freeze();

    // Constrain and Freeze clock
    info!("Setting up clock...");
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    // Set up I2C
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let scl = gpiob.pb8.into_alternate_open_drain();
    let sda = gpiob.pb9.into_alternate_open_drain();
    let mut i2c = dp.I2C1.i2c((scl, sda), 100.kHz(), ccdr.peripheral.I2C1, &ccdr.clocks);

    // LED class
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let led_pin = gpioe.pe1.into_push_pull_output();
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let mut led = Led::new(led_pin);

    // Set up BME680
    // 🔹 Probe for the chip
    let mut bme = BME680::probe(&mut i2c).expect("Unable to initialize sensor");

    // Start loop
    info!("Start Loop...");
    rprintln!();

    loop {
        led.blink(&mut delay, 1000);

        let _pressure = bme.read_pressure(&mut i2c);

        // Read register with generic register read
        let reg_address = 0xD0;
        let reg_val = chip::reg_read(&mut i2c, 0x76, 0xD0).expect("Unable to read register");

        info!("reg_address: 0x{:.02X}, reg_val: 0x{:.02X}", reg_address, reg_val);
        rprintln!();
    }
}