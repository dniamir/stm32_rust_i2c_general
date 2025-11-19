// src/led.rs
use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::hal::blocking::delay::DelayMs;

pub struct Led<P>
where
    P: OutputPin,
{
    pin: P,
}

impl<P> Led<P>
where
    P: OutputPin,
{
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    // pass delay in here
    pub fn blink<D>(&mut self, delay: &mut D, ms: u32)
    where
        D: DelayMs<u32>,
    {
        let _ = self.pin.set_high();
        delay.delay_ms(ms);
        let _ = self.pin.set_low();
        delay.delay_ms(ms);
    }
}