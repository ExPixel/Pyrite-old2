use super::GbaMemory;
use arm::{AccessType, Waitstates};

impl GbaMemory {
    pub(super) fn load32_sram(&mut self, address: u32, access: AccessType) -> (u32, Waitstates) {
        todo!("load32_sram")
    }

    pub(super) fn load16_sram(&mut self, address: u32, access: AccessType) -> (u16, Waitstates) {
        todo!("load16_sram")
    }

    pub(super) fn load8_sram(&mut self, address: u32, access: AccessType) -> (u8, Waitstates) {
        todo!("load8_sram")
    }

    pub(super) fn store32_sram(
        &mut self,
        address: u32,
        value: u32,
        access: AccessType,
    ) -> Waitstates {
        todo!("store32_sram")
    }

    pub(super) fn store16_sram(
        &mut self,
        address: u32,
        value: u16,
        access: AccessType,
    ) -> Waitstates {
        todo!("store16_sram")
    }

    pub(super) fn store8_sram(
        &mut self,
        address: u32,
        value: u8,
        access: AccessType,
    ) -> Waitstates {
        todo!("store8_sram")
    }
}
