use super::{GbaMemory, ROM_MAX_MASK};
use arm::{AccessType, Waitstates};
use byteorder::{ByteOrder, LittleEndian as LE};

impl GbaMemory {
    pub(super) fn load32_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        access: AccessType,
    ) -> (u32, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            LE::read_u32(&self.rom[masked..])
        } else {
            0
        };

        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn load16_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        access: AccessType,
    ) -> (u16, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            LE::read_u16(&self.rom[masked..])
        } else {
            0
        };

        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn load8_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        access: AccessType,
    ) -> (u8, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            self.rom[masked]
        } else {
            0
        };

        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn store32_gamepak(
        &self,
        _address: u32,
        _value: u32,
        waitstate: u8,
        access: AccessType,
    ) -> Waitstates {
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }

    pub(super) fn store16_gamepak(
        &self,
        _address: u32,
        _value: u16,
        waitstate: u8,
        access: AccessType,
    ) -> Waitstates {
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }

    pub(super) fn store8_gamepak(
        &self,
        _address: u32,
        _value: u8,
        waitstate: u8,
        access: AccessType,
    ) -> Waitstates {
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }
}
