use crate::memory::{io::IoRegisters, VRAM_SIZE};

use super::{line::LineBuffer, text};

pub fn render(
    _line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    vram: &[u8; VRAM_SIZE as usize],
) {
    for bg in 2..4 {
        if !ioregs.dispcnt.display_bg(bg) {
            continue;
        }
        text::render_affine(buf, bg as usize, ioregs, vram);
    }
}
