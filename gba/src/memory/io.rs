use byteorder::{ByteOrder as _, LittleEndian as LE};
use util::bits::Bits as _;

use super::GbaMemory;

impl GbaMemory {
    pub(super) fn load32_io(&mut self, address: u32) -> u32 {
        todo!("load32_io")
    }

    pub(super) fn load16_io(&mut self, address: u32) -> u16 {
        todo!("load16_io")
    }

    pub(super) fn load8_io(&mut self, address: u32) -> u8 {
        todo!("load8_io")
    }

    pub(super) fn store32_io(&mut self, address: u32, value: u32) {
        todo!("store32_io")
    }

    pub(super) fn store16_io(&mut self, address: u32, value: u16) {
        todo!("store16_io")
    }

    pub(super) fn store8_io(&mut self, address: u32, value: u8) {
        todo!("store8_io")
    }

    pub(super) fn set_waitcnt(&mut self, value: u16) {
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

        let value = set_preserve_bits(value, LE::read_u16(&self.ioregs[0x204..]), 0x8000);
        LE::write_u16(&mut self.ioregs[0x204..], value);

        self.sram_waitstates = SRAM_WAIT_VALUES[value.bits(0, 1) as usize].into();

        self.gamepak_waitstates[0] = WS0_NONSEQ_VALUES[value.bits(2, 3) as usize].into();
        self.gamepak_waitstates[1] = WS0_SEQ_VALUES[value.bit(4) as usize].into();

        self.gamepak_waitstates[2] = WS1_NONSEQ_VALUES[value.bits(5, 6) as usize].into();
        self.gamepak_waitstates[3] = WS1_SEQ_VALUES[value.bit(7) as usize].into();

        self.gamepak_waitstates[4] = WS2_NONSEQ_VALUES[value.bits(8, 9) as usize].into();
        self.gamepak_waitstates[5] = WS2_SEQ_VALUES[value.bit(10) as usize].into();

        self.prefetch_enabled = value.is_bit_set(14);
    }
}

fn set_preserve_bits<T>(new_value: T, old_value: T, readonly_mask: T) -> T
where
    T: Copy
        + std::ops::BitOr<Output = T>
        + std::ops::BitAnd<Output = T>
        + std::ops::Not<Output = T>,
{
    (new_value & !readonly_mask) | (old_value & readonly_mask)
}
