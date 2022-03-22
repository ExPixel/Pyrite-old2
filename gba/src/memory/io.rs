mod constants;
mod types;

use crate::scheduler::{EventFn, EventTag};

use super::GbaMemory;
pub use constants::*;
pub use types::*;
use util::{
    bits::Bits as _,
    fixedpoint::{FixedPoint16, FixedPoint32},
};

impl GbaMemory {
    pub(super) fn load32_io<const SIDE_EFFECTS: bool>(&mut self, address: u32) -> u32 {
        let lo = self.load16_io::<SIDE_EFFECTS>(address) as u32;
        let hi = self.load16_io::<SIDE_EFFECTS>(address + 2) as u32;

        lo | (hi << 16)
    }

    pub(super) fn load16_io<const SIDE_EFFECTS: bool>(&mut self, address: u32) -> u16 {
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
            BLDY => self.ioregs.bldy.lo(),
            BLDY_H => self.ioregs.bldy.hi(),
            WIN0H => self.ioregs.winhv.win0_h(),
            WIN1H => self.ioregs.winhv.win1_h(),
            WIN0V => self.ioregs.winhv.win0_v(),
            WIN1V => self.ioregs.winhv.win1_v(),
            WININ => self.ioregs.wininout.winin(),
            WINOUT => self.ioregs.wininout.winout(),
            MOSAIC => self.ioregs.mosaic.lo(),
            MOSAIC_HI => self.ioregs.mosaic.hi(),

            // Sound
            SOUND1CNT_L => self.ioregs.sound1cnt_l.into(),
            SOUND1CNT_H => self.ioregs.sound1cnt_h.into(),
            SOUND1CNT_X => self.ioregs.sound1cnt_x.lo(),
            SOUND1CNT_X_H => self.ioregs.sound1cnt_x.hi(),
            SOUND2CNT_L => self.ioregs.sound2cnt_l.into(),
            SOUND2CNT_H => self.ioregs.sound2cnt_h.lo(),
            SOUND2CNT_H_H => self.ioregs.sound2cnt_h.hi(),
            SOUND3CNT_L => self.ioregs.sound3cnt_l.into(),
            SOUND3CNT_H => self.ioregs.sound3cnt_h.into(),
            SOUND3CNT_X => self.ioregs.sound3cnt_x.lo(),
            SOUND3CNT_X_H => self.ioregs.sound3cnt_x.hi(),
            SOUND4CNT_L => self.ioregs.sound4cnt_l.lo(),
            SOUND4CNT_L_H => self.ioregs.sound4cnt_l.hi(),
            SOUND4CNT_H => self.ioregs.sound4cnt_h.lo(),
            SOUND4CNT_H_H => self.ioregs.sound4cnt_h.hi(),
            SOUNDCNT_L => self.ioregs.soundcnt_l.into(),
            SOUNDCNT_H => self.ioregs.soundcnt_h.into(),
            SOUNDCNT_X => self.ioregs.soundcnt_x.lo(),
            SOUNDCNT_X_H => self.ioregs.soundcnt_x.hi(),
            SOUNDBIAS => self.ioregs.soundbias.lo(),
            SOUNDBIAS_H => self.ioregs.soundbias.hi(),
            WAVE_RAM0_L => self.ioregs.waveram.load16(0),
            WAVE_RAM0_H => self.ioregs.waveram.load16(1),
            WAVE_RAM1_L => self.ioregs.waveram.load16(2),
            WAVE_RAM1_H => self.ioregs.waveram.load16(3),
            WAVE_RAM2_L => self.ioregs.waveram.load16(4),
            WAVE_RAM2_H => self.ioregs.waveram.load16(5),
            WAVE_RAM3_L => self.ioregs.waveram.load16(6),
            WAVE_RAM3_H => self.ioregs.waveram.load16(7),
            FIFO_A_L => 0xDEAD,
            FIFO_A_H => 0xDEAD,
            FIFO_B_L => 0xDEAD,
            FIFO_B_H => 0xDEAD,

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

            // Timers
            TM0CNT_L => self.read_timer_counter(0),
            TM0CNT_H => self.ioregs.timers[0].control.into(),
            TM1CNT_L => self.read_timer_counter(1),
            TM1CNT_H => self.ioregs.timers[1].control.into(),
            TM2CNT_L => self.read_timer_counter(2),
            TM2CNT_H => self.ioregs.timers[2].control.into(),
            TM3CNT_L => self.read_timer_counter(3),
            TM3CNT_H => self.ioregs.timers[3].control.into(),

            // Keypad Input
            KEYINPUT => self.ioregs.keyinput,

            // Interrupt, Waitstate, and Power-Down Control
            IE => self.ioregs.ie_reg.into(),
            IF => self.ioregs.if_reg.into(),
            WAITCNT => self.ioregs.waitcnt.into(),
            IME => self.ioregs.ime.lo(),
            IME_HI => self.ioregs.ime.hi(),

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
            BG2PA => self.ioregs.bg2pa = FixedPoint16::raw(value as i16),
            BG2PB => self.ioregs.bg2pb = FixedPoint16::raw(value as i16),
            BG2PC => self.ioregs.bg2pc = FixedPoint16::raw(value as i16),
            BG2PD => self.ioregs.bg2pd = FixedPoint16::raw(value as i16),
            BG2X => self.ioregs.bg2x.set_lo(value),
            BG2X_H => self.ioregs.bg2x.set_hi(value),
            BG2Y => self.ioregs.bg2y.set_lo(value),
            BG2Y_H => self.ioregs.bg2y.set_hi(value),
            BG3PA => self.ioregs.bg3pa = FixedPoint16::raw(value as i16),
            BG3PB => self.ioregs.bg3pb = FixedPoint16::raw(value as i16),
            BG3PC => self.ioregs.bg3pc = FixedPoint16::raw(value as i16),
            BG3PD => self.ioregs.bg3pd = FixedPoint16::raw(value as i16),
            BG3X => self.ioregs.bg3x.set_lo(value),
            BG3X_H => self.ioregs.bg3x.set_hi(value),
            BG3Y => self.ioregs.bg3y.set_lo(value),
            BG3Y_H => self.ioregs.bg3y.set_hi(value),
            BLDCNT => self.ioregs.bldcnt.set_preserve_bits(value),
            BLDALPHA => self.ioregs.bldalpha.set_preserve_bits(value),
            BLDY => self.ioregs.bldy.set_lo(value),
            BLDY_H => self.ioregs.bldy.set_hi(value),
            WIN0H => self.ioregs.winhv.set_win0_h(value),
            WIN1H => self.ioregs.winhv.set_win1_h(value),
            WIN0V => self.ioregs.winhv.set_win0_v(value),
            WIN1V => self.ioregs.winhv.set_win1_v(value),
            WININ => self.ioregs.wininout.set_winin(value),
            WINOUT => self.ioregs.wininout.set_winout(value),
            MOSAIC => self.ioregs.mosaic.set_lo(value),
            MOSAIC_HI => self.ioregs.mosaic.set_hi(value),

            // Sound
            SOUND1CNT_L => self.ioregs.sound1cnt_l.set_preserve_bits(value),
            SOUND1CNT_H => self.ioregs.sound1cnt_h.set_preserve_bits(value),
            SOUND1CNT_X => self.ioregs.sound1cnt_x.set_lo(value),
            SOUND1CNT_X_H => self.ioregs.sound1cnt_x.set_hi(value),
            SOUND2CNT_L => self.ioregs.sound2cnt_l.set_preserve_bits(value),
            SOUND2CNT_H => self.ioregs.sound2cnt_h.set_lo(value),
            SOUND2CNT_H_H => self.ioregs.sound2cnt_h.set_hi(value),
            SOUND3CNT_L => self.ioregs.sound3cnt_l.set_preserve_bits(value),
            SOUND3CNT_H => self.ioregs.sound3cnt_h.set_preserve_bits(value),
            SOUND3CNT_X => self.ioregs.sound3cnt_x.set_lo(value),
            SOUND3CNT_X_H => self.ioregs.sound3cnt_x.set_hi(value),
            SOUND4CNT_L => self.ioregs.sound4cnt_l.set_lo(value),
            SOUND4CNT_L_H => self.ioregs.sound4cnt_l.set_hi(value),
            SOUND4CNT_H => self.ioregs.sound4cnt_h.set_lo(value),
            SOUND4CNT_H_H => self.ioregs.sound4cnt_h.set_hi(value),
            SOUNDCNT_L => self.ioregs.soundcnt_l.set_preserve_bits(value),
            SOUNDCNT_H => self.ioregs.soundcnt_h.set_preserve_bits(value),
            SOUNDCNT_X => self.ioregs.soundcnt_x.set_lo(value),
            SOUNDCNT_X_H => self.ioregs.soundcnt_x.set_hi(value),
            SOUNDBIAS => self.ioregs.soundbias.set_lo(value),
            SOUNDBIAS_H => self.ioregs.soundbias.set_hi(value),
            WAVE_RAM0_L => self.ioregs.waveram.store16(0, value),
            WAVE_RAM0_H => self.ioregs.waveram.store16(1, value),
            WAVE_RAM1_L => self.ioregs.waveram.store16(2, value),
            WAVE_RAM1_H => self.ioregs.waveram.store16(3, value),
            WAVE_RAM2_L => self.ioregs.waveram.store16(4, value),
            WAVE_RAM2_H => self.ioregs.waveram.store16(5, value),
            WAVE_RAM3_L => self.ioregs.waveram.store16(6, value),
            WAVE_RAM3_H => self.ioregs.waveram.store16(7, value),
            FIFO_A_L => self.ioregs.fifo_a.store16(value),
            FIFO_A_H => self.ioregs.fifo_a.store16(value),
            FIFO_B_L => self.ioregs.fifo_b.store16(value),
            FIFO_B_H => self.ioregs.fifo_b.store16(value),

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

            // Timers
            TM0CNT_L => self.write_to_timer_reload(0, value),
            TM0CNT_H => self.write_to_timer_cotrol(0, value),
            TM1CNT_L => self.write_to_timer_reload(1, value),
            TM1CNT_H => self.write_to_timer_cotrol(1, value),
            TM2CNT_L => self.write_to_timer_reload(2, value),
            TM2CNT_H => self.write_to_timer_cotrol(2, value),
            TM3CNT_L => self.write_to_timer_reload(3, value),
            TM3CNT_H => self.write_to_timer_cotrol(3, value),

            // Keypad Input
            KEYINPUT => { /*NOP */ }

            // Interrupt, Waitstate, and Power-Down Control
            IE => self.ioregs.ie_reg.set_preserve_bits(value),
            IF => self.ioregs.if_reg.write(value),
            WAITCNT => {
                self.ioregs.waitcnt.set_preserve_bits(value);
                self.update_waitcnt();
            }
            IME => self.ioregs.ime.set_lo(value),
            IME_HI => self.ioregs.ime.set_hi(value),
            POSTFLG => {
                self.ioregs.postflg.set_preserve_bits(value as u8);
                self.write_to_haltcnt((value >> 8) as u8);
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
        match address {
            POSTFLG => self.ioregs.postflg.set_preserve_bits(value),
            HALTCNT => self.write_to_haltcnt(value),
            IF => self.ioregs.if_reg.write(value as u16),
            IF_HI => self.ioregs.if_reg.write((value as u16) << 8),

            FIFO_A_L => self.ioregs.fifo_a.store8(value),
            FIFO_A_L_H => self.ioregs.fifo_a.store8(value),
            FIFO_A_H => self.ioregs.fifo_a.store8(value),
            FIFO_A_H_H => self.ioregs.fifo_a.store8(value),
            FIFO_B_L => self.ioregs.fifo_b.store8(value),
            FIFO_B_L_H => self.ioregs.fifo_b.store8(value),
            FIFO_B_H => self.ioregs.fifo_b.store8(value),
            FIFO_B_H_H => self.ioregs.fifo_b.store8(value),

            _ => {
                let mut value16 = self.load16_io::<false>(address);
                let shift = (address & 1) * 8;
                value16 &= !0xFF << shift;
                value16 |= (value as u16) << shift;
                self.store16_io(address, value16)
            }
        }
    }

    fn write_to_haltcnt(&mut self, value: u8) {
        let haltcnt = LowPowerModeControl::new(value);

        if haltcnt.stop() {
            self.scheduler.schedule(|gba| gba.stop(), 0, EventTag::Stop);
        } else {
            self.scheduler.schedule(|gba| gba.halt(), 0, EventTag::Halt);
        }
    }

    fn write_to_timer_reload(&mut self, timer: usize, value: u16) {
        self.ioregs.timers[timer].set_reload(value);
    }

    fn write_to_timer_cotrol(&mut self, timer: usize, new: u16) {
        let old = self.ioregs.timers[timer].control;
        let new = TimerControl::new(new);

        let prescaler_changed = old.prescaler() != new.prescaler();
        let count_up_changed = old.count_up_timing() != new.count_up_timing();

        if old.started() && new.started() && (prescaler_changed || count_up_changed) {
            crate::timers::flush(&mut self.ioregs.timers[timer], self.ioregs.time);
        }

        self.ioregs.timers[timer]
            .control
            .set_preserve_bits(new.value);

        if !old.started() && new.started() {
            crate::timers::started(timer, &mut self.ioregs, &self.scheduler);
        } else if old.started() && !new.started() {
            crate::timers::stopped(timer, &mut self.ioregs.timers, &self.scheduler);
        } else if new.started() && count_up_changed {
            crate::timers::reschedule(timer, &mut self.ioregs.timers, &self.scheduler);
        }
    }

    fn read_timer_counter(&mut self, timer: usize) -> u16 {
        crate::timers::flush(&mut self.ioregs.timers[timer], self.ioregs.time);
        self.ioregs.timers[timer].counter()
    }

    fn write_to_dma_control(&mut self, dma: usize, value: u16) {
        use crate::dma;

        let old_value = self.ioregs.dma[dma].control;
        self.ioregs.dma[dma].control.set_preserve_bits(value);

        if !old_value.enabled() && self.ioregs.dma[dma].control.enabled() {
            let (event_fn, event_tag): (EventFn, EventTag) = match dma {
                0 => (dma::dma_enabled::<0>, EventTag::DMA0),
                1 => (dma::dma_enabled::<1>, EventTag::DMA1),
                2 => (dma::dma_enabled::<2>, EventTag::DMA2),
                3 => (dma::dma_enabled::<3>, EventTag::DMA3),
                _ => unreachable!("invalid DMA index"),
            };

            self.scheduler.schedule(event_fn, 0, event_tag);
        }
    }

    /// Copies the reference point registers (BG2X, BG2Y, BG3X, BG3Y) into
    /// the internal reference point registers that are actually used while rendering.
    pub fn copy_reference_points(&mut self) {
        self.ioregs.bg2x_internal = self.ioregs.bg2x.into();
        self.ioregs.bg2y_internal = self.ioregs.bg2y.into();
        self.ioregs.bg3x_internal = self.ioregs.bg3x.into();
        self.ioregs.bg3y_internal = self.ioregs.bg3y.into();
    }

    pub fn increment_reference_points(&mut self) {
        self.ioregs.bg2x_internal += FixedPoint32::from(self.ioregs.bg2pb);
        self.ioregs.bg2y_internal += FixedPoint32::from(self.ioregs.bg2pd);
        self.ioregs.bg3x_internal += FixedPoint32::from(self.ioregs.bg3pb);
        self.ioregs.bg3y_internal += FixedPoint32::from(self.ioregs.bg3pd);
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
    pub(crate) bg2pa: FixedPoint16,
    pub(crate) bg2pb: FixedPoint16,
    pub(crate) bg2pc: FixedPoint16,
    pub(crate) bg2pd: FixedPoint16,
    pub(crate) bg2x: FixedPoint28,
    pub(crate) bg2y: FixedPoint28,
    pub(crate) bg3pa: FixedPoint16,
    pub(crate) bg3pb: FixedPoint16,
    pub(crate) bg3pc: FixedPoint16,
    pub(crate) bg3pd: FixedPoint16,
    pub(crate) bg3x: FixedPoint28,
    pub(crate) bg3y: FixedPoint28,
    pub(crate) winhv: WindowDimensions,
    pub(crate) wininout: WindowInOut,
    pub(crate) mosaic: MosaicSize,
    pub(crate) bldcnt: ColorSpecialEffects,
    pub(crate) bldalpha: AlphaBlendingCoeff,
    pub(crate) bldy: BrightnessCoeff,

    pub(crate) bg2x_internal: FixedPoint32,
    pub(crate) bg2y_internal: FixedPoint32,
    pub(crate) bg3x_internal: FixedPoint32,
    pub(crate) bg3y_internal: FixedPoint32,

    // Sound
    pub(crate) sound1cnt_l: SweepControl,
    pub(crate) sound1cnt_h: DutyLenEnvelope,
    pub(crate) sound1cnt_x: FreqControl,
    pub(crate) sound2cnt_l: DutyLenEnvelope,
    pub(crate) sound2cnt_h: FreqControl,
    pub(crate) sound3cnt_l: StopWaveRamSelect,
    pub(crate) sound3cnt_h: LengthVolume,
    pub(crate) sound3cnt_x: FreqControl,
    pub(crate) sound4cnt_l: LengthEnvelope,
    pub(crate) sound4cnt_h: NoiseFreqControl,
    pub(crate) soundcnt_l: ChannelLRVolumeEnable,
    pub(crate) soundcnt_h: DMASoundControlMixing,
    pub(crate) soundcnt_x: SoundOnOff,
    pub(crate) soundbias: SoundBias,
    pub(crate) waveram: WaveRam,
    pub(crate) fifo_a: Fifo,
    pub(crate) fifo_b: Fifo,

    // DMA
    pub(crate) dma: [DMARegisters; 4],

    // Timers
    pub(crate) timers: [Timer; 4],

    /// This is the current time counted in cycles.
    /// This is NOT a register but it makes the most sense to use it here
    /// as it is used as the internal clock for timers.
    pub(crate) time: u64,

    // Keypad Input
    pub(crate) keyinput: u16,

    // Interrupt, Waitstate, and Power-Down Control
    pub(crate) ie_reg: InterruptEnable,
    pub(crate) if_reg: InterruptReqAck,
    pub(crate) waitcnt: WaitstateControl,
    pub(crate) ime: InterruptMasterEnable,
    pub(crate) postflg: PostBoot,

    /// This is NOT a register but a temporary location for pending interrupts.
    pub(crate) irq_pending: InterruptReqAck,
}

impl IoRegisters {
    pub fn init(&mut self) {
        self.keyinput = 0x3ff;
    }
}
