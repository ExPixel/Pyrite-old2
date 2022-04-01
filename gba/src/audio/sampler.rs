use crate::memory::io::FifoChannel;

use super::Command;

pub struct GbaAudioSampler {
    native_frequency: f64,

    // FIFO samples
    fifo_a: i8,
    fifo_b: i8,
    bias: i16,
    wait_frames: f64,
}

impl GbaAudioSampler {
    pub fn new(native_frequency: u32) -> Self {
        GbaAudioSampler {
            native_frequency: native_frequency as f64,
            fifo_a: 0,
            fifo_b: 0,
            bias: 0x100,
            wait_frames: 0.0,
        }
    }

    fn generate_output(&mut self) -> (f32, f32) {
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
        let psg_l = [0i16; 4];
        let psg_r = [0i16; 4];

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
        self.generate_output()
    }

    fn wait_cycles(&mut self, cycles: u32) {
        const GBA_FREQ: f64 = 1.0 / (16.0 * 1024.0 * 1024.0);
        self.wait_frames += (cycles as f64 * self.native_frequency) * GBA_FREQ;
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
                log::debug!("GBA frequency changed to {}", resolution.frequency());
            }
        }
    }

    pub fn needs_commands(&self) -> bool {
        self.wait_frames < 0.5
    }
}
