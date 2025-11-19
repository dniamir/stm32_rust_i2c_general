use embedded_hal::blocking::i2c;
use core::marker::PhantomData;
use log::{self, info};
use crate::chip_map;

/// Define some error types
#[derive(Debug)]
pub enum I2CError<I2C: i2c::WriteRead> {
    NotFound,
    I2CError(I2C::Error),
}

pub struct Chip<I2C, MAP=chip_map::NoFieldMap> {
    pub i2c: I2C,
    pub i2c_addr: u8,
    pub _map: PhantomData<MAP>,
}

impl<I2C> Chip<I2C, chip_map::NoFieldMap>
where
    I2C: i2c::WriteRead,
{
    pub fn new_generic(i2c: I2C, addr: u8) -> Self {
        Self { i2c, i2c_addr: addr, _map: PhantomData }
    }
}

impl<I2C, MAP> Chip<I2C, MAP>
where
    I2C: i2c::WriteRead,
{

    pub fn read_regs(&mut self, reg: u8, reg_values: &mut [u8]) -> Result<(), I2CError<I2C>> {
        // Basic function to read multiple registers
        self.i2c.write_read(self.i2c_addr, &[reg], reg_values).map_err(I2CError::I2CError)?;

        let mut reg_idx = 0;
        for reg_value in reg_values.iter() {
            info!("Read Register: 0x{:.02X}, {:08b}, 0x{:.02X}, {}", reg + reg_idx, reg_value, reg_value, reg_value);
            reg_idx += 1;
        }

        Ok(())
    }

    pub fn write_reg(&mut self, reg: u8, reg_val: u8) -> Result<(), I2CError<I2C>> {
        // Basic function to write registers by numerical address
        let mut buf = [0];
        self.i2c.write_read(self.i2c_addr, &[reg, reg_val], &mut buf).map_err(I2CError::I2CError)?;

        info!("Write Register: 0x{:.02X}, {:08b}, 0x{:.02X}, {}", reg, reg_val, reg_val, reg_val);

        Ok(())
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u8, I2CError<I2C>> {
        // Basic function to read registers by numerical address
        let mut reg_vals = [0];
        let old_level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        self.read_regs(reg, &mut reg_vals)?;
        log::set_max_level(old_level);

        let reg_value = reg_vals[0];
        info!("Read Register: 0x{:.02X}, {:08b}, 0x{:.02X}, {}", reg, reg_value, reg_value, reg_value);

        Ok(reg_value)
    }
}

impl<I2C, MAP> Chip<I2C, MAP>
where
    I2C: i2c::WriteRead,
    MAP: chip_map::FieldMapProvider,
{

    pub fn read_regs_str(&mut self, reg_str: &str, reg_values: &mut [u8]) -> Result<(), I2CError<I2C>> {
        // Basic function to read multiple registers
        let reg_dets = MAP::get_field(reg_str).ok_or(I2CError::NotFound)?;

        // Just read the raw register value
        // let old_level = log::max_level();
        // log::set_max_level(log::LevelFilter::Off);
        self.read_regs(reg_dets.reg, reg_values)?;
        // log::set_max_level(old_level);

        // let reg_value = reg_values[0];
        // info!("Read Register: {}, {:08b}, 0x{:.02X}, {}", reg_str, reg_value, reg_value, reg_value);

        Ok(())
    }

    pub fn read_reg_str(&mut self, reg_str: &str) -> Result<u8, I2CError<I2C>> {
        // Basic function to read registers by name
        let reg_dets = MAP::get_field(reg_str).ok_or(I2CError::NotFound)?;

        // Just read the raw register value
        let old_level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        let reg_value = self.read_reg(reg_dets.reg)?;
        log::set_max_level(old_level);

        info!("Read Register: {}, {:08b}, 0x{:.02X}, {}", reg_str, reg_value, reg_value, reg_value);

        Ok(reg_value)
    }

    pub fn write_reg_str(&mut self, reg_str: &str, reg_val: u8) -> Result<(), I2CError<I2C>> {
        // Basic function to write registers by name
        let reg_dets = MAP::get_field(reg_str).ok_or(I2CError::NotFound)?;

        // Write the register
        let old_level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        self.write_reg(reg_dets.reg, reg_val)?;
        log::set_max_level(old_level);

        info!("Write Register: {}, {:08b}, 0x{:.02X}, {}", reg_str, reg_val, reg_val, reg_val);

        Ok(())
    }

    pub fn read_field(&mut self, field: &str) -> Result<u8, I2CError<I2C>> {
        // Basic function to read a field by name, within a register
        // Will use a lookup table based on the field name
        
        // Get field details
        let field_dets = MAP::get_field(field).ok_or(I2CError::NotFound)?;
        let field_reg: u8 = field_dets.reg as u8;
        let field_offset: u8 = field_dets.offset as u8;
        let field_bits: u8 = field_dets.bits as u8;

        // Read register
        let old_level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        let reg_val = self.read_reg(field_reg as u8)?;
        log::set_max_level(old_level);

        // Create mask and get value
        let mask = (((1u32 << field_bits) - 1) << field_offset) as u8;
        let field_val = (reg_val & mask) >> field_offset;

        info!("Read Field: {}, {:0width$b}, 0x{:.02X}, {}", field, field_val, field_val, field_val, width=field_bits as usize);

        Ok(field_val)
    }

    pub fn write_field(&mut self, field: &str, field_val: u8) -> Result<(), I2CError<I2C>> {
        // Basic function to write a field by name, within a register
        // Will use a lookup table based on the field name

        // Get field details
        let field_dets = MAP::get_field(field).ok_or(I2CError::NotFound)?;
        let field_reg: u8 = field_dets.reg as u8;
        let field_offset: u8 = field_dets.offset as u8;
        let field_bits: u8 = field_dets.bits as u8;

        let curr_field_val = self.read_reg(field_reg)?;

        // Make mask of field_bits starting at field_offset
        let mask = ((1u32 << field_bits) - 1) << field_offset;

        // Clear the bits in the current value
        let cleared = (curr_field_val as u32) & !mask;

        // Insert field_val into the correct position
        let inserted = ((field_val as u32) << field_offset) & mask;
        let field_val = (cleared | inserted) as u8;   // back to u8
    
        // Write register
        let old_level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        self.write_reg(field_reg, field_val)?;
        log::set_max_level(old_level);

        info!("Write Field: {}, {:0width$b}, 0x{:.02X}, {}", field, field_val, field_val, field_val, width=field_bits as usize);

        Ok(())
    }
}
