use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::hal::blocking::delay::DelayMs;

// Represents an LED connected to a pin and delay provider.
// Structs are basically classes
pub struct Led<PIN, D> {
    pin: PIN,
    delay: D,
}

impl<PIN, D> Led<PIN, D>
where
    PIN: OutputPin,
    D: DelayMs<u16>,
{
    /// Blinks the LED once for the given duration.
    pub fn new(pin: PIN, delay: D) -> Self {
        Self { pin, delay }
    }

    pub fn blink(&mut self, ms: u16) {
        self.pin.set_high().ok();
        self.delay.delay_ms(ms);
        self.pin.set_low().ok();
        self.delay.delay_ms(ms);
    }
}