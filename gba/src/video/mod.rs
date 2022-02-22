use arm::Cycles;

use crate::{scheduler::Scheduler, Gba, GbaMemory};

const SCREEN_WIDTH: usize = 240;
const SCREEN_HEIGHT: usize = 160;
const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct GbaVideo {
    scheduler: Scheduler,
    buffer: [u16; SCREEN_PIXEL_COUNT],
}

impl GbaVideo {
    pub fn new(scheduler: Scheduler) -> Self {
        let buffer = [0u16; SCREEN_PIXEL_COUNT];
        GbaVideo { scheduler, buffer }
    }

    pub(crate) fn init(&mut self, _mem: &mut GbaMemory) {}

    fn on_hdraw(&mut self, _late: Cycles) {}

    fn on_hblank(&mut self, _late: Cycles) {}

    fn hdraw_callback(gba: &mut Gba, late: Cycles) {
        gba.video.on_hdraw(late)
    }

    fn hblank_callback(gba: &mut Gba, late: Cycles) {
        gba.video.on_hblank(late)
    }

    pub fn buffer(&self) -> &[u16; SCREEN_PIXEL_COUNT] {
        &self.buffer
    }
}
