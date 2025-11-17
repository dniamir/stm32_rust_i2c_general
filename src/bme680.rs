use embedded_hal::blocking::i2c;

use log::{self, info};

use heapless::String;
use core::fmt::Write;

use crate::chip::Chip;
use crate::chip::I2CError;
use crate::chip_map::{Field, FieldMapProvider};

pub struct BME680<I2C> {
    pub chip: Chip<I2C>,
    pub cal_codes: CalCodes,
    pub temp_comp: i32,
    pub t_fine: i32,
}

#[derive(Default)]
pub struct CalCodes {
    // Pressure
    pub par_p10: u8,
    pub par_p9: i16,
    pub par_p8: i16,
    pub par_p7: i8,
    pub par_p6: i8,
    pub par_p5: i16,
    pub par_p4: i16,
    pub par_p3: i8,
    pub par_p2: i16,
    pub par_p1: u16,

    // Temperature
    pub par_t3: i16,
    pub par_t2: i16,
    pub par_t1: u16,

    // Humidity
    pub par_h7: i8,
    pub par_h6: u8,
    pub par_h5: i8,
    pub par_h4: i8,
    pub par_h3: i8,
    pub par_h2: u16,
    pub par_h1: u16,

    // Gas
    pub par_g3: i8,
    pub par_g2: i16,
    pub par_g1: i8,

    // Misc
    pub res_heat_range: i8,
    pub res_heat_val: i8,
    pub gas_adc: i16,
    pub gas_range: i8,
    pub range_switching_error: i8,
}

impl<I2C: i2c::WriteRead> BME680<I2C> {
    pub fn new(chip: Chip<I2C>) -> Result<Self, I2CError<I2C>> {
        let mut this = Self {
            chip,
            cal_codes: CalCodes::default(), // ← created here
            temp_comp: 0,
            t_fine: 0,
        };

        this.read_cal_codes()?;

        Ok(this)
    }

    pub fn config(&mut self, profile_num: u8) -> Result<(), I2CError<I2C>> {

        // Other Sensor Settings
        self.chip.write_field("osrs_h", 0b101)?;  // 16x oversampling
        self.chip.write_field("osrs_t", 0b101)?;  // 16x oversampling
        self.chip.write_field("osrs_p", 0b101)?;  // 16x oversampling
        self.chip.write_field("filter", 0b010)?;  // Filter coefficient of 3 - form of averaging filter

        // Gas Sensor Settings
        self.chip.write_field("gas_range_r", 4)?; // Set Gas Range
        self.chip.write_field("run_gas", 0b1)?; // Turn on Gas Sensor
        self.chip.write_field("nb_conv", profile_num)?; // Set Heater profile to profile 0

        // Set time between beginning of the heat phase and start of resistance conversion
        self.set_gas_wait(0b00011110, profile_num)?; // This should be 30ms

        // Set heater temperature
        self.set_heater_temp(300, profile_num)?;  // Set heater profile 0 to 300C

        Ok(())
    }

    pub fn set_gas_wait(&mut self, wait_time_ms: u8, profile_num: u8) -> Result<(), I2CError<I2C>> {
        let mut buf: String<16> = String::new();
        write!(buf, "gas_wait_{}", profile_num).unwrap();   
        self.chip.write_field(&buf, wait_time_ms)
    }

    pub fn set_heater_temp(&mut self, target_temp: i16, profile_num: u8) -> Result<(), I2CError<I2C>> {

        // --- Get calibration values ---
        let par_g1 = self.cal_codes.par_g1;
        let par_g2 = self.cal_codes.par_g2;
        let par_g3 = self.cal_codes.par_g3;

        // --- Ensure temperature compensation is available ---
        if self.temp_comp == 0 {self.read_temperature()?;}
        let amb_temp = (self.temp_comp / 100) as i32;

        // --- Read intermediates ---
        let res_heat_range = self.chip.read_field("res_heat_range")? as i32;
        let res_heat_val = self.chip.read_field("res_heat_val")? as i32;

        // --- Calculate heater resistance ---
        let var1 = (((amb_temp * par_g3 as i32) / 10) << 8) as i32;
        let var2 = (par_g1 as i32 + 784)* (((((par_g2 as i32 + 154_009) * target_temp as i32 * 5) / 100) + 3_276_800) / 10);
        let var3 = var1 + (var2 >> 1);
        let var4 = var3 / (res_heat_range + 4);
        let var5 = 131 * res_heat_val + 65_536;
        let res_heat_x100 = ((var4 / var5) - 250) * 34;
        let res_heat_x = ((res_heat_x100 + 50) / 100) as u8;

        // Format field name and write
        let mut buf: String<16> = String::new();
        write!(buf, "res_heat_{}", profile_num).unwrap();   
        self.chip.write_field(&buf, res_heat_x)
    }

    pub fn read_cal_codes(&mut self) -> Result<(), I2CError<I2C>> {
        let rf = |name: &str, this: &mut Self| this.chip.read_field(name);
        let rr = |reg: u8,  this: &mut Self| this.chip.read_reg(reg);

        // Temperature
        self.cal_codes.par_t1 =(rf("par_t1", self)? as u16) | ((rr(0xea, self)? as u16) << 8);
        self.cal_codes.par_t2 =(rf("par_t2", self)? as i16) | ((rr(0x8b, self)? as i16) << 8);
        self.cal_codes.par_t3 = rf("par_t3", self)? as i16;

        // Pressure
        self.cal_codes.par_p1 =(rf("par_p1", self)? as u16) | ((rr(0x8f, self)? as u16) << 8);
        self.cal_codes.par_p2 =(rf("par_p2", self)? as i16) | ((rr(0x91, self)? as i16) << 8);
        self.cal_codes.par_p3 = rf("par_p3", self)? as i8;
        self.cal_codes.par_p4 =(rf("par_p4", self)? as i16) | ((rr(0x95, self)? as i16) << 8);
        self.cal_codes.par_p5 =(rf("par_p5", self)? as i16) | ((rr(0x97, self)? as i16) << 8);
        self.cal_codes.par_p6 = rf("par_p6", self)? as i8;
        self.cal_codes.par_p7 = rf("par_p7", self)? as i8;
        self.cal_codes.par_p8 =(rf("par_p8", self)? as i16) | ((rr(0x9d, self)? as i16) << 8);
        self.cal_codes.par_p9 =(rf("par_p9", self)? as i16) | ((rr(0x9f, self)? as i16) << 8);
        self.cal_codes.par_p10 = rf("par_p10", self)?;

        // Humidity
        self.cal_codes.par_h1 =((rf("par_h1", self)? & 0x0F) as u16) | ((rr(0xe3, self)? as u16) << 4);
        self.cal_codes.par_h2 =((rf("par_h2", self)? as u16) << 4) | ((rr(0xe2, self)? as u16) >> 4);
        self.cal_codes.par_h3 = rf("par_h3", self)? as i8;
        self.cal_codes.par_h4 = rf("par_h4", self)? as i8;
        self.cal_codes.par_h5 = rf("par_h5", self)? as i8;
        self.cal_codes.par_h6 = rf("par_h6", self)?;
        self.cal_codes.par_h7 = rf("par_h7", self)? as i8;

        // Gas
        self.cal_codes.par_g1 = rf("par_g1", self)? as i8;
        self.cal_codes.par_g2 =(rf("par_g2", self)? as i16) | ((rr(0xec, self)? as i16) << 8);
        self.cal_codes.par_g3 = rf("par_g3", self)? as i8;

        Ok(())
    }

    pub fn read_temperature(&mut self) -> Result<(i32), I2CError<I2C>> {
        self.chip.write_field("mode", 0b01)?;

        let mut temp_out = [0u8; 3];
        self.chip.read_regs_str("temp_msb", &mut temp_out)?;

        // 20-bit ADC value
        let temp_adc: u32 =
            ((temp_out[0] as u32) << 12) |
            ((temp_out[1] as u32) << 4)  |
            ((temp_out[2] as u32) >> 4);

        let temp_comp = self.calibrate_temperature(temp_adc);

        // Log statement with decimal points
        let whole = temp_comp / 100;
        let frac  = temp_comp % 100;
        info!("Temperature: {}.{:02} °C", whole, frac);

        Ok(temp_comp)
    }

    pub fn calibrate_temperature(&mut self, temp_adc: u32) -> i32 {
        // Calibration constants
        let par_t1 = self.cal_codes.par_t1; // i16
        let par_t2 = self.cal_codes.par_t2; // i16
        let par_t3 = self.cal_codes.par_t3; // u16

        // Promote to i64 for intermediate math
        let var1 = ((temp_adc as i32 >> 3) - ((par_t1 as i32) << 1)) as i64;
        let var2 = ((var1 * par_t2 as i64) >> 11) as i64;
        let var3 = ((((var1 >> 1) * (var1 >> 1)) >> 12) * ((par_t3 as i64) << 4)) >> 14;

        let t_fine = (var2 + var3) as i32;
        let temp_comp = ((t_fine * 5 + 128) >> 8) as i32;

        // Save intermediate values
        self.t_fine = t_fine;
        self.temp_comp = temp_comp;

        temp_comp
    }

    // pub fn read_something(&mut self) -> Result<u8, BME680Error<I2C>> {
    //     // call Chip’s method through base
    //     self.base.read_reg(0x10)
    // }
}