use arm::Cycles;

use crate::{scheduler::Scheduler, Gba, GbaMemory};

pub struct GbaVideo {
    scheduler: Scheduler,
}

impl GbaVideo {
    pub fn new(scheduler: Scheduler) -> Self {
        GbaVideo { scheduler }
    }

    pub fn init(&mut self, mem: &mut GbaMemory) {}

    fn on_hdraw(&mut self, late: Cycles) {}

    fn on_hblank(&mut self, late: Cycles) {}

    fn hdraw_callback(gba: &mut Gba, late: Cycles) {
        gba.video.on_hdraw(late)
    }

    fn hblank_callback(gba: &mut Gba, late: Cycles) {
        gba.video.on_hblank(late)
    }
}
