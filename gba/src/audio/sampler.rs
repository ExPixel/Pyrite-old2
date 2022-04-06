use crate::memory::io::{FifoChannel, PSGChannel};

use super::Command;

pub struct GbaAudioSampler {
    native_frequency: u32,
    native_frequency_f: f64,

    // FIFO samples
    fifo_a: i8,
    fifo_b: i8,

    // PSG outputs BEFORE volumes for left and right channels are
    // applied. This DOES take into the account the master volume
    // that is set in the DMA Sound Control/Mixing register.
    bias: i16,
    wait_frames: f64,

    sound1: SquareWave,
    sound2: SquareWave,
}

impl GbaAudioSampler {
    pub fn new(native_frequency: u32) -> Self {
        GbaAudioSampler {
            native_frequency,
            native_frequency_f: native_frequency as f64,
            fifo_a: 0,
            fifo_b: 0,
            bias: 0x100,
            wait_frames: 0.0,

            sound1: SquareWave::default(),
            sound2: SquareWave::default(),
        }
    }

    fn generate_output_frame(&mut self) -> (f32, f32) {
        const GBA_RANGE_RECIP: f32 = 1.0 / 1024.0;

        // Each of the two FIFOs can span the FULL output range (+/-200h).
        let fifo_a = (self.fifo_a as i16) << 2;
        let fifo_b = (self.fifo_b as i16) << 2;

        // FIXME implement volume for FIFO
        let fifo_a_l = fifo_a;
        let fifo_a_r = fifo_a;
        let fifo_b_l = fifo_b;
        let fifo_b_r = fifo_b;

        // Each of the four PSGs can span one QUARTER of the output range (+/-80h).
        // FIXME implement PSG output
        let psg = [self.sound1.frame(), self.sound2.frame(), 0, 0];
        let mut psg_l = [0i16; 4];
        let mut psg_r = [0i16; 4];

        // FIXME temporary, eventually this will apply volume to each element first.
        psg_l[..4].clone_from_slice(&psg[..4]);
        psg_r[..4].clone_from_slice(&psg[..4]);

        let psg_l: i16 = psg_l.iter().sum();
        let psg_r: i16 = psg_r.iter().sum();

        // The current output levels of all six channels are added together by hardware.
        // So together, the FIFOs and PSGs, could reach THRICE the range (+/-600h).
        //
        // The BIAS value is added to that signed value. With default BIAS (200h),
        // the possible range becomes -400h..+800h.
        //
        // Values that exceed the unsigned 10bit output range of 0..3FFh are clipped to MinMax(0,3FFh).
        let gba_out_l = (fifo_a_l + fifo_b_l + psg_l + self.bias).clamp(0, 0x3FF);
        let gba_out_r = (fifo_a_r + fifo_b_r + psg_r + self.bias).clamp(0, 0x3FF);

        let out_l = gba_out_l as f32 * GBA_RANGE_RECIP;
        let out_r = gba_out_r as f32 * GBA_RANGE_RECIP;
        (out_l, out_r)
    }

    pub fn frame(&mut self) -> (f32, f32) {
        if self.wait_frames >= 0.0 {
            self.wait_frames -= 1.0;
        }
        self.generate_output_frame()
    }

    fn wait_cycles(&mut self, cycles: u32) {
        const GBA_FREQ: f64 = 1.0 / (16.0 * 1024.0 * 1024.0);
        self.wait_frames += (cycles as f64 * self.native_frequency_f) * GBA_FREQ;
    }

    pub fn command(&mut self, command: Command) {
        match command {
            Command::Wait(cycles) if cycles != 0 => self.wait_cycles(cycles),
            Command::Wait(_) => { /* 0 cycles = NOP */ }
            Command::PlaySample { channel, sample } => {
                if channel == FifoChannel::A {
                    self.fifo_a = sample;
                } else {
                    self.fifo_b = sample;
                }
            }
            Command::SetBias(bias) => self.bias = bias as i16,
            Command::SetResolution(resolution) => {
                // FIXME: Frequency is currently ignored. Should it continue to be this way?
                let frequency = resolution.frequency();
                let bits = resolution.bit_depth();
                log::debug!("GBA resolution change: frequency={frequency}, bit-depth={bits}");
            }

            Command::SetPSGEnabled(chan, enabled) => match chan {
                PSGChannel::Sound1 => self.sound1.enabled = enabled,
                PSGChannel::Sound2 => self.sound2.enabled = enabled,
                PSGChannel::Sound3 => log::debug!("SetPSGEnabled(3)"),
                PSGChannel::Sound4 => log::debug!("SetPSGEnabled(4)"),
            },

            Command::SetPSGFrequencyRate(chan, rate) => match chan {
                PSGChannel::Sound1 => self
                    .sound1
                    .set_frequency_rate(rate as u32, self.native_frequency),
                PSGChannel::Sound2 => self
                    .sound2
                    .set_frequency_rate(rate as u32, self.native_frequency),
                _ => unreachable!(),
            },

            Command::SetPSGDuty(chan, duty) => match chan {
                PSGChannel::Sound1 => self.sound1.set_duty(duty),
                PSGChannel::Sound2 => self.sound2.set_duty(duty),
                _ => unreachable!(),
            },

            Command::SetPSGEnvelopeVolume(chan, volume) => match chan {
                PSGChannel::Sound1 => self.sound1.set_volume(volume as i16),
                PSGChannel::Sound2 => self.sound2.set_volume(volume as i16),
                PSGChannel::Sound3 => log::debug!("SetPSGVolume(3)"),
                PSGChannel::Sound4 => log::debug!("SetPSGVolume(4)"),
            },
        }
    }

    pub fn needs_commands(&self) -> bool {
        self.wait_frames < 0.5
    }
}

#[derive(Default)]
struct SquareWave {
    frequency_rate: u32,
    native_frequency: u32,
    phase: u32,
    phase_inc: u32,
    duty: u32,
    enabled: bool,
    volume: i16,
    output: i16,
}

impl SquareWave {
    fn set_frequency_rate(&mut self, rate: u32, native_frequency: u32) {
        debug_assert!(rate < 2048);
        let frequency = 131072 / (2048 - rate);
        self.frequency_rate = rate;
        self.native_frequency = native_frequency;
        self.phase_inc = (u32::MAX / native_frequency) * frequency;
    }

    fn set_volume(&mut self, volume: i16) {
        self.volume = volume;
        self.output = (0x80 * volume) / 15;
    }

    /// Wave Duty:  
    /// 0: 12.5% ( -_______-_______-_______ )  
    /// 1: 25%   ( --______--______--______ )  
    /// 2: 50%   ( ----____----____----____ ) (normal)  
    /// 3: 75%   ( ------__------__------__ )  
    fn set_duty(&mut self, duty: u16) {
        self.duty = match duty {
            0 => 0x1fffffff,
            1 => 0x3fffffff,
            2 => 0x7fffffff,
            3 => 0xbfffffff,
            _ => unreachable!("invalid wave duty"),
        };
    }

    /// Returns `volume` if the square wave is high for this frame, or 0 if the
    /// square wave is currently low.
    fn frame(&mut self) -> i16 {
        let output = if self.enabled && self.phase <= self.duty {
            self.output
        } else {
            0
        };
        self.phase = self.phase.wrapping_add(self.phase_inc);
        output
    }
}
