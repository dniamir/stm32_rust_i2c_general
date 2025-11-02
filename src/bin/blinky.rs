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

use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*};
use rtt_target::{rtt_init_log, rprintln};
use log::{info, LevelFilter};

use stm32h7xx_hal::hal::blocking::delay::DelayMs;
use stm32h7xx_hal::hal::digital::v2::OutputPin;

// Structs
// Structs are basically classes
struct Led<PIN, D> {
    pin: PIN,
    delay: D,
}

impl<PIN, D> Led<PIN, D>
where
    PIN: OutputPin,
    D: DelayMs<u16>,
{
    fn new(pin: PIN, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Blinks the LED by setting it high, waiting for `ms` milliseconds,
    /// setting it low, and waiting for `ms` milliseconds again.
    ///
    /// # Arguments
    ///
    /// * `ms`: The number of milliseconds to wait between each blink.
    fn blink(&mut self, ms: u16) {
        self.pin.set_high().ok();
        self.delay.delay_ms(ms);
        self.pin.set_low().ok();
        self.delay.delay_ms(ms);
    }
}


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
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    // LED class
    let led_pin = gpioe.pe1.into_push_pull_output();
    let delay = cp.SYST.delay(ccdr.clocks);
    let mut led = Led::new(led_pin, delay);

    // Loop variable
    let mut count = 0;

    info!("Start Loop...");
    rprintln!();

    loop {
        led.blink(1000);

        // Smaller loop for the print statmements
        count += 1;
        for x in 0..3 {
            rprintln!("Outer Loop: {}, Inner Loop: {}", count, x);
        }
        rprintln!();
    }
}