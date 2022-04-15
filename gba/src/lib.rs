mod audio;
mod dma;
mod interrupts;
pub mod memory;
mod scheduler;
mod timers;
mod video;

use dma::GbaDMA;
pub use memory::GbaMemory;

use arm::{Cpu, Cycles, Memory};
pub use audio::{sampler::GbaAudioSampler, Command, GbaAudio};
use scheduler::Scheduler;
use util::bits::Bits;
pub use video::{GbaVideo, SCREEN_HEIGHT, SCREEN_PIXEL_COUNT, SCREEN_WIDTH};

pub struct Gba {
    mem: GbaMemory,
    cpu: Cpu,
    dma: [GbaDMA; 4],
    in_dma: bool,
    video: GbaVideo,
    audio: GbaAudio,
    scheduler: Scheduler,
    step_fn: fn(&mut Self) -> arm::Cycles,
    state: State,
}

impl Gba {
    pub const CYCLES_PER_SECOND: u32 = 16 * 1024 * 1024;
    pub fn new() -> Gba {
        let scheduler = Scheduler::default();

        Gba {
            mem: GbaMemory::new(scheduler.clone()),
            cpu: Cpu::uninitialized(arm::Isa::Arm, arm::CpuMode::System),
            dma: [
                GbaDMA::default(),
                GbaDMA::default(),
                GbaDMA::default(),
                GbaDMA::default(),
            ],
            in_dma: false,
            video: GbaVideo::new(scheduler.clone()),
            audio: GbaAudio::new(scheduler.clone()),
            scheduler,
            state: State::Running,
            step_fn: Self::step_cpu,
        }
    }

    pub fn reset(&mut self, boot_from_bios: bool) {
        self.scheduler.clear();
        self.mem.init();
        self.video.init(&mut self.mem);

        if boot_from_bios {
            self.cpu.branch(0x0, &mut self.mem);
        } else {
            self.emulate_boot()
        }
    }

    fn emulate_boot(&mut self) {
        self.cpu.registers.write_mode(arm::CpuMode::Supervisor);
        self.cpu.registers.write(13, 0x3007FE0); // sp_svc = 0x3007FE0
        self.cpu.registers.write(14, 0); // lr_svc = 0
        self.cpu.registers.write_spsr(0); // spsr_svc = 0

        self.cpu.registers.write_mode(arm::CpuMode::IRQ);
        self.cpu.registers.write(13, 0x3007FA0); // sp_irq = 0x3007FA0
        self.cpu.registers.write(14, 0); // lr_irq = 0
        self.cpu.registers.write_spsr(0); // spsr_irq = 0

        self.cpu.registers.write_mode(arm::CpuMode::System);
        self.cpu.registers.write(13, 0x3007F00); // sp_sys = 0x3007F00

        // r0-r12 = 0
        (0..=12).for_each(|r| self.cpu.registers.write(r, 0));

        // zero fill 512 byte region [3007E00h, 3007FFFh]
        (0u32..0x200).for_each(|idx| {
            self.mem.store8(0x3007E00 + idx, 0, arm::AccessType::Seq);
        });

        self.cpu.registers.clearf_t();
        self.cpu.branch(0x08000000, &mut self.mem);
    }

    pub fn set_bios(&mut self, bios: Option<Vec<u8>>) {
        if let Some(bios) = bios {
            self.mem.set_bios(bios);
        } else {
            self.mem.use_custom_bios();
        }
    }

    pub fn set_gamepak(&mut self, cart: Vec<u8>) {
        self.mem.set_gamepak(cart);
    }

    pub fn frame(&mut self) {
        self.audio.clear(self.mem.ioregs.time);

        // wait until we are out of VBLANK
        while self.mem.ioregs.dispstat.vblank() {
            self.step();
        }

        // Wait until the end of the frame (enter VBLANK)
        while !self.mem.ioregs.dispstat.vblank() {
            self.step();
        }
    }

    fn step_cpu(&mut self) -> arm::Cycles {
        self.cpu.step(&mut self.mem)
    }

    fn step_idle(&mut self) -> arm::Cycles {
        let remain_idle = self.mem.ioregs.ie_reg.value & self.mem.ioregs.if_reg.value == 0;

        if !remain_idle {
            self.state = State::Running;
            self.restore_step();
            Cycles::ZERO
        } else {
            self.scheduler
                .cycles_until_next_event(self.mem.ioregs.time)
                .unwrap_or(Cycles::new(32))
        }
    }

    fn restore_step(&mut self) {
        self.step_fn = match self.state {
            State::Running => Self::step_cpu,
            State::Halted => Self::step_idle,
            State::Stopped => Self::step_idle,
        };
    }

    fn step(&mut self) {
        let now = self.mem.ioregs.time + u32::from((self.step_fn)(self)) as u64;
        while let Some((event, when)) = self.scheduler.next(now) {
            self.mem.ioregs.time = when;
            (event)(self);
        }
        self.mem.ioregs.time = now;
    }

    pub(crate) fn stop(&mut self) {
        self.state = State::Stopped;

        if !self.in_dma {
            self.restore_step();
        }
    }

    pub(crate) fn halt(&mut self) {
        self.state = State::Halted;

        if !self.in_dma {
            self.restore_step();
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

    pub fn memory(&self) -> &GbaMemory {
        &self.mem
    }

    pub fn memory_mut(&mut self) -> &mut GbaMemory {
        &mut self.mem
    }

    pub fn video(&self) -> &GbaVideo {
        &self.video
    }

    pub fn audio(&self) -> &GbaAudio {
        &self.audio
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Halted,
    Stopped,
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
