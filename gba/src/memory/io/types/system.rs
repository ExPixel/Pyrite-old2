use util::bitfields;

bitfields! {
    /// ***000300h - POSTFLG - BYTE - Undocumented - Post Boot / Debug Control (R/W)***
    /// After initial reset, the GBA BIOS initializes the register to 01h, and any further execution of the
    /// Reset vector (00000000h) will pass control to the Debug vector (0000001Ch) when sensing the register to be still set to 01h.
    ///
    /// Bit   Expl.
    /// 0     Undocumented. First Boot Flag  (0=First, 1=Further)
    /// 1-7   Undocumented. Not used.
    pub struct PostBoot: u8 {
        [0]     not_first_boot, set_not_first_boot: bool,
    }
}

bitfields! {
    /// ***4000301h - HALTCNT - BYTE - Undocumented - Low Power Mode Control (W)***
    /// Writing to this register switches the GBA into battery saving mode.
    /// In Halt mode, the CPU is paused as long as (IE AND IF)=0, this should be used to reduce
    /// power-consumption during periods when the CPU is waiting for interrupt events.
    /// In Stop mode, most of the hardware including sound and video are paused,
    /// this very-low-power mode could be used much like a screensaver.
    ///
    /// Bit   Expl.
    /// 0-6   Undocumented. Not used.
    /// 7     Undocumented. Power Down Mode  (0=Halt, 1=Stop)
    pub struct LowPowerModeControl: u8 {
        [7]     stop, set_stop: bool,
    }
}

bitfields! {
    /// ***4000301h - HALTCNT - BYTE - Undocumented - Low Power Mode Control (W)***
    /// Writing to this register switches the GBA into battery saving mode.
    /// In Halt mode, the CPU is paused as long as (IE AND IF)=0, this should be used to reduce
    /// power-consumption during periods when the CPU is waiting for interrupt events.
    /// In Stop mode, most of the hardware including sound and video are paused,
    /// this very-low-power mode could be used much like a screensaver.
    ///
    /// Bit   Expl.
    /// 0-6   Undocumented. Not used.
    /// 7     Undocumented. Power Down Mode  (0=Halt, 1=Stop)
    pub struct WaitstateControl: u16 {
        readonly = 0x8000
    }
}
