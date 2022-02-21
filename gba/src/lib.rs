mod memory;
mod scheduler;
mod video;

pub use memory::GbaMemory;

use arm::Cpu;
use scheduler::Scheduler;
use video::GbaVideo;

pub struct Gba {
    mem: GbaMemory,
    cpu: Cpu,
    video: GbaVideo,
    scheduler: Scheduler,
}

impl Gba {
    pub fn new() -> Gba {
        let scheduler = Scheduler::default();

        Gba {
            mem: GbaMemory::new(),
            cpu: Cpu::uninitialized(arm::Isa::Arm, arm::CpuMode::System),
            video: GbaVideo::new(scheduler.clone()),
            scheduler,
        }
    }

    pub fn reset(&mut self) {
        self.mem.init();
        self.video.init(&mut self.mem);
        self.cpu.branch(0x08000000, &mut self.mem);
    }

    pub fn set_gamepak(&mut self, cart: Vec<u8>) {
        self.mem.set_gamepak(cart);
    }

    pub fn frame(&mut self, video: &mut dyn VideoOutput, _audio: &mut dyn AudioOutput) {
        static mut VALUE: u16 = 0;
        static mut ADD: bool = true;

        let value = unsafe { VALUE };
        unsafe {
            if ADD {
                VALUE += 1;

                if VALUE == 31 {
                    ADD = false;
                }
            } else {
                VALUE -= 1;

                if VALUE == 0 {
                    ADD = true;
                }
            }
        };

        let line_data = [(value << 10); 240];
        for line_idx in 0..160 {
            video.output_line(line_idx, &line_data);
        }
    }

    pub fn step(&mut self, _video: &mut dyn VideoOutput, _audio: &mut dyn AudioOutput) {
        let mut cycles = self.cpu.step(&mut self.mem);

        while let Some((event, next_cycles)) = self.scheduler.advance(cycles) {
            cycles = next_cycles;
            (event)(self, cycles);
        }
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

pub trait VideoOutput {
    /// Called when the GBA has a line ready to be output to the screen.
    /// At line 239 (240th line), a frame is ready to be output to the screen.
    fn output_line(&mut self, line: u32, pixels: &[u16; 240]);
}

pub trait AudioOutput {}

// Send should be safe to implement for the GBA because we never leak the RC's
// that are used by the GBA and its other parts.
unsafe impl Send for Gba {}
