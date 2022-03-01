use crate::memory::{io::IoRegisters, palette::Palette, VRAM_SIZE};

use super::line::LineBuffer;

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    palette: &Palette,
    vram: &[u8; VRAM_SIZE as usize],
) {
    let mut frame_line_start = line as usize * 240;
    if ioregs.display_frame() == 1 {
        frame_line_start += 0xA000;
    }
    let frame_line = &vram[frame_line_start..(frame_line_start + 240)];

    for (x, &entry) in frame_line.iter().take(240).enumerate() {
        let mut color = palette.get_bg256(entry);
        if entry == 0 {
            color &= !0x8000;
        }
        buf.put(2, x, color);
    }
}
