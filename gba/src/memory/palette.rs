use byteorder::{ByteOrder as _, LittleEndian as LE};
use util::array;

use super::{PAL_MASK, PAL_SIZE};

pub struct Palette {
    pub(crate) data: Box<[u8; PAL_SIZE as usize]>,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            data: array::boxed_copied(0),
        }
    }
}

impl Palette {
    pub fn load32(&self, address: u32) -> u32 {
        LE::read_u32(&self.data[(address & PAL_MASK) as usize..])
    }

    pub fn load16(&self, address: u32) -> u16 {
        LE::read_u16(&self.data[(address & PAL_MASK) as usize..])
    }

    pub fn load8(&self, address: u32) -> u8 {
        self.data[(address & PAL_MASK) as usize]
    }

    pub fn store32(&mut self, address: u32, value: u32) {
        LE::write_u32(&mut self.data[(address & PAL_MASK) as usize..], value);
    }

    pub fn store16(&mut self, address: u32, value: u16) {
        LE::write_u16(&mut self.data[(address & PAL_MASK) as usize..], value);
    }

    pub fn store8(&mut self, address: u32, value: u8) {
        // 8bit writes to PAL write the 8bit value to both the lower and upper byte of
        // the addressed halfword.
        let address = ((address & !0x1) & PAL_MASK) as usize;
        self.data[address] = value;
        self.data[address + 1] = value;
    }

    pub fn view32(&self, address: u32) -> u32 {
        LE::read_u32(&self.data[(address & PAL_MASK) as usize..])
    }

    pub fn view16(&self, address: u32) -> u16 {
        LE::read_u16(&self.data[(address & PAL_MASK) as usize..])
    }

    pub fn view8(&self, address: u32) -> u8 {
        self.data[(address & PAL_MASK) as usize]
    }
}
