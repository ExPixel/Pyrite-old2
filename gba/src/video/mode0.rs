use super::{line::LineBuffer, text};
use crate::memory::{io::IoRegisters, VRAM_SIZE};

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    vram: &[u8; VRAM_SIZE as usize],
) {
    for bg in 0..4 {
        if !ioregs.dispcnt.display_bg(bg) {
            continue;
        }

        let bgcnt = ioregs.bgcnt[bg as usize];
        let bgofs = ioregs.bgofs[bg as usize];

        if bgcnt.palette_256() {
            text::render_8bpp(buf, line, bg as usize, bgcnt, bgofs, vram);
        } else {
            text::render_4bpp(buf, line, bg as usize, bgcnt, bgofs, vram);
        }
    }
}
