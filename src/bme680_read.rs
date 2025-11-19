// Basic RUST tutorial

// #![deny(warnings)]
#![no_main]
#![no_std]

use rust_general::led::Led;
use rust_general::chip::Chip;
use rust_general::bme680::BME680;

use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*};
use rtt_target::{rtt_init_log, rprintln};
use log::{info, LevelFilter};

use shared_bus;

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
    let i2c = dp.I2C1.i2c((scl, sda), 50.kHz(), ccdr.peripheral.I2C1, &ccdr.clocks);
    let i2c_manager = shared_bus::BusManagerSimple::new(i2c);  // This is the shared bus manager

    // LED class
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let led_pin = gpioe.pe1.into_push_pull_output();
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let mut led = Led::new(led_pin);

    // Set up BME680
    // 🔹 Probe for the chip
    let bme_address = 0x76;
    let bme_chip = Chip{i2c: i2c_manager.acquire_i2c(), i2c_addr: bme_address, _map: core::marker::PhantomData};
    let mut bme = BME680::new(bme_chip).expect("failed to init bme");
    bme.config(1).expect("Unable to configure BME680");

    // Start loop
    info!("Start Loop...");
    let mut count = 0;
    rprintln!();

    loop {

        count += 1;
        info!("Count: {}", count);

        led.blink(&mut delay, 1000);

        // let _pressure = bme.read_pressure(&mut i2c);

        // Read register with generic register read
        let _field_val1 = bme.chip.read_field("chip_id").expect("Unable to read register");
        let _field_val2 = bme.chip.read_reg(0xD0).expect("Unable to read register");

        let _temperature = bme.read_temperature().expect("Unable to read temperature");

        rprintln!();

    }
}