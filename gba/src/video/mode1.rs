use crate::memory::{io::IoRegisters, VRAM_SIZE};

use super::line::LineBuffer;

pub fn render(
    _line: u16,
    _buf: &mut LineBuffer,
    _ioregs: &IoRegisters,
    _vram: &[u8; VRAM_SIZE as usize],
) {
    log::debug!("mode1 not yet implemented")
}
