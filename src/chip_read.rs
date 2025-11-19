// Basic RUST tutorial
// #![deny(warnings)]
#![no_main]
#![no_std]

use rust_general::led::Led;
use rust_general::chip::Chip;

use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*};
use rtt_target::{rtt_init_log, rprintln};
use log::{info, LevelFilter};

use shared_bus;

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

    // Set up for generic chip
    let bme_address = 0x76;
    let mut chip = Chip::new_generic(i2c_manager.acquire_i2c(), bme_address);

    // Start loop
    info!("Start Loop...");
    let mut count = 0;
    rprintln!();

    loop {

        count += 1;
        info!("Count: {}", count);

        led.blink(&mut delay, 1000);

        // Read register with generic register read
        let _field_val2 = chip.read_reg(0xD0).expect("Unable to read register");

        rprintln!();

        chip.write_reg(0x74, 0b11100011).expect("Unable to read register");
        chip.read_reg(0x74).expect("Unable to read register");

        rprintln!();

        chip.write_reg(0x74, 0b00011100).expect("Unable to read register");
        chip.read_reg(0x74).expect("Unable to read register");

        rprintln!();

        let reg_vals = &mut [0u8; 4];
        chip.read_regs(0x74, reg_vals).expect("Unable to read register");

        rprintln!();

    }
}