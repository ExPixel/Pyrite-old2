use crate::memory::VRAM_SIZE;
use byteorder::{ByteOrder as _, LittleEndian as LE};

use super::line::LineBuffer;

/// Render a single line in mode3.
///
/// **BG Mode 3 - 240x160 pixels, 32768 colors**
/// Two bytes are associated to each pixel, directly defining one of the 32768 colors (without using palette data,
/// and thus not supporting a 'transparent' BG color).  
///   Bit   Expl.  
///   0-4   Red Intensity   (0-31)  
///   5-9   Green Intensity (0-31)  
///   10-14 Blue Intensity  (0-31)  
///   15    Not used in GBA Mode (in NDS Mode: Alpha=0=Transparent, Alpha=1=Normal)  
/// The first 480 bytes define the topmost line, the next 480 the next line, and so on.
/// The background occupies 75 KBytes (06000000-06012BFF), most of the 80 Kbytes BG area,
/// not allowing to redraw an invisible second frame in background, so this mode is mostly recommended for still images only.
pub fn render(line: u16, buf: &mut LineBuffer, vram: &[u8; VRAM_SIZE as usize]) {
    buf.layer_metadata_mut(2).set_bitmap();

    let vstart = 480 * line as usize;
    for x in 0..240 {
        buf.put(2, x, LE::read_u16(&vram[(vstart + x * 2)..]));
    }
}
