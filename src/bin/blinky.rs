//! Basic example that produces a 1Hz square-wave on Pin PE1

// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_reset as _;
use stm32h7xx_hal::{pac, prelude::*, i2c::I2c};
use rtt_target::{rtt_init_print, rprintln};


#[entry]
fn main() -> ! {

    // Initialize printing
    rtt_init_print!();
    rprintln!("Program start");

    let cp = cortex_m::Peripherals::take().unwrap();  // cp is the core peripherals
    let dp = pac::Peripherals::take().unwrap();  // dp is the device peripherals

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.ldo().freeze();

    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    // Configure PE1 as output.
    let mut led = gpioe.pe1.into_push_pull_output();

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    loop {
        rprintln!("Test");
        delay.delay_ms(1000_u16);
    }

    // // Configure PB8 and PB9 as I2C.
    // let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    // gpiob.pb8.into_alternate::<4>().set_open_drain();
    // gpiob.pb9.into_alternate::<4>().set_open_drain();
    // let mut i2c = I2c::i2c1(dp.I2C1, 400.kHz(), ccdr.peripheral.I2C1, &ccdr.clocks);
    
    // // Write the register address we want to read from
    // let register: u8 = 0xD0;
    // let mut buffer: [u8; 1] = [0];

    // // Main loop
    // loop {
    //     // Perform the read operation
    //     match i2c.write_read(0x76, &[register], &mut buffer) {
    //         Ok(_) => {
    //             let value = buffer[0];
    //             if value != 0 {
    //                 led.set_high();
    //                 rprintln!("It worked!: {}", buffer[0]);
    //             } else {
    //                 led.set_low();
    //                 rprintln!("It didn't work!: {}", buffer[0]);
    //             }
    //             // Add a small delay between reads
    //             delay.delay_ms(100_u16);
    //         }
    //         Err(e) => {
    //             // Error handling with fast blinking
    //             for _ in 0..5 {
    //                 led.set_high();
    //                 delay.delay_ms(100_u16);
    //                 led.set_low();
    //                 delay.delay_ms(100_u16);
    //             }
    //         }
    //     }
    // }
}
