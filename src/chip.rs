use embedded_hal::blocking::i2c;

use log::{self, info};

// Efficient map for register maps
use phf::Map;
use phf_macros::phf_map;

/// Define some error types
#[derive(Debug)]
pub enum I2CError<I2C: i2c::WriteRead> {
    NotFound,
    I2CError(I2C::Error),
}

pub struct Chip<I2C> {
    pub i2c: I2C,
    pub i2c_addr: u8,
}

impl<I2C: i2c::WriteRead> Chip<I2C> {

    pub fn read_reg(&mut self, reg: u8) -> Result<u8, I2CError<I2C>> {
        // Basic function to read registers
        let mut buf = [0];
        self.i2c.write_read(self.i2c_addr, &[reg], &mut buf).map_err(I2CError::I2CError)?;
        let reg_value = buf[0];

        info!("Read Register: 0x{:.02X}, {:08b}, 0x{:.02X}, {}", reg, reg_value, reg_value, reg_value);
        log::set_max_level(log::LevelFilter::Info);

        Ok(reg_value)
    }

    pub fn write_reg(&mut self, reg: u8, reg_val: u8) -> Result<(), I2CError<I2C>> {
        // Basic function to write registers
        let mut buf = [0];
        self.i2c.write_read(self.i2c_addr, &[reg, reg_val], &mut buf).map_err(I2CError::I2CError)?;

        info!("Write Register: 0x{:.02X}, {:08b}, 0x{:.02X}, {}", reg, reg_val, reg_val, reg_val);

        Ok(())
    }

    pub fn read_field(&mut self, field: &str) -> Result<u8, I2CError<I2C>> {
        // Basic function to read a field, within a register
        // Will use a lookup table based on the field name
        
        // Get field details
        let field_dets = get_field(field).ok_or(I2CError::NotFound)?;
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
        // Basic function to write fields, within a register
        // Will use a lookup table based on the field name

        // Get field details
        let field_dets = get_field(field).ok_or(I2CError::NotFound)?;
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
        self.write_reg(field_reg, field_val)?;

        info!("Write Field: {}, {:0width$b}, 0x{:.02X}, {}", field, field_val, field_val, field_val, width=field_bits as usize);

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Field {
    pub reg: u8,
    pub offset: u8,
    pub bits: u8,
    pub writable: bool,
}

fn get_field(name: &str) -> Option<&'static Field> {
    FIELD_MAP.get(name)
}

pub static FIELD_MAP: Map<&'static str, Field> = phf_map! {
    "status" => Field { reg: 0x73, offset: 0, bits: 8, writable: true },
    "reset" => Field { reg: 0xe0, offset: 0, bits: 8, writable: true },
    "Id" => Field { reg: 0xd0, offset: 0, bits: 8, writable: false },
    "chip_id" => Field { reg: 0xd0, offset: 0, bits: 8, writable: false },
    "Config" => Field { reg: 0x75, offset: 0, bits: 8, writable: true },
    "filter" => Field { reg: 0x75, offset: 2, bits: 3, writable: true },
    "ctrl_meas" => Field { reg: 0x74, offset: 0, bits: 8, writable: true },
    "osrs_t" => Field { reg: 0x74, offset: 5, bits: 3, writable: true },
    "osrs_p" => Field { reg: 0x74, offset: 2, bits: 3, writable: true },
    "mode" => Field { reg: 0x74, offset: 0, bits: 2, writable: true },
    "osrs_h" => Field { reg: 0x72, offset: 0, bits: 3, writable: true },
    "ctrl_gas_1" => Field { reg: 0x71, offset: 0, bits: 8, writable: true },
    "ctrl_gas_0" => Field { reg: 0x70, offset: 4, bits: 2, writable: true },
    "run_gas" => Field { reg: 0x71, offset: 4, bits: 1, writable: true },
    "nb_conv" => Field { reg: 0x71, offset: 0, bits: 4, writable: true },
    "heat_off" => Field { reg: 0x70, offset: 3, bits: 1, writable: true },
    "gas_wait_9" => Field { reg: 0x6d, offset: 0, bits: 8, writable: true },
    "gas_wait_8" => Field { reg: 0x6c, offset: 0, bits: 8, writable: true },
    "gas_wait_7" => Field { reg: 0x6b, offset: 0, bits: 8, writable: true },
    "gas_wait_6" => Field { reg: 0x6a, offset: 0, bits: 8, writable: true },
    "gas_wait_5" => Field { reg: 0x69, offset: 0, bits: 8, writable: true },
    "gas_wait_4" => Field { reg: 0x68, offset: 0, bits: 8, writable: true },
    "gas_wait_3" => Field { reg: 0x67, offset: 0, bits: 8, writable: true },
    "gas_wait_2" => Field { reg: 0x66, offset: 0, bits: 8, writable: true },
    "gas_wait_1" => Field { reg: 0x65, offset: 0, bits: 8, writable: true },
    "gas_wait_0" => Field { reg: 0x64, offset: 0, bits: 8, writable: true },
    "res_heat_9" => Field { reg: 0x63, offset: 0, bits: 8, writable: true },
    "res_heat_8" => Field { reg: 0x62, offset: 0, bits: 8, writable: true },
    "res_heat_7" => Field { reg: 0x61, offset: 0, bits: 8, writable: true },
    "res_heat_6" => Field { reg: 0x60, offset: 0, bits: 8, writable: true },
    "res_heat_5" => Field { reg: 0x5f, offset: 0, bits: 8, writable: true },
    "res_heat_4" => Field { reg: 0x5e, offset: 0, bits: 8, writable: true },
    "res_heat_3" => Field { reg: 0x5d, offset: 0, bits: 8, writable: true },
    "res_heat_2" => Field { reg: 0x5c, offset: 0, bits: 8, writable: true },
    "res_heat_1" => Field { reg: 0x5b, offset: 0, bits: 8, writable: true },
    "res_heat_0" => Field { reg: 0x5a, offset: 0, bits: 8, writable: true },

    "gas_r_lsb" => Field { reg: 0x2b, offset: 0, bits: 8, writable: false },
    "gas_range_r" => Field { reg: 0x2b, offset: 0, bits: 4, writable: false },
    "heat_stab_r" => Field { reg: 0x2b, offset: 4, bits: 1, writable: false },
    "gas_valid_r" => Field { reg: 0x2b, offset: 5, bits: 1, writable: false },

    "gas_r_msb" => Field { reg: 0x2a, offset: 0, bits: 8, writable: false },
    "hum_lsb" => Field { reg: 0x26, offset: 0, bits: 8, writable: false },
    "hum_msb" => Field { reg: 0x25, offset: 0, bits: 8, writable: false },
    "temp_xlsb" => Field { reg: 0x24, offset: 4, bits: 4, writable: false },
    "temp_lsb" => Field { reg: 0x23, offset: 0, bits: 8, writable: false },
    "temp_msb" => Field { reg: 0x22, offset: 0, bits: 8, writable: false },
    "press_xlsb" => Field { reg: 0x21, offset: 4, bits: 4, writable: false },
    "press_lsb" => Field { reg: 0x20, offset: 0, bits: 8, writable: false },
    "press_msb" => Field { reg: 0x1f, offset: 0, bits: 8, writable: false },

    "par_t1" => Field { reg: 0xe9, offset: 0, bits: 8, writable: false },
    "par_t2" => Field { reg: 0x8a, offset: 0, bits: 8, writable: false },
    "par_t3" => Field { reg: 0x8c, offset: 0, bits: 8, writable: false },
    "par_p1" => Field { reg: 0x8e, offset: 0, bits: 8, writable: false },
    "par_p2" => Field { reg: 0x90, offset: 0, bits: 8, writable: false },
    "par_p3" => Field { reg: 0x92, offset: 0, bits: 8, writable: false },
    "par_p4" => Field { reg: 0x94, offset: 0, bits: 8, writable: false },
    "par_p5" => Field { reg: 0x96, offset: 0, bits: 8, writable: false },
    "par_p6" => Field { reg: 0x99, offset: 0, bits: 8, writable: false },
    "par_p7" => Field { reg: 0x98, offset: 0, bits: 8, writable: false },
    "par_p8" => Field { reg: 0x9c, offset: 0, bits: 8, writable: false },
    "par_p9" => Field { reg: 0x9e, offset: 0, bits: 8, writable: false },
    "par_p10" => Field { reg: 0xa0, offset: 0, bits: 8, writable: false },
    "par_h1" => Field { reg: 0xe2, offset: 0, bits: 8, writable: false },
    "par_h2" => Field { reg: 0xe1, offset: 0, bits: 8, writable: false },
    "par_h3" => Field { reg: 0xe4, offset: 0, bits: 8, writable: false },
    "par_h4" => Field { reg: 0xe5, offset: 0, bits: 8, writable: false },
    "par_h5" => Field { reg: 0xe6, offset: 0, bits: 8, writable: false },
    "par_h6" => Field { reg: 0xe7, offset: 0, bits: 8, writable: false },
    "par_h7" => Field { reg: 0xe8, offset: 0, bits: 8, writable: false },
    "par_g1" => Field { reg: 0xed, offset: 0, bits: 8, writable: false },
    "par_g2" => Field { reg: 0xeb, offset: 0, bits: 8, writable: false },
    "par_g3" => Field { reg: 0xee, offset: 0, bits: 8, writable: false },
    "res_heat_range" => Field { reg: 0x02, offset: 4, bits: 2, writable: false },
    "res_heat_val" => Field { reg: 0x00, offset: 0, bits: 8, writable: false },
    "range_switching_error" => Field { reg: 0x04, offset: 0, bits: 8, writable: false },

};