use util::{bitfields, bits::Bits, circular::CircularBuffer, primitive_enum};

bitfields! {
    /// *** 4000060h - SOUND1CNT_L (NR10) - Channel 1 Sweep register (R/W) ***
    /// Bit        Expl.
    /// 0-2   R/W  Number of sweep shift      (n=0-7)
    /// 3     R/W  Sweep Frequency Direction  (0=Increase, 1=Decrease)
    /// 4-6   R/W  Sweep Time; units of 7.8ms (0-7, min=7.8ms, max=54.7ms)
    /// 7-15  -    Not used
    pub struct SweepControl: u16 {
        [0,2]   shifts, set_shifts: u16,
        [4,6]   sweep_time, set_sweep_time: u16,
    }
}

impl SweepControl {
    pub fn direction(&self) -> Direction {
        // NOTE: direction is reversed for this register relative
        //       to its other uses.
        Direction::from(1 - self.value.bit(3))
    }

    pub fn set_direction(&mut self, direction: Direction) {
        // NOTE: direction is reversed for this register relative
        //       to its other uses.
        self.value = self.value.replace_bit(3, u16::from(direction) == 0);
    }
}

bitfields! {
    /// *** 4000062h - SOUND1CNT_H (NR11, NR12) - Channel 1 Duty/Length/Envelope (R/W) ***
    /// *** 4000068h - SOUND2CNT_L (NR21, NR22) - Channel 2 Duty/Length/Envelope (R/W) ***
    /// Bit        Expl.
    /// 0-5   W    Sound length; units of (64-n)/256s  (0-63)
    /// 6-7   R/W  Wave Pattern Duty                   (0-3, see below)
    /// 8-10  R/W  Envelope Step-Time; units of n/64s  (1-7, 0=No Envelope)
    /// 11    R/W  Envelope Direction                  (0=Decrease, 1=Increase)
    /// 12-15 R/W  Initial Volume of envelope          (1-15, 0=No Sound)
    pub struct DutyLenEnvelope: u16 {
        [0,5]   length, set_length: u16,
        [6,7]   wave_pattern_duty, set_wave_pattern_duty: u16,
        [8,10]  envelope_step_time, set_envelope_step_time: u16,
        [11]    envelope_direction, set_envelope_direction: Direction,
        [12,15] initial_envelope_volume, set_initial_envelope_volume: u16,
    }
}

bitfields! {
    /// 4000064h - SOUND1CNT_X (NR13, NR14) - Channel 1 Frequency/Control (R/W)
    /// 400006Ch - SOUND2CNT_H (NR23, NR24) - Channel 2 Frequency/Control (R/W)
    ///   Bit        Expl.
    ///   0-10  W    Frequency; 131072/(2048-n)Hz  (0-2047)
    ///   11-13 -    Not used
    ///   14    R/W  Length Flag  (1=Stop output when length in NR11 expires)
    ///   15    W    Initial      (1=Restart Sound)
    ///   16-31 -    Not used
    pub struct FreqControl: u32 {
        [0,10]  freq_setting, set_freq_setting: u16,
        [14]    length_flag, set_length_flag: bool,
        [15]    initial, set_initial: bool,
        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,
    }
}

bitfields! {
    /// 4000070h - SOUND3CNT_L (NR30) - Channel 3 Stop/Wave RAM select (R/W)
    /// Bit        Expl.
    /// 0-4   -    Not used
    /// 5     R/W  Wave RAM Dimension   (0=One bank/32 digits, 1=Two banks/64 digits)
    /// 6     R/W  Wave RAM Bank Number (0-1, see below)
    /// 7     R/W  Sound Channel 3 Off  (0=Stop, 1=Playback)
    /// 8-15  -    Not used
    pub struct StopWaveRamSelect: u16 {
        [5]     dimension, set_dimension: Dimension,
        [6]     bank_number, set_bank_number: u16,
        [7]     playback, set_playback: bool,
    }
}

bitfields! {
    /// 4000072h - SOUND3CNT_H (NR31, NR32) - Channel 3 Length/Volume (R/W)
    /// Bit        Expl.
    /// 0-7   W    Sound length; units of (256-n)/256s  (0-255)
    /// 8-12  -    Not used.
    /// 13-14 R/W  Sound Volume  (0=Mute/Zero, 1=100%, 2=50%, 3=25%)
    /// 15    R/W  Force Volume  (0=Use above, 1=Force 75% regardless of above)
    pub struct LengthVolume: u16 {
        [0,7]   length, set_length: u16,
    }
}

impl LengthVolume {
    pub fn volume(&self) -> u32 {
        match (self.value.bit(15), self.value.bits(13, 14)) {
            (1, _) => 75,
            (0, 0) => 0,
            (0, 1) => 100,
            (0, 2) => 50,
            (0, 3) => 25,
            _ => unreachable!(),
        }
    }
}

/// Swaps the nibbles of each byte in a u16.
fn swap_nibbles16(n: u16) -> u16 {
    const MASK: u16 = 0x0F0F;
    ((n >> 4) & MASK) | ((n & MASK) << 4)
}

#[derive(Copy, Clone, Default)]
pub struct WaveRam(u128);

impl WaveRam {
    pub fn store16(&mut self, index: u32, value: u16) {
        self.0 |= (swap_nibbles16(value) as u128) << (index * 16);
    }

    pub fn load16(&mut self, index: u32) -> u16 {
        swap_nibbles16((self.0 >> (index * 16)) as u16)
    }
}

bitfields! {
    /// 4000078h - SOUND4CNT_L (NR41, NR42) - Channel 4 Length/Envelope (R/W)
    /// Bit        Expl.
    /// 0-5   W    Sound length; units of (64-n)/256s  (0-63)
    /// 6-7   -    Not used
    /// 8-10  R/W  Envelope Step-Time; units of n/64s  (1-7, 0=No Envelope)
    /// 11    R/W  Envelope Direction                  (0=Decrease, 1=Increase)
    /// 12-15 R/W  Initial Volume of envelope          (1-15, 0=No Sound)
    /// 16-31 -    Not used
    pub struct LengthEnvelope: u32 {
        [0,5]   length, set_length: u16,
        [8,10]  envelope_step_time, set_envelope_step_time: u16,
        [11]    envelope_direction, set_envelope_direction: Direction,
        [12,15] initial_volume, set_initial_volume: u16,

        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,
    }
}

bitfields! {
    /// 400007Ch - SOUND4CNT_H (NR43, NR44) - Channel 4 Frequency/Control (R/W)
    /// The amplitude is randomly switched between high and low at the given frequency.
    /// A higher frequency will make the noise to appear 'softer'.
    /// When Bit 3 is set, the output will become more regular, and some frequencies will
    /// sound more like Tone than Noise.
    /// Bit        Expl.
    /// 0-2   R/W  Dividing Ratio of Frequencies (r)
    /// 3     R/W  Counter Step/Width (0=15 bits, 1=7 bits)
    /// 4-7   R/W  Shift Clock Frequency (s)
    /// 8-13  -    Not used
    /// 14    R/W  Length Flag  (1=Stop output when length in NR41 expires)
    /// 15    W    Initial      (1=Restart Sound)
    /// 16-31 -    Not used
    pub struct NoiseFreqControl: u32 {
        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,
    }
}

#[derive(Default)]
pub struct Fifo {
    buffer: CircularBuffer<u8, 32>,
}

impl Fifo {
    pub fn store16(&mut self, value: u16) {
        self.store8(value as u8);
        self.store8((value >> 8) as u8);
    }

    pub fn store8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn pop_sample(&mut self) -> u8 {
        self.buffer.pop().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

bitfields! {
    /// 4000080h - SOUNDCNT_L (NR50, NR51) - Channel L/R Volume/Enable (R/W)
    /// Bit        Expl.
    /// 0-2   R/W  Sound 1-4 Master Volume RIGHT (0-7)
    /// 3     -    Not used
    /// 4-6   R/W  Sound 1-4 Master Volume LEFT (0-7)
    /// 7     -    Not used
    /// 8-11  R/W  Sound 1-4 Enable Flags RIGHT (each Bit 8-11, 0=Disable, 1=Enable)
    /// 12-15 R/W  Sound 1-4 Enable Flags LEFT (each Bit 12-15, 0=Disable, 1=Enable)
    pub struct ChannelLRVolumeEnable: u16 {
        [0,2]   master_volume_right, set_master_volume_right: u16,
        [4,6]   master_volme_left, set_master_volume_left: u16,
    }
}

impl ChannelLRVolumeEnable {
    pub fn enable_right(&self, channel: PSGChannel) -> bool {
        self.value.is_bit_set(u32::from(channel) + 8)
    }

    pub fn enable_left(&self, channel: PSGChannel) -> bool {
        self.value.is_bit_set(u32::from(channel) + 12)
    }
}

bitfields! {
    /// 4000082h - SOUNDCNT_H (GBA only) - DMA Sound Control/Mixing (R/W)
    /// Bit        Expl.
    /// 0-1   R/W  Sound # 1-4 Volume   (0=25%, 1=50%, 2=100%, 3=Prohibited)
    /// 2     R/W  DMA Sound A Volume   (0=50%, 1=100%)
    /// 3     R/W  DMA Sound B Volume   (0=50%, 1=100%)
    /// 4-7   -    Not used
    /// 8     R/W  DMA Sound A Enable RIGHT (0=Disable, 1=Enable)
    /// 9     R/W  DMA Sound A Enable LEFT  (0=Disable, 1=Enable)
    /// 10    R/W  DMA Sound A Timer Select (0=Timer 0, 1=Timer 1)
    /// 11    W?   DMA Sound A Reset FIFO   (1=Reset)
    /// 12    R/W  DMA Sound B Enable RIGHT (0=Disable, 1=Enable)
    /// 13    R/W  DMA Sound B Enable LEFT  (0=Disable, 1=Enable)
    /// 14    R/W  DMA Sound B Timer Select (0=Timer 0, 1=Timer 1)
    /// 15    W?   DMA Sound B Reset FIFO   (1=Reset)
    pub struct DMASoundControlMixing: u16 {
        [0,1]   analogue_volume, set_analogue_volume: u16,
    }
}

impl DMASoundControlMixing {
    pub fn dma_volume(&self, channel: FifoChannel) -> u16 {
        self.value.bit(u32::from(channel) + 2)
    }

    pub fn dma_enable(&self, channel: FifoChannel) -> bool {
        let start = u32::from(channel) * 4 + 8;
        self.value.bits(start, start + 1) != 0
    }

    pub fn dma_enable_right(&self, channel: FifoChannel) -> bool {
        self.value.is_bit_set(u32::from(channel) * 4 + 8)
    }

    pub fn dma_enable_left(&self, channel: FifoChannel) -> bool {
        self.value.is_bit_set(u32::from(channel) * 4 + 9)
    }

    pub fn dma_timer_select(&self, channel: FifoChannel) -> usize {
        self.value.bit(u32::from(channel) * 4 + 10) as usize
    }

    pub fn dma_reset_fifo(&self, channel: FifoChannel) -> bool {
        self.value.is_bit_set(u32::from(channel) * 4 + 11)
    }
}

bitfields! {
    /// 4000084h - SOUNDCNT_X (NR52) - Sound on/off (R/W)
    /// Bits 0-3 are automatically set when starting sound output, and are automatically cleared when a sound ends. (Ie. when the length expires, as far as length is enabled. The bits are NOT reset when an volume envelope ends.)
    /// Bit        Expl.
    /// 0     R    Sound 1 ON flag (Read Only)
    /// 1     R    Sound 2 ON flag (Read Only)
    /// 2     R    Sound 3 ON flag (Read Only)
    /// 3     R    Sound 4 ON flag (Read Only)
    /// 4-6   -    Not used
    /// 7     R/W  PSG/FIFO Master Enable (0=Disable, 1=Enable) (Read/Write)
    /// 8-31  -    Not used
    pub struct SoundOnOff: u32 {
        [7]     master_enable, set_master_enable: bool,
        [0,15]  lo, set_lo: u16,
        [16,31] hi, set_hi: u16,

        readonly = 0x0000000F
    }
}

impl SoundOnOff {
    pub fn sound_on(&self, channel: PSGChannel) -> bool {
        self.value.is_bit_set(u32::from(channel))
    }

    pub fn set_sound_on(&mut self, channel: PSGChannel, on: bool) {
        self.value = self.value.replace_bit(u32::from(channel), on);
    }
}

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
    /// Programmable sound generator channel (Sound 1-4)
    pub enum PSGChannel: u16 (u32) {
        Sound1 = 0,
        Sound2,
        Sound3,
        Sound4,
    }
}

primitive_enum! {
    pub enum FifoChannel: u16 (u32) {
        A,
        B,
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

impl Resolution {
    pub fn frequency(&self) -> u32 {
        match self {
            Resolution::Res9bit32khz => 32 * 1024,
            Resolution::Res8Bit64khz => 64 * 1024,
            Resolution::Res7Bit128khz => 128 * 1024,
            Resolution::Res6bit256khz => 256 * 1024,
        }
    }

    pub fn bit_depth(&self) -> u32 {
        match self {
            Resolution::Res9bit32khz => 9,
            Resolution::Res8Bit64khz => 8,
            Resolution::Res7Bit128khz => 7,
            Resolution::Res6bit256khz => 6,
        }
    }
}

primitive_enum! {
    pub enum Direction: u16 (u32) {
        Decreasing = 0,
        Increasing
    }
}

primitive_enum! {
    pub enum Dimension: u16 {
        OneBank,
        TwoBanks,
    }
}
