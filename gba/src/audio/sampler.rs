use crate::memory::io::FifoChannel;

use super::Command;

pub struct GbaAudioSampler {
    native_frequency: u32,
    frequency: u32,
    fifo_a: f32,
    fifo_b: f32,
    wait_frames: u32,
}

impl GbaAudioSampler {
    pub fn new(native_frequency: u32) -> Self {
        assert!(native_frequency.is_power_of_two());
        GbaAudioSampler {
            native_frequency,
            frequency: 32768,
            fifo_a: 0.0,
            fifo_b: 0.0,
            wait_frames: 0,
        }
    }

    pub fn frame(&mut self) -> (f32, f32) {
        self.wait_frames = self.wait_frames.saturating_sub(1);
        (self.fifo_a, self.fifo_a)
    }

    fn wait_cycles(&mut self, cycles: u32) {
        const GBA_FREQ: u64 = 16 * 1024 * 1024;
        let wait_frames = (cycles as u64 * self.frequency as u64) / GBA_FREQ;
        self.wait_frames =
            ((wait_frames * self.native_frequency as u64) / self.frequency as u64) as u32;
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
                self.frequency = resolution.frequency();
                log::debug!("sampler frequency changed to {}", self.frequency);
            }
        }
    }

    pub fn needs_commands(&self) -> bool {
        self.wait_frames == 0
    }
}
