use util::bits::Bits as _;

use super::GbaMemory;

impl GbaMemory {
    pub(super) fn load32_io(&mut self, address: u32) -> u32 {
        let lo = self.load16_io(address) as u32;
        let hi = self.load16_io(address + 2) as u32;

        lo | (hi << 16)
    }

    pub(super) fn load16_io(&mut self, address: u32) -> u16 {
        match address {
            // LCD
            DISPCNT => self.ioregs.dispcnt,
            DISPSTAT => self.ioregs.dispstat,
            VCOUNT => self.ioregs.vcount,

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
            DISPCNT => self.ioregs.set_dispcnt(value),
            DISPSTAT => self.ioregs.set_dispstat(value),
            VCOUNT => { /* NOP */ }

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
    pub(crate) dispcnt: u16,
    pub(crate) dispstat: u16,
    pub(crate) waitcnt: u16,
    pub(crate) vcount: u16,
}

impl IoRegisters {
    pub fn waitcnt(&self) -> u16 {
        self.waitcnt
    }

    pub fn set_waitcnt(&mut self, value: u16) {
        set_preserve_bits(&mut self.waitcnt, value, 0x8000);
    }

    pub fn dispcnt(&self) -> u16 {
        self.dispcnt
    }

    pub fn set_dispcnt(&mut self, value: u16) {
        self.dispcnt = value;
    }

    pub fn dispstat(&self) -> u16 {
        self.dispstat
    }

    pub fn set_dispstat(&mut self, value: u16) {
        set_preserve_bits(&mut self.dispstat, value, 0x0047);
    }

    pub fn vcount(&self) -> u16 {
        self.vcount
    }

    pub fn vblank(&self) -> bool {
        self.dispstat.is_bit_set(0)
    }

    pub(crate) fn set_vblank(&mut self, value: bool) {
        self.dispstat = self.dispstat.replace_bit(0, value);
    }

    pub fn hblank(&self) -> bool {
        self.dispstat.is_bit_set(1)
    }

    pub(crate) fn set_hblank(&mut self, value: bool) {
        self.dispstat = self.dispstat.replace_bit(1, value);
    }

    pub fn vcount_match(&self) -> bool {
        self.dispstat.is_bit_set(2)
    }

    pub(crate) fn set_vcount_match(&mut self, value: bool) {
        self.dispstat = self.dispstat.replace_bit(2, value);
    }

    pub fn vcount_setting(&self) -> u16 {
        self.dispstat >> 8
    }

    pub fn bg_mode(&self) -> u16 {
        self.dispcnt & 0x7
    }

    pub fn is_bitmap_mode(&self) -> bool {
        (3..6).contains(&self.bg_mode())
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

// LCD I/O
pub const DISPCNT: u32 = 0x04000000;
pub const GREENSWAP: u32 = 0x04000002;
pub const DISPSTAT: u32 = 0x04000004;
pub const VCOUNT: u32 = 0x04000006;
pub const BG0CNT: u32 = 0x04000008;
pub const BG1CNT: u32 = 0x0400000A;
pub const BG2CNT: u32 = 0x0400000C;
pub const BG3CNT: u32 = 0x0400000E;
pub const BG0HOFS: u32 = 0x04000010;
pub const BG0VOFS: u32 = 0x04000012;
pub const BG1HOFS: u32 = 0x04000014;
pub const BG1VOFS: u32 = 0x04000016;
pub const BG2HOFS: u32 = 0x04000018;
pub const BG2VOFS: u32 = 0x0400001A;
pub const BG3HOFS: u32 = 0x0400001C;
pub const BG3VOFS: u32 = 0x0400001E;
pub const BG2PA: u32 = 0x04000020;
pub const BG2PB: u32 = 0x04000022;
pub const BG2PC: u32 = 0x04000024;
pub const BG2PD: u32 = 0x04000026;
pub const BG2X: u32 = 0x04000028;
pub const BG2X_HI: u32 = 0x0400002A;
pub const BG2Y: u32 = 0x0400002C;
pub const BG2Y_HI: u32 = 0x0400002E;
pub const BG3PA: u32 = 0x04000030;
pub const BG3PB: u32 = 0x04000032;
pub const BG3PC: u32 = 0x04000034;
pub const BG3PD: u32 = 0x04000036;
pub const BG3X: u32 = 0x04000038;
pub const BG3X_HI: u32 = 0x0400003A;
pub const BG3Y: u32 = 0x0400003C;
pub const BG3Y_HI: u32 = 0x0400003E;
pub const WIN0H: u32 = 0x04000040;
pub const WIN1H: u32 = 0x04000042;
pub const WIN0V: u32 = 0x04000044;
pub const WIN1V: u32 = 0x04000046;
pub const WININ: u32 = 0x04000048;
pub const WINOUT: u32 = 0x0400004A;
pub const MOSAIC: u32 = 0x0400004C;
pub const MOSAIC_HI: u32 = 0x0400004E;
pub const BLDCNT: u32 = 0x04000050;
pub const BLDALPHA: u32 = 0x04000052;
pub const BLDY: u32 = 0x04000054;

// Sound Registers
pub const SOUND1CNT_L: u32 = 0x04000060;
pub const SOUND1CNT_H: u32 = 0x04000062;
pub const SOUND1CNT_X: u32 = 0x04000064;
pub const SOUND2CNT_L: u32 = 0x04000068;
pub const SOUND2CNT_H: u32 = 0x0400006C;
pub const SOUND3CNT_L: u32 = 0x04000070;
pub const SOUND3CNT_H: u32 = 0x04000072;
pub const SOUND3CNT_X: u32 = 0x04000074;
pub const SOUND4CNT_L: u32 = 0x04000078;
pub const SOUND4CNT_H: u32 = 0x0400007C;
pub const SOUNDCNT_L: u32 = 0x04000080;
pub const SOUNDCNT_H: u32 = 0x04000082;
pub const SOUNDCNT_X: u32 = 0x04000084;
pub const SOUNDCNT_X_H: u32 = 0x04000086;
pub const SOUNDBIAS: u32 = 0x04000088;
pub const SOUNDBIAS_H: u32 = 0x0400008A;
pub const FIFO_A: u32 = 0x040000A0;
pub const FIFO_B: u32 = 0x040000A4;

// Sound Registers (Using NR names)
pub const NR10: u32 = 0x04000060;
pub const NR11: u32 = 0x04000062;
pub const NR12: u32 = 0x04000063;
pub const NR13: u32 = 0x04000064;
pub const NR14: u32 = 0x04000065;

pub const NR21: u32 = 0x04000068;
pub const NR22: u32 = 0x04000069;
pub const NR23: u32 = 0x0400006C;
pub const NR24: u32 = 0x0400006D;

pub const NR30: u32 = 0x04000070;
pub const NR31: u32 = 0x04000072;
pub const NR32: u32 = 0x04000073;
pub const NR33: u32 = 0x04000074;
pub const NR34: u32 = 0x04000075;

pub const NR41: u32 = 0x04000078;
pub const NR42: u32 = 0x04000079;
pub const NR43: u32 = 0x0400007C;
pub const NR44: u32 = 0x0400007D;

pub const NR50: u32 = 0x04000080;
pub const NR51: u32 = 0x04000081;
pub const NR52: u32 = 0x04000084;

// DMA Transfer Channels
pub const DMA0SAD: u32 = 0x040000B0;
pub const DMA0SAD_H: u32 = 0x040000B2;
pub const DMA0DAD: u32 = 0x040000B4;
pub const DMA0DAD_H: u32 = 0x040000B6;
pub const DMA0CNT_L: u32 = 0x040000B8;
pub const DMA0CNT_H: u32 = 0x040000BA;
pub const DMA1SAD: u32 = 0x040000BC;
pub const DMA1SAD_H: u32 = 0x040000BE;
pub const DMA1DAD: u32 = 0x040000C0;
pub const DMA1DAD_H: u32 = 0x040000C2;
pub const DMA1CNT_L: u32 = 0x040000C4;
pub const DMA1CNT_H: u32 = 0x040000C6;
pub const DMA2SAD: u32 = 0x040000C8;
pub const DMA2SAD_H: u32 = 0x040000CA;
pub const DMA2DAD: u32 = 0x040000CC;
pub const DMA2DAD_H: u32 = 0x040000CE;
pub const DMA2CNT_L: u32 = 0x040000D0;
pub const DMA2CNT_H: u32 = 0x040000D2;
pub const DMA3SAD: u32 = 0x040000D4;
pub const DMA3SAD_H: u32 = 0x040000D6;
pub const DMA3DAD: u32 = 0x040000D8;
pub const DMA3DAD_H: u32 = 0x040000DA;
pub const DMA3CNT_L: u32 = 0x040000DC;
pub const DMA3CNT_H: u32 = 0x040000DE;

// Timer Registers
pub const TM0CNT_L: u32 = 0x04000100;
pub const TM0CNT_H: u32 = 0x04000102;
pub const TM1CNT_L: u32 = 0x04000104;
pub const TM1CNT_H: u32 = 0x04000106;
pub const TM2CNT_L: u32 = 0x04000108;
pub const TM2CNT_H: u32 = 0x0400010A;
pub const TM3CNT_L: u32 = 0x0400010C;
pub const TM3CNT_H: u32 = 0x0400010E;

// Serial Communication (1)_
pub const SIODATA32: u32 = 0x04000120;
pub const SIOMULTI0: u32 = 0x04000120;
pub const SIOMULTI1: u32 = 0x04000122;
pub const SIOMULTI2: u32 = 0x04000124;
pub const SIOMULTI3: u32 = 0x04000126;
pub const SIOCNT: u32 = 0x04000128;
pub const SIOMLT_SEND: u32 = 0x0400012A;
pub const SIODATA8: u32 = 0x0400012A;

// Keypad Input
pub const KEYINPUT: u32 = 0x04000130;
pub const KEYCNT: u32 = 0x04000132;

// Serial Communication (2)
pub const RCNT: u32 = 0x04000134;
pub const IR: u32 = 0x04000136;
pub const JOYCNT: u32 = 0x04000140;
pub const JOY_RECV: u32 = 0x04000150;
pub const JOY_TRANS: u32 = 0x04000154;
pub const JOYSTAT: u32 = 0x04000158;

// Interrupt, Waitstate, and Power-Down Control
pub const IE: u32 = 0x04000200;
pub const IF: u32 = 0x04000202;
pub const WAITCNT: u32 = 0x04000204;
pub const IME: u32 = 0x04000208;
pub const IME_HI: u32 = 0x040020A;
pub const POSTFLG: u32 = 0x04000300;
pub const HALTCNT: u32 = 0x04000301;
pub const BUG410: u32 = 0x04000410;
pub const IMC: u32 = 0x04000800;
pub const IMC_H: u32 = 0x04000802;
