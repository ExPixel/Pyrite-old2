use arm::Memory;
use byteorder::{ByteOrder, LittleEndian};

pub struct TestMemory {
    data: Vec<u8>,
    len_no_padding: usize,
}

impl TestMemory {
    // pub fn new(data: Vec<u8>) -> Self {
    //     TestMemory {
    //         len_no_padding: data.len(),
    //         data,
    //     }
    // }

    pub fn with_padding(mut data: Vec<u8>, min_len: usize) -> Self {
        let len_no_padding = data.len();
        data.resize(min_len, 0xc0);
        TestMemory {
            data,
            len_no_padding,
        }
    }

    pub fn set_memory_with_padding(&mut self, mut data: Vec<u8>, min_len: usize) {
        let len_no_padding = data.len();
        data.resize(min_len, 0xc0);
        self.data = data;
        self.len_no_padding = len_no_padding;
    }

    pub fn view32(&mut self, address: u32) -> u32 {
        LittleEndian::read_u32(&self.data[address as usize..])
    }

    pub fn view16(&mut self, address: u32) -> u16 {
        LittleEndian::read_u16(&self.data[address as usize..])
    }
}

impl Memory for TestMemory {
    fn load32(&mut self, address: u32, _access: arm::AccessType) -> (u32, arm::Waitstates) {
        let data = LittleEndian::read_u32(&self.data[address as usize..]);
        (data, 0u8.into())
    }

    fn load16(&mut self, address: u32, _access: arm::AccessType) -> (u16, arm::Waitstates) {
        let data = LittleEndian::read_u16(&self.data[address as usize..]);
        (data, 0u8.into())
    }

    fn load8(&mut self, address: u32, _access: arm::AccessType) -> (u8, arm::Waitstates) {
        (self.data[address as usize], 0u8.into())
    }

    fn store32(&mut self, address: u32, value: u32, _access: arm::AccessType) -> arm::Waitstates {
        LittleEndian::write_u32(&mut self.data[address as usize..], value);
        0u8.into()
    }

    fn store16(&mut self, address: u32, value: u16, _access: arm::AccessType) -> arm::Waitstates {
        LittleEndian::write_u16(&mut self.data[address as usize..], value);
        0u8.into()
    }

    fn store8(&mut self, address: u32, value: u8, _access: arm::AccessType) -> arm::Waitstates {
        self.data[address as usize] = value;
        0u8.into()
    }
}
