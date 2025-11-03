//! Basic example that produces a 1Hz square-wave on Pin PE1

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
use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*};
use rtt_target::{rtt_init_log, rprintln};
use log::{info, LevelFilter};

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
    let dummy_data = [0x00];

    // LED class
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let led_pin = gpioe.pe1.into_push_pull_output();
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let mut led = Led::new(led_pin);

    info!("Start Loop...");
    rprintln!();

    loop {
        led.blink(&mut delay, 1000);

        for addr in 0x03..=0x77 {

            // Match is like "switch" in python
            // Ok and Err are common across Rust as returns
            match i2c.write(addr, &dummy_data) {
                Ok(()) => info!("Found device at address 0x{:02X}", addr),
                Err(_) => info!("No device at address 0x{:02X}", addr),
            }

            // Delay to make sure the I2C bus is not overwhelmed
            // cortex_m::asm::delay(10_000);  // runs n CPU cycles. So if the clock is 100Mhz, this is 0.1ms
            delay.delay_ms(10u16);
        }
        rprintln!();
    }
}