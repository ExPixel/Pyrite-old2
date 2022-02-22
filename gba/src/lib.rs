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

    pub fn frame(&mut self) {}

    pub fn step(&mut self) {
        let mut cycles = self.cpu.step(&mut self.mem);

        while let Some((event, next_cycles)) = self.scheduler.advance(cycles) {
            cycles = next_cycles;
            (event)(self, cycles);
        }
    }

    pub fn video(&self) -> &GbaVideo {
        &self.video
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

// Send should be safe to implement for the GBA because we never leak the RC's
// that are used by the GBA and its other parts.
unsafe impl Send for Gba {}
