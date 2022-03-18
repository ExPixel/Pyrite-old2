use util::{bitfields, primitive_enum};

bitfields! {
    /// 4000088h - SOUNDBIAS - Sound PWM Control (R/W, see below)
    /// This register controls the final sound output. The default setting is 0200h, it is normally not required to change this value.
    /// Bit        Expl.
    /// 0     -    Not used
    /// 1-9   R/W  Bias Level (Default=100h, converting signed samples into unsigned)
    /// 10-13 -    Not used
    /// 14-15 R/W  Amplitude Resolution/Sampling Cycle (Default=0, see below)
    /// 16-31 -    Not used
    pub struct SoundBias: u32 {
        [1,9]   bias, set_bias: u16,
        [14,15] resolution, set_resolution: Resolution,
        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,
    }
}

primitive_enum! {
    pub enum Resolution: u8 (u32) {
        Res9bit32khz,
        Res8Bit64khz,
        Res7Bit128khz,
        Res6bit256khz,
    }
}
