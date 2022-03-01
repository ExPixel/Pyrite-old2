mod mode3;

use arm::Cycles;

use crate::{scheduler::Scheduler, GbaMemory};

pub const SCREEN_WIDTH: usize = 240;
pub const SCREEN_HEIGHT: usize = 160;
pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const HDRAW_CYCLES: Cycles = Cycles::new(960);
const HBLANK_CYCLES: Cycles = Cycles::new(272);

pub struct GbaVideo {
    scheduler: Scheduler,
    pub(crate) buffer: [u16; SCREEN_PIXEL_COUNT],
}

impl GbaVideo {
    pub fn new(scheduler: Scheduler) -> Self {
        let buffer = [0u16; SCREEN_PIXEL_COUNT];
        GbaVideo { scheduler, buffer }
    }

    pub(crate) fn init(&mut self, mem: &mut GbaMemory) {
        self.enter_hdraw(mem, Cycles::ZERO);
    }

    fn enter_hdraw(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.set_hblank(false);
        self.scheduler.schedule(
            |gba, late| gba.video.enter_hblank(&mut gba.mem, late),
            HDRAW_CYCLES - late,
        );
    }

    fn exit_hblank(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.vcount = match mem.ioregs.vcount {
            159 => {
                mem.ioregs.set_vblank(true);
                160
            }

            // VBLANK flag is NOT set during line 227
            226 => {
                mem.ioregs.set_vblank(false);
                227
            }

            227 => 0,
            other => other + 1,
        };

        mem.ioregs
            .set_vcount_match(mem.ioregs.vcount == mem.ioregs.vcount_setting());

        self.enter_hdraw(mem, late);
    }

    fn enter_hblank(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.set_hblank(true);

        let line = mem.ioregs.vcount;

        if line < 160 {
            let buf_start = line as usize * 240;
            let buf_end = buf_start + 240;
            let buf = &mut self.buffer[buf_start..buf_end];
            mode3::render(line, buf.try_into().unwrap(), &mem.vram);
        }

        self.scheduler.schedule(
            |gba, late| gba.video.exit_hblank(&mut gba.mem, late),
            HBLANK_CYCLES - late,
        );
    }

    pub fn buffer(&self) -> &[u16; SCREEN_PIXEL_COUNT] {
        &self.buffer
    }
}
