use super::GbaMemory;
use arm::{AccessType, Waitstates};

impl GbaMemory {
    pub(super) fn load32_sram(&mut self, _address: u32, _access: AccessType) -> (u32, Waitstates) {
        todo!("load32_sram")
    }

    pub(super) fn load16_sram(&mut self, _address: u32, _access: AccessType) -> (u16, Waitstates) {
        todo!("load16_sram")
    }

    pub(super) fn load8_sram(&mut self, _address: u32, _access: AccessType) -> (u8, Waitstates) {
        todo!("load8_sram")
    }

    pub(super) fn store32_sram(
        &mut self,
        _address: u32,
        _value: u32,
        _access: AccessType,
    ) -> Waitstates {
        todo!("store32_sram")
    }

    pub(super) fn store16_sram(
        &mut self,
        _address: u32,
        _value: u16,
        _access: AccessType,
    ) -> Waitstates {
        todo!("store16_sram")
    }

    pub(super) fn store8_sram(
        &mut self,
        _address: u32,
        _value: u8,
        _access: AccessType,
    ) -> Waitstates {
        todo!("store8_sram")
    }
}
