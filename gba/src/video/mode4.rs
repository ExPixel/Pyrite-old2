use crate::memory::{io::IoRegisters, palette::Palette, VRAM_SIZE};

use super::line::LineBuffer;

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    vram: &[u8; VRAM_SIZE as usize],
) {
    buf.layer_metadata_mut(2).set_8bpp();

    let mut frame_line_start = line as usize * 240;
    if ioregs.dispcnt.frame() == 1 {
        frame_line_start += 0xA000;
    }
    let frame_line = &vram[frame_line_start..(frame_line_start + 240)];

    for (x, &entry) in frame_line.iter().take(240).enumerate() {
        buf.put_8bpp(2, x, entry);
    }
}
