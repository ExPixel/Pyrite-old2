use crate::memory::io::FifoChannel;

use super::Command;

pub struct GbaAudioSampler {
    native_frequency: f64,
    fifo_a: f32,
    fifo_b: f32,
    wait_frames: f64,
}

impl GbaAudioSampler {
    pub fn new(native_frequency: u32) -> Self {
        GbaAudioSampler {
            native_frequency: native_frequency as f64,
            fifo_a: 0.0,
            fifo_b: 0.0,
            wait_frames: 0.0,
        }
    }

    pub fn frame(&mut self, volume_coeff: f32) -> (f32, f32) {
        if self.wait_frames >= 0.0 {
            self.wait_frames -= 1.0;
        }
        (self.fifo_a * volume_coeff, self.fifo_a * volume_coeff)
    }

    fn wait_cycles(&mut self, cycles: u32) {
        const GBA_FREQ: f64 = 1.0 / (16.0 * 1024.0 * 1024.0);
        self.wait_frames += (cycles as f64 * self.native_frequency) * GBA_FREQ;
    }

    pub fn command(&mut self, command: Command) {
        match command {
            Command::Wait(cycles) if cycles != 0 => self.wait_cycles(cycles),
            Command::Wait(_) => { /* 0 cycles = NOP */ }
            Command::PlaySample(fifo, sample) => {
                const CONVERT: f32 = 1.0 / 128.0;
                let sample: f32 = (sample as f32 * CONVERT).clamp(-1.0, 1.0);
                if fifo == FifoChannel::A {
                    self.fifo_a = sample;
                } else {
                    self.fifo_b = sample;
                }
            }
            Command::SetResolution(resolution) => {
                // FIXME: This is currently ignored. Should it continue to be this way?
                log::debug!("GBA frequency changed to {}", resolution.frequency());
            }
        }
    }

    pub fn needs_commands(&self) -> bool {
        self.wait_frames < 1.0
    }
}
