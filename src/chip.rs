use core::marker::PhantomData;
use embedded_hal::blocking::i2c;

/// BME680 pressure sensor
///
/// Datasheet: https://cdn.sparkfun.com/assets/8/a/1/c/f/BME680-Datasheet.pdf
pub struct BME680<I2C: i2c::WriteRead> {
    addr: u8,
    _i2c: PhantomData<I2C>,
}

/// Define some error types
#[derive(Debug)]
pub enum BME680Error<I2C: i2c::WriteRead> {
    NotFound,
    I2CError(I2C::Error),
}

impl<I2C: i2c::WriteRead> BME680<I2C> {
    /// Initialize the driver by probing the I2C bus.
    pub fn probe(i2c: &mut I2C) -> Result<Self, BME680Error<I2C>> {
        // Try both addresses in the datasheet
        for addr in [0x76, 0x77] {
            // Per the datasheet, the ID register (0xD0) should contain the chip_id
            // value 0x61
            match reg_read(i2c, addr, 0xD0) {
                Ok(0x61) => {
                    // We found the chip, so construct and return an instance of this object.
                    // The object can be used to read/write to the chip.
                    let mut bme = Self {
                        addr,
                        _i2c: PhantomData {},
                    };

                    // Configure the chip
                    bme.config(i2c)?;

                    // Enable the chip
                    bme.set_enabled(i2c, true)?;

                    return Ok(bme);
                }
                _ => continue,
            }
        }

        return Err(BME680Error::NotFound);
    }

    /// Configure basic registers
    pub fn config(&mut self, _i2c: &mut I2C) -> Result<(), BME680Error<I2C>> {
        // TODO: write registers per the datasheet
        // reg_write(i2c, self.addr, 0x74, 0x24);

        Ok(())
    }

    /// Configure basic registers
    pub fn set_enabled(&mut self, i2c: &mut I2C, enabled: bool) -> Result<(), BME680Error<I2C>> {
        // First read previous value in register
        let ctrl_gas1 = reg_read(i2c, self.addr, 0x71)?;
        // Then write to register, setting enabled correctly
        reg_write(i2c,self.addr, 0x71, (ctrl_gas1 & !0x10) | (if enabled { 0x10 } else { 0x00 }),)?;

        Ok(())
    }

    pub fn read_pressure(&mut self, i2c: &mut I2C) -> Result<u32, BME680Error<I2C>> {
        // We want to read 3 consecutive bytes in one operation
        let mut buf = [0x00, 0x00, 0x00];
        i2c.write_read(self.addr, &[0x1F], &mut buf)
            .map_err(|e| BME680Error::I2CError(e))?;

        // Restructure the bytes into a single u32
        let val = ((buf[0] as u32) << 12) | ((buf[1] as u32) << 4) | ((buf[2] as u32) >> 4);

        Ok(val)
    }
}

/// Perform a I2C read
pub fn reg_read<I2C: i2c::WriteRead>(i2c: &mut I2C, addr: u8, reg: u8) -> Result<u8, BME680Error<I2C>> {
    let mut buf = [0x00];
    i2c.write_read(addr, &[reg], &mut buf).map_err(|e| BME680Error::I2CError(e))?;
    Ok(buf[0])
}

/// Perform a I2C write
pub fn reg_write<I2C: i2c::WriteRead>(i2c: &mut I2C, addr: u8, reg: u8, val: u8) -> Result<(), BME680Error<I2C>> {
    let mut buf = [val];
    i2c.write_read(addr, &[reg], &mut buf).map_err(|e| BME680Error::I2CError(e))?;
    Ok(())
}