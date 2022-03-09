use util::bits::Bits as _;

use crate::memory::{
    io::{Effect, IoRegisters},
    palette::Palette,
};

pub const OBJ: usize = 4;
pub const BACKDROP: usize = 5;

pub struct LineBuffer {
    // 240 pixels for each background (BG0-3 + OBJ)
    pixels: [[u16; 240]; 5],
    layer_attrs: [LayerAttrs; 5],
}

impl Default for LineBuffer {
    fn default() -> Self {
        LineBuffer {
            pixels: [[0; 240]; 5],
            layer_attrs: [LayerAttrs::default(); 5],
        }
    }
}

impl LineBuffer {
    pub(crate) fn put(&mut self, layer: usize, x: usize, pixel: u16) {
        self.pixels[layer][x] = pixel | 0x8000;
    }

    pub(crate) fn put_obj_4bpp(&mut self, attrs: PixelAttrs, x: usize, palette: u8, entry: u8) {
        self.pixels[OBJ][x] =
            (attrs.value as u16) | ((palette as u16) << 12) | ((entry as u16) << 8);
    }

    pub(crate) fn put_obj_8bpp(&mut self, attrs: PixelAttrs, x: usize, entry: u8) {
        self.pixels[OBJ][x] = attrs.value as u16 | ((entry as u16) << 8);
    }

    pub(crate) fn put_4bpp(&mut self, layer: usize, x: usize, palette: u8, entry: u8) {
        self.pixels[layer][x] = ((palette as u16) << 4) | (entry as u16);
    }

    pub(crate) fn put_8bpp(&mut self, layer: usize, x: usize, entry: u8) {
        self.pixels[layer][x] = entry as u16;
    }

    pub(crate) fn layer_attrs_mut(&mut self, layer: usize) -> &mut LayerAttrs {
        &mut self.layer_attrs[layer]
    }

    fn color_obj(&self, x: usize, priority: u16, palette: &Palette) -> Option<(u16, PixelAttrs)> {
        let entry = self.pixels[OBJ][x];
        let attrs = PixelAttrs { value: entry as u8 };

        if attrs.priority() != priority {
            return None;
        }

        let entry = (entry >> 8) as u8;

        if attrs.is_4bpp() {
            let color_entry = entry & 0xF;
            if color_entry == 0 {
                return None;
            }
            let palette_index = entry >> 4;
            Some((
                palette.get_obj16(palette_index as _, color_entry as _),
                attrs,
            ))
        } else if entry as u8 == 0 {
            None
        } else {
            Some((palette.get_obj256(entry), attrs))
        }
    }

    fn color_bg(&self, layer: usize, x: usize, palette: &Palette) -> Option<u16> {
        let attrs = self.layer_attrs[layer];
        let entry = self.pixels[layer][x];

        if attrs.is_bitmap() {
            return Some(entry);
        }

        if attrs.is_4bpp() {
            let color_entry = entry & 0xF;
            if color_entry == 0 {
                return None;
            }
            let palette_index = (entry >> 4) & 0xF;

            Some(palette.get_bg16(palette_index as _, color_entry as _))
        } else if entry == 0 {
            None
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
        let mut attrs = (PixelAttrs::default(), PixelAttrs::default());
        let mut colors = (backdrop, 0);

        if ioregs.bldcnt.is_first_target(BACKDROP) {
            attrs.0.set_first_target();
        }
        if ioregs.bldcnt.is_second_target(BACKDROP) {
            attrs.0.set_second_target();
        }

        for priority in (0..4).rev() {
            for bg in (0usize..4).rev() {
                // FIXME do window checks here.
                if !ioregs.dispcnt.display_bg(bg as _) || ioregs.bgcnt[bg].priority() != priority {
                    continue;
                }

                if let Some(color) = self.color_bg(bg, x, palette) {
                    let mut new_attrs = PixelAttrs::default();
                    if ioregs.bldcnt.is_first_target(bg) {
                        new_attrs.set_first_target();
                    }
                    if ioregs.bldcnt.is_second_target(bg) {
                        new_attrs.set_second_target();
                    }

                    attrs.1 = attrs.0;
                    attrs.0 = new_attrs;

                    colors.1 = colors.0;
                    colors.0 = color;
                }
            }

            if ioregs.dispcnt.display_obj() {
                if let Some((color, new_attrs)) = self.color_obj(x, priority, palette) {
                    attrs.1 = attrs.0;
                    attrs.0 = new_attrs;

                    colors.1 = colors.0;
                    colors.0 = color;
                }
            }
        }

        let effect = ioregs.bldcnt.effect();
        if effect == Effect::None {
            return colors.0;
        }

        match effect {
            Effect::AlphaBlending => {
                // For this effect, the top-most non-transparent pixel must be selected as 1st Target,
                // and the next-lower non-transparent pixel must be selected as 2nd Target, if so - and only if so,
                // then color intensities of 1st and 2nd Target are mixed together by using the parameters in BLDALPHA register,
                // for each pixel each R, G, B intensities are calculated separately:
                //   I = MIN ( 31, I1st*EVA + I2nd*EVB )
                // Otherwise - for example, if only one target exists, or if a non-transparent non-2nd-target
                // pixel is moved between the two targets, or if 2nd target has higher display priority than 1st target -
                // then only the top-most pixel is displayed (at normal intensity, regardless of BLDALPHA).
                if attrs.0.is_first_target() && attrs.1.is_second_target() {
                    let eva = ioregs.bldalpha.eva_coeff();
                    let evb = ioregs.bldalpha.evb_coeff();
                    alpha_blend(colors.0, colors.1, eva, evb)
                } else {
                    colors.0
                }
            }

            Effect::None => unreachable!(),

            _ => colors.0,
        }
    }
}

fn alpha_blend(c1: u16, c2: u16, eva: u16, evb: u16) -> u16 {
    let (r1, g1, b1) = decompose(c1);
    let (r2, g2, b2) = decompose(c2);
    //   I = MIN ( 31, I1st*EVA + I2nd*EVB )
    let r = (r1 * eva + r2 * evb) / 16;
    let g = (g1 * eva + g2 * evb) / 16;
    let b = (b1 * eva + b2 * evb) / 16;
    recompose(r, g, b)
}

fn decompose(c: u16) -> (u16, u16, u16) {
    let r = c.bits(0, 4);
    let g = c.bits(5, 9);
    let b = c.bits(10, 14);
    (r, g, b)
}

fn recompose(r: u16, g: u16, b: u16) -> u16 {
    r.min(31) | (g.min(31) << 5) | (b.min(31) << 10)
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LayerAttrs {
    value: u8,
}

impl LayerAttrs {
    const BITMAP_16BPP: u8 = 0x1;
    const PALETTE_4BPP: u8 = 0x2;

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

#[derive(Clone, Copy, Default)]
pub(crate) struct PixelAttrs {
    value: u8,
}

impl PixelAttrs {
    const FIRST_TARGET: u8 = 0x1;
    const SECOND_TARGET: u8 = 0x2;
    const PALETTE_4BPP: u8 = 0x4;

    pub fn is_first_target(&self) -> bool {
        (self.value & Self::FIRST_TARGET) != 0
    }

    pub fn is_second_target(&self) -> bool {
        (self.value & Self::SECOND_TARGET) != 0
    }

    pub fn set_first_target(&mut self) {
        self.value |= Self::FIRST_TARGET;
    }

    pub fn set_second_target(&mut self) {
        self.value |= Self::SECOND_TARGET;
    }

    /// Only used by OBJ layer pixels
    pub fn is_4bpp(&self) -> bool {
        (self.value & Self::PALETTE_4BPP) != 0
    }

    /// Only used by OBJ layer pixels
    pub fn set_4bpp(&mut self) {
        self.value |= Self::PALETTE_4BPP;
    }

    /// Only used by OBJ layer pixels
    pub fn set_8bpp(&mut self) {
        /* NOP */
    }

    pub fn set_priority(&mut self, priority: u16) {
        self.value |= (priority as u8) << 6;
    }

    pub fn priority(&self) -> u16 {
        (self.value >> 6) as u16
    }
}
