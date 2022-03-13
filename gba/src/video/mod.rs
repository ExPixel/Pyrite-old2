mod line;
mod mode0;
mod mode1;
mod mode2;
mod mode3;
mod mode4;
mod mode5;
mod obj;
mod text;

use arm::Cycles;

use crate::{dma::dma_on_timing, scheduler::Scheduler, GbaMemory};

use self::line::LineBuffer;

pub const SCREEN_WIDTH: usize = 240;
pub const SCREEN_HEIGHT: usize = 160;
pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const HDRAW_CYCLES: Cycles = Cycles::new(960);
const HBLANK_CYCLES: Cycles = Cycles::new(272);

pub struct GbaVideo {
    scheduler: Scheduler,
    pub(crate) screen: [u16; SCREEN_PIXEL_COUNT],
}

impl GbaVideo {
    pub fn new(scheduler: Scheduler) -> Self {
        let screen = [0u16; SCREEN_PIXEL_COUNT];
        GbaVideo { scheduler, screen }
    }

    pub(crate) fn init(&mut self, mem: &mut GbaMemory) {
        self.enter_hdraw(mem, Cycles::ZERO);
    }

    fn enter_hdraw(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.dispstat.set_hblank(false);
        self.scheduler.schedule(
            |gba, late| {
                if gba.mem.ioregs.vcount < 160 {
                    dma_on_timing(gba, crate::memory::io::Timing::HBlank);
                }
                gba.video.enter_hblank(&mut gba.mem, late);
            },
            HDRAW_CYCLES - late,
        );
    }

    fn exit_hblank(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.vcount = match mem.ioregs.vcount {
            159 => {
                mem.ioregs.dispstat.set_vblank(true);
                160
            }

            // VBLANK flag is NOT set during line 227
            226 => {
                mem.ioregs.dispstat.set_vblank(false);
                227
            }

            227 => 0,
            other => other + 1,
        };

        mem.ioregs
            .dispstat
            .set_vcounter_match(mem.ioregs.vcount == mem.ioregs.dispstat.vcount_setting());

        self.enter_hdraw(mem, late);
    }

    fn enter_hblank(&mut self, mem: &mut GbaMemory, late: Cycles) {
        mem.ioregs.dispstat.set_hblank(true);

        let line = mem.ioregs.vcount;

        if line < 160 {
            let output_buf_start = line as usize * 240;
            let output_buf_end = output_buf_start + 240;
            let output_buf = &mut self.screen[output_buf_start..output_buf_end];
            Self::render_line(line, output_buf, mem);
        }

        self.scheduler.schedule(
            |gba, late| {
                gba.video.exit_hblank(&mut gba.mem, late);
                if gba.mem.ioregs.vcount == 160 {
                    dma_on_timing(gba, crate::memory::io::Timing::VBlank)
                }
            },
            HBLANK_CYCLES - late,
        );
    }

    fn render_line(line: u16, output: &mut [u16], mem: &GbaMemory) {
        let mut buf = LineBuffer::default();

        match mem.ioregs.dispcnt.bg_mode() {
            0 => mode0::render(line, &mut buf, &mem.ioregs, &mem.vram),
            1 => mode1::render(line, &mut buf, &mem.ioregs, &mem.vram),
            2 => mode2::render(line, &mut buf, &mem.ioregs, &mem.vram),
            3 => mode3::render(line, &mut buf, &mem.ioregs, &mem.vram),
            4 => mode4::render(line, &mut buf, &mem.ioregs, &mem.vram),
            5 => mode5::render(line, &mut buf, &mem.ioregs, &mem.vram),
            _ => {}
        }

        if mem.ioregs.dispcnt.display_obj() {
            obj::render(line, &mut buf, &mem.ioregs, &mem.oam, &mem.vram);
        }

        buf.render(output, &mem.ioregs, &mem.palette);
    }

    pub fn screen(&self) -> &[u16; SCREEN_PIXEL_COUNT] {
        &self.screen
    }
}
