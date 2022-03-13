mod constants;
mod types;

use super::GbaMemory;
pub use constants::*;
pub use types::*;
use util::bits::Bits as _;

impl GbaMemory {
    pub(super) fn load32_io<const SIDE_EFFECTS: bool>(&self, address: u32) -> u32 {
        let lo = self.load16_io::<SIDE_EFFECTS>(address) as u32;
        let hi = self.load16_io::<SIDE_EFFECTS>(address + 2) as u32;

        lo | (hi << 16)
    }

    pub(super) fn load16_io<const SIDE_EFFECTS: bool>(&self, address: u32) -> u16 {
        match address {
            // LCD
            DISPCNT => self.ioregs.dispcnt.into(),
            GREENSWAP => self.ioregs.greenswap,
            DISPSTAT => self.ioregs.dispstat.into(),
            VCOUNT => self.ioregs.vcount,
            BG0CNT => self.ioregs.bgcnt[0].into(),
            BG1CNT => self.ioregs.bgcnt[1].into(),
            BG2CNT => self.ioregs.bgcnt[2].into(),
            BG3CNT => self.ioregs.bgcnt[3].into(),
            BLDCNT => self.ioregs.bldcnt.into(),
            BLDALPHA => self.ioregs.bldalpha.into(),
            BLDY => self.ioregs.bldy.into(),
            WIN0H => self.ioregs.winhv.win0_h(),
            WIN1H => self.ioregs.winhv.win1_h(),
            WIN0V => self.ioregs.winhv.win0_v(),
            WIN1V => self.ioregs.winhv.win1_v(),
            WININ => self.ioregs.wininout.winin(),
            WINOUT => self.ioregs.wininout.winout(),

            // DMA
            DMA0SAD => self.ioregs.dma[0].source.lo(),
            DMA0SAD_H => self.ioregs.dma[0].source.hi(),
            DMA0DAD => self.ioregs.dma[0].destination.lo(),
            DMA0DAD_H => self.ioregs.dma[0].destination.hi(),
            DMA0CNT_L => self.ioregs.dma[0].count,
            DMA0CNT_H => self.ioregs.dma[0].control.into(),
            DMA1SAD => self.ioregs.dma[1].source.lo(),
            DMA1SAD_H => self.ioregs.dma[1].source.hi(),
            DMA1DAD => self.ioregs.dma[1].destination.lo(),
            DMA1DAD_H => self.ioregs.dma[1].destination.hi(),
            DMA1CNT_L => self.ioregs.dma[1].count,
            DMA1CNT_H => self.ioregs.dma[1].control.into(),
            DMA2SAD => self.ioregs.dma[2].source.lo(),
            DMA2SAD_H => self.ioregs.dma[2].source.hi(),
            DMA2DAD => self.ioregs.dma[2].destination.lo(),
            DMA2DAD_H => self.ioregs.dma[2].destination.hi(),
            DMA2CNT_L => self.ioregs.dma[2].count,
            DMA2CNT_H => self.ioregs.dma[2].control.into(),
            DMA3SAD => self.ioregs.dma[3].source.lo(),
            DMA3SAD_H => self.ioregs.dma[3].source.hi(),
            DMA3DAD => self.ioregs.dma[3].destination.lo(),
            DMA3DAD_H => self.ioregs.dma[3].destination.hi(),
            DMA3CNT_L => self.ioregs.dma[3].count,
            DMA3CNT_H => self.ioregs.dma[3].control.into(),

            // Keypad Input
            KEYINPUT => self.ioregs.keyinput,

            // Interrupt, Waitstate, and Power-Down Control
            IE => self.ioregs.ie_reg.into(),
            IF => self.ioregs.if_reg.into(),
            WAITCNT => self.ioregs.waitcnt.into(),
            IME => self.ioregs.ime.into(),

            _ => {
                log::warn!(
                    "attempted to read from unused/readonly IO address 0x{:08X}",
                    address
                );
                0
            }
        }
    }

    pub(super) fn load8_io<const SIDE_EFFECTS: bool>(&mut self, address: u32) -> u8 {
        let shift = (address & 1) * 8;
        (self.load16_io::<SIDE_EFFECTS>(address & !0x1) >> shift) as u8
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
            BG0CNT => self.ioregs.bgcnt[0].set_preserve_bits(value),
            BG1CNT => self.ioregs.bgcnt[1].set_preserve_bits(value),
            BG2CNT => self.ioregs.bgcnt[2].set_preserve_bits(value),
            BG3CNT => self.ioregs.bgcnt[3].set_preserve_bits(value),
            BG0HOFS => self.ioregs.bgofs[0].set_x(value),
            BG0VOFS => self.ioregs.bgofs[0].set_y(value),
            BG1HOFS => self.ioregs.bgofs[1].set_x(value),
            BG1VOFS => self.ioregs.bgofs[1].set_y(value),
            BG2HOFS => self.ioregs.bgofs[2].set_x(value),
            BG2VOFS => self.ioregs.bgofs[2].set_y(value),
            BG3HOFS => self.ioregs.bgofs[3].set_x(value),
            BG3VOFS => self.ioregs.bgofs[3].set_y(value),
            BLDCNT => self.ioregs.bldcnt.set_preserve_bits(value),
            BLDALPHA => self.ioregs.bldalpha.set_preserve_bits(value),
            BLDY => self.ioregs.bldy.set_preserve_bits(value),
            WIN0H => self.ioregs.winhv.set_win0_h(value),
            WIN1H => self.ioregs.winhv.set_win1_h(value),
            WIN0V => self.ioregs.winhv.set_win0_v(value),
            WIN1V => self.ioregs.winhv.set_win1_v(value),
            WININ => self.ioregs.wininout.set_winin(value),
            WINOUT => self.ioregs.wininout.set_winout(value),

            // DMA
            DMA0SAD => self.ioregs.dma[0].source.set_lo(value),
            DMA0SAD_H => self.ioregs.dma[0].source.set_hi(value),
            DMA0DAD => self.ioregs.dma[0].destination.set_lo(value),
            DMA0DAD_H => self.ioregs.dma[0].destination.set_hi(value),
            DMA0CNT_L => self.ioregs.dma[0].count = value,
            DMA0CNT_H => self.write_to_dma_control(0, value),
            DMA1SAD => self.ioregs.dma[1].source.set_lo(value),
            DMA1SAD_H => self.ioregs.dma[1].source.set_hi(value),
            DMA1DAD => self.ioregs.dma[1].destination.set_lo(value),
            DMA1DAD_H => self.ioregs.dma[1].destination.set_hi(value),
            DMA1CNT_L => self.ioregs.dma[1].count = value,
            DMA1CNT_H => self.write_to_dma_control(1, value),
            DMA2SAD => self.ioregs.dma[2].source.set_lo(value),
            DMA2SAD_H => self.ioregs.dma[2].source.set_hi(value),
            DMA2DAD => self.ioregs.dma[2].destination.set_lo(value),
            DMA2DAD_H => self.ioregs.dma[2].destination.set_hi(value),
            DMA2CNT_L => self.ioregs.dma[2].count = value,
            DMA2CNT_H => self.write_to_dma_control(2, value),
            DMA3SAD => self.ioregs.dma[3].source.set_lo(value),
            DMA3SAD_H => self.ioregs.dma[3].source.set_hi(value),
            DMA3DAD => self.ioregs.dma[3].destination.set_lo(value),
            DMA3DAD_H => self.ioregs.dma[3].destination.set_hi(value),
            DMA3CNT_L => self.ioregs.dma[3].count = value,
            DMA3CNT_H => self.write_to_dma_control(3, value),

            // Keypad Input
            KEYINPUT => { /*NOP */ }

            // Interrupt, Waitstate, and Power-Down Control
            IE => self.ioregs.ie_reg.set_preserve_bits(value),
            IF => self.ioregs.if_reg.set_acknowledge(value),
            WAITCNT => {
                self.ioregs.waitcnt.set_preserve_bits(value);
                self.update_waitcnt();
            }
            IME => self.ioregs.ime.set_preserve_bits(value),

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
        let mut value16 = self.load16_io::<false>(address);
        let shift = (address & 1) * 8;
        value16 &= !0xFF << shift;
        value16 |= (value as u16) << shift;
        self.store16_io(address, value16)
    }

    fn write_to_dma_control(&mut self, dma: usize, value: u16) {
        use crate::dma;

        let old_value = self.ioregs.dma[dma].control;
        self.ioregs.dma[dma].control.set_preserve_bits(value);

        if !old_value.enabled() && self.ioregs.dma[dma].control.enabled() {
            self.scheduler.schedule(
                match dma {
                    0 => dma::dma_enabled::<0>,
                    1 => dma::dma_enabled::<1>,
                    2 => dma::dma_enabled::<2>,
                    3 => dma::dma_enabled::<3>,
                    _ => unreachable!("invalid DMA index"),
                },
                0,
            );
        }
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

        let waitcnt = self.ioregs.waitcnt.value;

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
    pub(crate) vcount: u16,
    pub(crate) bgcnt: [BgControl; 4],
    pub(crate) bgofs: [BgOffset; 4],
    pub(crate) winhv: WindowDimensions,
    pub(crate) wininout: WindowInOut,
    pub(crate) bldcnt: ColorSpecialEffects,
    pub(crate) bldalpha: AlphaBlendingCoeff,
    pub(crate) bldy: BrightnessCoeff,

    // DMA
    pub(crate) dma: [DMARegisters; 4],

    // Keypad Input
    pub(crate) keyinput: u16,

    // Interrupt, Waitstate, and Power-Down Control
    pub(crate) ie_reg: InterruptEnable,
    pub(crate) if_reg: InterruptReqAck,
    pub(crate) waitcnt: WaitstateControl,
    pub(crate) ime: InterruptMasterEnable,

    /// This is NOT a register but a temporary location for pending interrupts.
    pub(crate) irq_pending: InterruptReqAck,
}

impl IoRegisters {
    pub fn init(&mut self) {
        self.keyinput = 0x3ff;
    }
}
