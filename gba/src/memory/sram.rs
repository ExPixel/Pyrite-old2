use super::GbaMemory;
use arm::{AccessType, Waitstates};

impl GbaMemory {
    pub(super) fn load32_sram(&mut self, address: u32, access: AccessType) -> (u32, Waitstates) {
        let (value, waitstates) = self.load8_sram(address, access);
        (value as u32 * 0x01010101, waitstates) // return repeated byte
    }

    pub(super) fn load16_sram(&mut self, address: u32, access: AccessType) -> (u16, Waitstates) {
        let (value, waitstates) = self.load8_sram(address, access);
        (value as u16 * 0x0101, waitstates) // return repeated byte
    }

    pub(super) fn load8_sram(&mut self, address: u32, _access: AccessType) -> (u8, Waitstates) {
        // FIXME implement real Flash IDs

        // These are some default values that Pokemon works with
        const MANUFACTURER: u8 = 0xC2;
        const DEVICE: u8 = 0x09;

        let value = match address {
            0xE000000 => MANUFACTURER,
            0xE000001 => DEVICE,
            _ => 0,
        };
        unimplemented_sram_load();
        (value, self.sram_waitstates)
    }

    pub(super) fn store32_sram(
        &mut self,
        _address: u32,
        _value: u32,
        _access: AccessType,
    ) -> Waitstates {
        unimplemented_sram_store();
        self.sram_waitstates
    }

    pub(super) fn store16_sram(
        &mut self,
        _address: u32,
        _value: u16,
        _access: AccessType,
    ) -> Waitstates {
        unimplemented_sram_store();
        self.sram_waitstates
    }

    pub(super) fn store8_sram(
        &mut self,
        _address: u32,
        _value: u8,
        _access: AccessType,
    ) -> Waitstates {
        unimplemented_sram_store();
        self.sram_waitstates
    }
}

fn unimplemented_sram_load() {
    static mut CALLED: bool = false;
    if unsafe { !CALLED } {
        unsafe { CALLED = true };
        log::warn!("unimplemented SRAM load");
    }
}

fn unimplemented_sram_store() {
    static mut CALLED: bool = false;
    if unsafe { !CALLED } {
        unsafe { CALLED = true };
        log::warn!("unimplemented SRAM store");
    }
}
