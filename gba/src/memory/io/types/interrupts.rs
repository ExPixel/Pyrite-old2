use util::{bitfields, bits::Bits, primitive_enum};

bitfields! {
    /// 4000208h - IME - Interrupt Master Enable Register (R/W)
    ///   Bit   Expl.
    ///   0     Disable all interrupts         (0=Disable All, 1=See [`InterruptEnable`])
    ///   1-31  Not used
    pub struct InterruptMasterEnable: u16 {
        [0] enabled, set_enabled: bool,
    }
}

bitfields! {
    pub struct InterruptEnable: u16 {}
}

impl InterruptEnable {
    pub fn enabled(&self, interrupt: Interrupt) -> bool {
        self.value.is_bit_set(u8::from(interrupt) as u32)
    }

    pub fn set_enabled(&self, interrupt: Interrupt, enabled: bool) {
        self.value.replace_bit(u8::from(interrupt) as u32, enabled);
    }
}

bitfields! {
    pub struct InterruptReqAck: u16 {}
}

impl InterruptReqAck {
    /// Interrupts can be acknowledged by writing a one to one of the IRQ bits.
    pub fn set_acknowledge(&mut self, value: u16) {
        self.value &= !value;
    }
}

primitive_enum! {
    pub enum Interrupt: u8 {
        VBlank = 0,
        HBlank,
        VCounterMatch,
        Timer0Overflow,
        Timer1Overflow,
        Timer2Overflow,
        Timer3Overflow,
        SerialCommunication,
        DMA0,
        DMA1,
        DMA2,
        DMA3,
        Keypad,
        GamePak,
    }
}
