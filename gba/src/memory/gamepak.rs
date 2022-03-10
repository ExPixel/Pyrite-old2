use super::{GbaMemory, ROM_MAX_MASK};
use arm::{AccessType, Waitstates};
use util::mem::{read_u16, read_u32};

impl GbaMemory {
    pub(super) fn load32_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        mut access: AccessType,
    ) -> (u32, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            read_u32(&self.rom, masked)
        } else {
            0
        };

        gamepak_access_fix(address, &mut access);
        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn load16_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        mut access: AccessType,
    ) -> (u16, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            read_u16(&self.rom, masked)
        } else {
            0
        };

        gamepak_access_fix(address, &mut access);
        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn load8_gamepak(
        &self,
        address: u32,
        waitstate: u8,
        mut access: AccessType,
    ) -> (u8, Waitstates) {
        let masked = (address & ROM_MAX_MASK) as usize;
        let value = if masked < self.rom.len() {
            self.rom[masked]
        } else {
            0
        };

        gamepak_access_fix(address, &mut access);
        let wait = self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1];

        (value, wait)
    }

    pub(super) fn store32_gamepak(
        &self,
        address: u32,
        _value: u32,
        waitstate: u8,
        mut access: AccessType,
    ) -> Waitstates {
        gamepak_access_fix(address, &mut access);
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }

    pub(super) fn store16_gamepak(
        &self,
        address: u32,
        _value: u16,
        waitstate: u8,
        mut access: AccessType,
    ) -> Waitstates {
        gamepak_access_fix(address, &mut access);
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }

    pub(super) fn store8_gamepak(
        &self,
        address: u32,
        _value: u8,
        waitstate: u8,
        mut access: AccessType,
    ) -> Waitstates {
        gamepak_access_fix(address, &mut access);
        self.gamepak_waitstates[((waitstate as usize) << 1) + (access as usize)]
            + self.gamepak_waitstates[((waitstate as usize) << 1) + 1]
    }
}

/// Changes the given access type to nonsequential if the address is the start of
/// a 128K block.
#[inline(always)]
fn gamepak_access_fix(address: u32, access: &mut AccessType) {
    if (address & 0x1FFFF) == 0 {
        *access = AccessType::NonSeq;
    }
}
