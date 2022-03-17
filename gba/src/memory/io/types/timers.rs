use util::{bitfields, bits::Bits};

#[derive(Default)]
pub struct Timer {
    pub(crate) reload: u16,
    pub(crate) control: TimerControl,
    pub(crate) counter: u16,
    pub(crate) origin: u64,
}

impl Timer {
    pub fn counter(&self) -> u16 {
        self.counter
    }

    pub fn set_reload(&mut self, value: u16) {
        self.reload = value;
    }
}

bitfields! {
    /// 4000102h - TM0CNT_H - Timer 0 Control (R/W)
    /// 4000106h - TM1CNT_H - Timer 1 Control (R/W)
    /// 400010Ah - TM2CNT_H - Timer 2 Control (R/W)
    /// 400010Eh - TM3CNT_H - Timer 3 Control (R/W)
    /// Bit   Expl.
    /// 0-1   Prescaler Selection (0=F/1, 1=F/64, 2=F/256, 3=F/1024)
    /// 2     Count-up Timing   (0=Normal, 1=See below)  ;Not used in TM0CNT_H
    /// 3-5   Not used
    /// 6     Timer IRQ Enable  (0=Disable, 1=IRQ on Timer overflow)
    /// 7     Timer Start/Stop  (0=Stop, 1=Operate)
    /// 8-15  Not used
    pub struct TimerControl: u16 {
        [2]     count_up_timing, set_count_up_timing: bool,
        [6]     irq_enable, set_irq_enable: bool,
        [7]     started, set_started: bool,
    }
}

impl TimerControl {
    pub fn prescaler(&self) -> u16 {
        match self.value.bits(0, 1) {
            0 => 1,
            1 => 64,
            2 => 256,
            3 => 1024,
            _ => unreachable!(),
        }
    }

    pub fn prescaler_shift(&self) -> u32 {
        match self.value.bits(0, 1) {
            0 => 0,
            1 => 6,
            2 => 8,
            3 => 10,
            _ => unreachable!(),
        }
    }
}
