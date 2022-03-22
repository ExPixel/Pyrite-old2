use util::{bitfields, bits::Bits, primitive_enum};

bitfields! {
    /// 4000208h - IME - Interrupt Master Enable Register (R/W)
    ///   Bit   Expl.
    ///   0     Disable all interrupts         (0=Disable All, 1=See [`InterruptEnable`])
    ///   1-31  Not used
    pub struct InterruptMasterEnable: u32 {
        [0]     enabled, set_enabled: bool,

        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,
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
    pub fn request(&mut self, interrupt: Interrupt) {
        self.value |= 1 << u8::from(interrupt);
    }

    /// Inherits IRQ requests from another [`InterruptReqAck`]
    pub fn inherit(&mut self, other: InterruptReqAck) {
        self.set_preserve_bits(self.value | other.value);
    }

    pub fn clear(&mut self) {
        self.value = self.value.replace_bits(0, 13, 0);
    }

    pub fn has_requests(&self) -> bool {
        self.value.bits(0, 13) != 0
    }

    /// Interrupts can be acknowledged by writing a one to one of the IRQ bits.
    pub fn write(&mut self, value: u16) {
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

impl Interrupt {
    pub fn timer(timer: usize) -> Interrupt {
        match timer {
            0 => Interrupt::Timer0Overflow,
            1 => Interrupt::Timer1Overflow,
            2 => Interrupt::Timer2Overflow,
            3 => Interrupt::Timer3Overflow,
            _ => unreachable!("invalid timer"),
        }
    }
}
