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
        log::warn!("attempted to load from  unimplemented SRAM; address=0x{address:08X}; value=0x{value:02X}");
        (value, self.sram_waitstates)
    }

    pub(super) fn store32_sram(
        &mut self,
        address: u32,
        value: u32,
        _access: AccessType,
    ) -> Waitstates {
        log::warn!("attempted to store to unimplemented SRAM; address=0x{address:08X}; value=0x{value:08X}",);
        self.sram_waitstates
    }

    pub(super) fn store16_sram(
        &mut self,
        address: u32,
        value: u16,
        _access: AccessType,
    ) -> Waitstates {
        log::warn!("attempted to store to unimplemented SRAM; address=0x{address:08X}; value=0x{value:04X}",);
        self.sram_waitstates
    }

    pub(super) fn store8_sram(
        &mut self,
        address: u32,
        value: u8,
        _access: AccessType,
    ) -> Waitstates {
        log::warn!("attempted to store to unimplemented SRAM; address=0x{address:08X}; value=0x{value:02X}",);
        self.sram_waitstates
    }
}
