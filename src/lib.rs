#![no_std]

// Point modules into src/lib/...
#[path = "lib/chip.rs"]
pub mod chip;

#[path = "lib/chip_map.rs"]
pub mod chip_map;

#[path = "lib/bme680.rs"]
pub mod bme680;

#[path = "lib/led.rs"]
pub mod led;