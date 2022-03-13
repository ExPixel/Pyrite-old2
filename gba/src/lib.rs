mod interrupts;
pub mod memory;
mod scheduler;
mod video;

pub use memory::GbaMemory;

use arm::Cpu;
use scheduler::Scheduler;
use util::bits::Bits;
pub use video::{GbaVideo, SCREEN_HEIGHT, SCREEN_PIXEL_COUNT, SCREEN_WIDTH};

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
        self.mem.use_custom_bios();
        self.cpu.branch(0x0, &mut self.mem);
    }

    pub fn set_gamepak(&mut self, cart: Vec<u8>) {
        self.mem.set_gamepak(cart);
    }

    pub fn frame(&mut self) {
        // wait until we are out of VBLANK
        while self.mem.ioregs.dispstat.vblank() {
            self.step();
        }

        // Wait until the end of the frame (enter VBLANK)
        while !self.mem.ioregs.dispstat.vblank() {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let mut cycles = self.cpu.step(&mut self.mem);

        while let Some((event, next_cycles)) = self.scheduler.advance(cycles) {
            cycles = next_cycles;
            (event)(self, cycles);
        }
    }

    pub fn set_pressed(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.mem.ioregs.keyinput &= !(1 << button as u16);
        } else {
            self.mem.ioregs.keyinput |= 1 << button as u16;
        }
    }

    pub fn set_buttons(&mut self, buttons: ButtonSet) {
        self.mem.ioregs.keyinput &= !0x3ff;
        self.mem.ioregs.keyinput |= u16::from(buttons) & 0x3ff;
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        !self.mem.ioregs.keyinput.is_bit_set(button as u16 as u32)
    }

    pub fn video(&self) -> &GbaVideo {
        &self.video
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Right = 4,
    Left = 5,
    Up = 6,
    Down = 7,
    R = 8,
    L = 9,
}

impl From<usize> for Button {
    fn from(value: usize) -> Button {
        match value {
            0 => Button::A,
            1 => Button::B,
            2 => Button::Select,
            3 => Button::Start,
            4 => Button::Right,
            5 => Button::Left,
            6 => Button::Up,
            7 => Button::Down,
            8 => Button::R,
            9 => Button::L,
            bad => panic!("{} is not a valid button value", bad),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ButtonSet(u16);

impl Default for ButtonSet {
    fn default() -> Self {
        ButtonSet(0x3ff)
    }
}

impl ButtonSet {
    pub fn set_pressed(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.0 &= !(1 << button as u16);
        } else {
            self.0 |= 1 << button as u16;
        }
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        !self.0.is_bit_set(button as u16 as u32)
    }
}

impl From<u16> for ButtonSet {
    fn from(value: u16) -> Self {
        ButtonSet(value & 0x3ff)
    }
}

impl From<ButtonSet> for u16 {
    fn from(set: ButtonSet) -> u16 {
        set.0
    }
}

// Send should be safe to implement for the GBA because we never leak the RC's
// that are used by the GBA and its other parts.
unsafe impl Send for Gba {}
