use crate::memory::{io::IoRegisters, palette::Palette};

pub const OBJ: usize = 4;

pub struct LineBuffer {
    // 240 pixels for each background (BG0-3 + OBJ)
    pixels: [[u16; 240]; 5],
    layer_metadata: [LayerMetadata; 5],
}

impl Default for LineBuffer {
    fn default() -> Self {
        LineBuffer {
            pixels: [[0; 240]; 5],
            layer_metadata: [LayerMetadata::default(); 5],
        }
    }
}

impl LineBuffer {
    pub(crate) fn put(&mut self, layer: usize, x: usize, pixel: u16) {
        self.pixels[layer][x] = pixel | 0x8000;
    }
    pub(crate) fn put_4bpp(&mut self, layer: usize, x: usize, palette: u8, entry: u8) {
        self.pixels[layer][x] = ((palette as u16) << 4) | (entry as u16);
    }

    pub(crate) fn put_8bpp(&mut self, layer: usize, x: usize, entry: u8) {
        self.pixels[layer][x] = entry as u16;
    }

    pub(crate) fn layer_metadata_mut(&mut self, layer: usize) -> &mut LayerMetadata {
        &mut self.layer_metadata[layer]
    }

    fn color(&self, layer: usize, x: usize, palette: &Palette) -> Option<u16> {
        let metadata = self.layer_metadata[layer];
        let entry = self.pixels[layer][x];

        if metadata.is_bitmap() {
            return Some(entry);
        }

        if metadata.is_4bpp() {
            let color_entry = entry & 0xF;
            if color_entry == 0 {
                return None;
            }
            let palette_index = entry >> 4;

            if layer == OBJ {
                Some(palette.get_obj16(palette_index as _, color_entry as _))
            } else {
                Some(palette.get_bg16(palette_index as _, color_entry as _))
            }
        } else if entry == 0 {
            None
        } else if layer == OBJ {
            Some(palette.get_obj256(entry as _))
        } else {
            Some(palette.get_bg256(entry as _))
        }
    }

    pub fn render(&self, output: &mut [u16], ioregs: &IoRegisters, palette: &Palette) {
        let backdrop = palette.get_bg256(0) | 0x8000;
        (0..240)
            .map(|x| (x, self.render_pixel(x, ioregs, backdrop, palette)))
            .for_each(|(x, p)| output[x] = p);
    }

    fn render_pixel(
        &self,
        x: usize,
        ioregs: &IoRegisters,
        backdrop: u16,
        palette: &Palette,
    ) -> u16 {
        let mut displayed_layers: usize = 0xFF;

        for priority in (0..4).rev() {
            for bg in 0usize..4 {
                if ioregs.dispcnt.display_bg(bg as _) && ioregs.bgcnt[bg].priority() == priority {
                    displayed_layers <<= 4;
                    displayed_layers |= bg as usize;
                }
            }
            // FIXME add OBJ pixel here :)
        }

        let layer = displayed_layers & 0xF;
        if layer > 4 {
            return backdrop;
        }

        self.color(layer, x, palette).unwrap_or(backdrop)
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LayerMetadata {
    value: u16,
}

impl LayerMetadata {
    const BITMAP_16BPP: u16 = 0x1;
    const PALETTE_4BPP: u16 = 0x2;

    pub fn is_bitmap(&self) -> bool {
        (self.value & Self::BITMAP_16BPP) != 0
    }

    pub fn is_4bpp(&self) -> bool {
        (self.value & Self::PALETTE_4BPP) != 0
    }

    pub fn set_bitmap(&mut self) {
        self.value |= Self::BITMAP_16BPP;
    }

    pub fn set_4bpp(&mut self) {
        self.value |= Self::PALETTE_4BPP;
    }

    pub fn set_8bpp(&mut self) {
        /* NOP */
    }
}

struct PixelMetadata {}
