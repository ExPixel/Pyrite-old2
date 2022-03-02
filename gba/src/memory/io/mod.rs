mod constants;
mod types;

use super::GbaMemory;
pub use constants::*;
pub use types::*;
use util::bits::Bits as _;

impl GbaMemory {
    pub(super) fn load32_io(&mut self, address: u32) -> u32 {
        let lo = self.load16_io(address) as u32;
        let hi = self.load16_io(address + 2) as u32;

        lo | (hi << 16)
    }

    pub(super) fn load16_io(&mut self, address: u32) -> u16 {
        match address {
            // LCD
            DISPCNT => self.ioregs.dispcnt.into(),
            GREENSWAP => self.ioregs.greenswap,
            DISPSTAT => self.ioregs.dispstat.into(),
            VCOUNT => self.ioregs.vcount,
            BG0CNT => self.ioregs.bgcnt[0],
            BG1CNT => self.ioregs.bgcnt[1],
            BG2CNT => self.ioregs.bgcnt[2],
            BG3CNT => self.ioregs.bgcnt[3],
            BLDCNT => self.ioregs.bldcnt,
            BLDALPHA => self.ioregs.bldalpha,
            BLDY => self.ioregs.bldy,

            // Keypad Input
            KEYINPUT => self.ioregs.keyinput,

            WAITCNT => self.ioregs.waitcnt,
            _ => {
                log::warn!("attempted to read from unused IO address 0x{:08X}", address);
                0
            }
        }
    }

    pub(super) fn load8_io(&mut self, address: u32) -> u8 {
        let shift = (address & 1) * 8;
        (self.load16_io(address & !0x1) >> shift) as u8
    }

    pub(super) fn store32_io(&mut self, address: u32, value: u32) {
        self.store16_io(address, value as u16);
        self.store16_io(address + 2, (value >> 16) as u16);
    }

    pub(super) fn store16_io(&mut self, address: u32, value: u16) {
        match address {
            // LCD
            DISPCNT => self.ioregs.dispcnt.set_preserve_bits(value),
            GREENSWAP => self.ioregs.greenswap = value,
            DISPSTAT => self.ioregs.dispstat.set_preserve_bits(value),
            VCOUNT => { /* NOP */ }
            BG0CNT => self.ioregs.bgcnt[0] = value,
            BG1CNT => self.ioregs.bgcnt[1] = value,
            BG2CNT => self.ioregs.bgcnt[2] = value,
            BG3CNT => self.ioregs.bgcnt[3] = value,
            BLDCNT => self.ioregs.bldcnt = value,
            BLDALPHA => self.ioregs.bldalpha = value,
            BLDY => self.ioregs.bldy = value,

            // Keypad Input
            KEYINPUT => { /*NOP */ }

            WAITCNT => {
                self.ioregs.set_waitcnt(value);
                self.update_waitcnt();
            }

            _ => {
                log::warn!(
                    "attempted to write 0x{:04X} to unused IO address 0x{:08X}",
                    value,
                    address
                );
            }
        }
    }

    pub(super) fn store8_io(&mut self, address: u32, value: u8) {
        let mut value16 = self.view16_io(address);
        let shift = (address & 1) * 8;
        value16 &= !0xFF << shift;
        value16 |= (value as u16) << shift;
        self.store16_io(address, value16)
    }

    pub(super) fn view32_io(&self, _address: u32) -> u32 {
        todo!("view32_io")
    }

    pub(super) fn view16_io(&self, _address: u32) -> u16 {
        todo!("view16_io")
    }

    pub(super) fn view8_io(&self, _address: u32) -> u8 {
        todo!("view8_io")
    }

    pub(super) fn update_waitcnt(&mut self) {
        // NOTE: subtract 1 from cycles to get number of wait states.
        //
        //  Bit   Expl.
        //  0-1   SRAM Wait Control          (0..3 = 4,3,2,8 cycles)
        //  2-3   Wait State 0 First Access  (0..3 = 4,3,2,8 cycles)
        //  4     Wait State 0 Second Access (0..1 = 2,1 cycles)
        //  5-6   Wait State 1 First Access  (0..3 = 4,3,2,8 cycles)
        //  7     Wait State 1 Second Access (0..1 = 4,1 cycles; unlike above WS0)
        //  8-9   Wait State 2 First Access  (0..3 = 4,3,2,8 cycles)
        //  10    Wait State 2 Second Access (0..1 = 8,1 cycles; unlike above WS0,WS1)
        //  11-12 PHI Terminal Output        (0..3 = Disable, 4.19MHz, 8.38MHz, 16.78MHz)
        //  13    Not used
        //  14    Game Pak Prefetch Buffer (Pipe) (0=Disable, 1=Enable)
        //  15    Game Pak Type Flag  (Read Only) (0=GBA, 1=CGB) (IN35 signal)
        //  16-31 Not used

        const SRAM_WAIT_VALUES: [u8; 4] = [3, 2, 1, 7];

        const WS0_NONSEQ_VALUES: [u8; 4] = [3, 2, 1, 7];
        const WS0_SEQ_VALUES: [u8; 2] = [1, 0];

        const WS1_NONSEQ_VALUES: [u8; 4] = [3, 2, 1, 7];
        const WS1_SEQ_VALUES: [u8; 2] = [3, 0];

        const WS2_NONSEQ_VALUES: [u8; 4] = [3, 2, 1, 7];
        const WS2_SEQ_VALUES: [u8; 2] = [7, 0];

        let waitcnt = self.ioregs.waitcnt;

        self.sram_waitstates = SRAM_WAIT_VALUES[waitcnt.bits(0, 1) as usize].into();

        self.gamepak_waitstates[0] = WS0_NONSEQ_VALUES[waitcnt.bits(2, 3) as usize].into();
        self.gamepak_waitstates[1] = WS0_SEQ_VALUES[waitcnt.bit(4) as usize].into();

        self.gamepak_waitstates[2] = WS1_NONSEQ_VALUES[waitcnt.bits(5, 6) as usize].into();
        self.gamepak_waitstates[3] = WS1_SEQ_VALUES[waitcnt.bit(7) as usize].into();

        self.gamepak_waitstates[4] = WS2_NONSEQ_VALUES[waitcnt.bits(8, 9) as usize].into();
        self.gamepak_waitstates[5] = WS2_SEQ_VALUES[waitcnt.bit(10) as usize].into();

        self.prefetch_enabled = waitcnt.is_bit_set(14);
    }
}

#[derive(Default)]
pub struct IoRegisters {
    // LCD
    pub(crate) dispcnt: LCDControl,
    pub(crate) greenswap: u16,
    pub(crate) dispstat: LCDStatus,
    pub(crate) waitcnt: u16,
    pub(crate) vcount: u16,
    pub(crate) bldcnt: u16,
    pub(crate) bldalpha: u16,
    pub(crate) bldy: u16,
    pub(crate) bgcnt: [u16; 4],

    // Keypad Input
    pub(crate) keyinput: u16,
}

impl IoRegisters {
    pub fn init(&mut self) {
        self.keyinput = 0x3ff;
    }

    pub fn waitcnt(&self) -> u16 {
        self.waitcnt
    }

    pub fn set_waitcnt(&mut self, value: u16) {
        set_preserve_bits(&mut self.waitcnt, value, 0x8000);
    }

    pub fn dispcnt(&self) -> LCDControl {
        self.dispcnt
    }

    pub fn set_dispcnt(&mut self, value: LCDControl) {
        self.dispcnt = value;
    }
}

fn set_preserve_bits<T>(dst: &mut T, src: T, readonly_mask: T)
where
    T: Copy
        + std::ops::BitOr<Output = T>
        + std::ops::BitAnd<Output = T>
        + std::ops::Not<Output = T>,
{
    *dst = (src & !readonly_mask) | (*dst & readonly_mask);
}
