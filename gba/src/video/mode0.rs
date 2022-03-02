use crate::memory::{io::IoRegisters, palette::Palette, VRAM_SIZE};

use super::line::LineBuffer;

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    palette: &Palette,
    vram: &[u8; VRAM_SIZE as usize],
) {
    for (priority, bg) in (0..16).map(|idx| (idx & 0x3, idx >> 2)) {
        // if ioregs.screen_display_bg(bg) && ioregs.bgcnt[bg as usize] {}
    }
}
