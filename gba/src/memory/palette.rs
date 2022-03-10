use util::mem::{read_u16, read_u32, write_u16, write_u32};

use super::{PAL_MASK, PAL_SIZE};

pub struct Palette {
    pub(crate) data: [u8; PAL_SIZE as usize],
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            data: [0; PAL_SIZE as usize],
        }
    }
}

impl Palette {
    pub fn get_bg256(&self, entry: u8) -> u16 {
        self.view16((entry as u32) * 2)
    }

    pub fn get_obj256(&self, entry: u8) -> u16 {
        let addr = (entry as u32) * 2 + 0x200;
        self.view16(addr)
    }

    pub fn get_bg16(&self, palette: u8, entry: u8) -> u16 {
        self.get_bg256(palette * 16 + entry)
    }

    pub fn get_obj16(&self, palette: u8, entry: u8) -> u16 {
        self.get_obj256(palette * 16 + entry)
    }

    pub fn load32(&self, address: u32) -> u32 {
        read_u32(&self.data, (address & PAL_MASK) as usize)
    }

    pub fn load16(&self, address: u32) -> u16 {
        read_u16(&self.data, (address & PAL_MASK) as usize)
    }

    pub fn load8(&self, address: u32) -> u8 {
        self.data[(address & PAL_MASK) as usize]
    }

    pub fn store32(&mut self, address: u32, value: u32) {
        write_u32(&mut self.data, (address & PAL_MASK) as usize, value);
    }

    pub fn store16(&mut self, address: u32, value: u16) {
        write_u16(&mut self.data, (address & PAL_MASK) as usize, value);
    }

    pub fn store8(&mut self, address: u32, value: u8) {
        // 8bit writes to PAL write the 8bit value to both the lower and upper byte of
        // the addressed halfword.
        let address = ((address & !0x1) & PAL_MASK) as usize;
        self.data[address] = value;
        self.data[address + 1] = value;
    }

    pub fn view32(&self, address: u32) -> u32 {
        read_u32(&self.data, (address & PAL_MASK) as usize)
    }

    pub fn view16(&self, address: u32) -> u16 {
        read_u16(&self.data, (address & PAL_MASK) as usize)
    }

    pub fn view8(&self, address: u32) -> u8 {
        self.data[(address & PAL_MASK) as usize]
    }
}
