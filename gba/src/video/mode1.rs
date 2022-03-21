use crate::memory::{io::IoRegisters, VRAM_SIZE};

use super::{line::LineBuffer, text};

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    vram: &[u8; VRAM_SIZE as usize],
) {
    for bg in 0..3 {
        if !ioregs.dispcnt.display_bg(bg) {
            continue;
        }

        if bg < 2 {
            let bgcnt = ioregs.bgcnt[bg as usize];
            if bgcnt.palette_256() {
                text::render_8bpp(buf, line, bg as usize, ioregs, vram);
            } else {
                text::render_4bpp(buf, line, bg as usize, ioregs, vram);
            }
        } else {
            text::render_affine(buf, bg as usize, ioregs, vram);
        }
    }
}
