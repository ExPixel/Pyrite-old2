use util::bits::Bits as _;

use crate::memory::{
    io::{AlphaBlendingCoeff, BrightnessCoeff, Effect, IoRegisters},
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
        let backdrop_color = palette.get_bg256(0) | 0x8000;
        let mut backdrop_attrs = PixelAttrs::default();
        if ioregs.bldcnt.is_first_target(BACKDROP) {
            backdrop_attrs.set_first_target();
        }
        if ioregs.bldcnt.is_second_target(BACKDROP) {
            backdrop_attrs.set_second_target();
        }
        let mut backdrop = Pixel::default();
        backdrop.push(backdrop_color, backdrop_attrs);
        let mut pixels = [backdrop; 240];

        for priority in (0..4).rev() {
            for bg in (0usize..4).rev() {
                // FIXME do window checks here.
                if !ioregs.dispcnt.display_bg(bg as _) || ioregs.bgcnt[bg].priority() != priority {
                    continue;
                }

                let mut attrs = PixelAttrs::default();
                if ioregs.bldcnt.is_first_target(bg) {
                    attrs.set_first_target();
                }
                if ioregs.bldcnt.is_second_target(bg) {
                    attrs.set_second_target();
                }

                (0..240).for_each(|x| {
                    if let Some(color) = self.color_bg(bg, x, palette) {
                        pixels[x].push(color, attrs);
                    }
                });
            }

            if ioregs.dispcnt.display_obj() {
                (0..240).for_each(|x| {
                    if let Some((color, attrs)) = self.color_obj(x, priority, palette) {
                        pixels[x].push(color, attrs);
                    }
                });
            }
        }

        let effect = ioregs.bldcnt.effect();
        let bldalpha = ioregs.bldalpha;
        let bldy = ioregs.bldy;

        (0..240)
            .map(|x| (x, self.render_pixel(pixels[x], effect, bldalpha, bldy)))
            .for_each(|(x, p)| output[x] = p);
    }

    fn render_pixel(
        &self,
        pixel: Pixel,
        mut effect: Effect,
        bldalpha: AlphaBlendingCoeff,
        bldy: BrightnessCoeff,
    ) -> u16 {
        let Pixel { mut attrs, colors } = pixel;

        // OBJs that are defined as 'Semi-Transparent' in OAM memory are always selected as 1st Target (regardless of BLDCNT Bit 4),
        // and are always using Alpha Blending mode (regardless of BLDCNT Bit 6-7).
        // The BLDCNT register may be used to perform Brightness effects on the OBJ (and/or other BG/BD layers).
        // However, if a semi-transparent OBJ pixel does overlap a 2nd target pixel, then semi-transparency becomes priority,
        // and the brightness effect will not take place (neither on 1st, nor 2nd target).
        if attrs.0.is_semi_transparent() {
            attrs.0.set_first_target();
            if attrs.1.is_second_target() {
                effect = Effect::AlphaBlending;
            }
        }

        if effect == Effect::None || !attrs.0.is_first_target() {
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
                if attrs.1.is_second_target() {
                    let eva = bldalpha.eva_coeff();
                    let evb = bldalpha.evb_coeff();
                    alpha_blend(colors.0, colors.1, eva, evb)
                } else {
                    colors.0
                }
            }

            Effect::BrightnessIncrease => {
                //  For each pixel each R, G, B intensities are calculated separately:
                //   I = I1st + (31-I1st)*EVY   ;For Brightness Increase
                // The color intensities of any selected 1st target surface(s) are increased by using the parameter in BLDY register.
                let evy = bldy.evy_coeff();
                brightness_increase(colors.0, evy)
            }

            Effect::BrightnessDecrease => {
                //  For each pixel each R, G, B intensities are calculated separately:
                //   I = I1st - (I1st)*EVY      ;For Brightness Decrease
                // The color intensities of any selected 1st target surface(s) are decreased by using the parameter in BLDY register.
                let evy = bldy.evy_coeff();
                brightness_decrease(colors.0, evy)
            }

            Effect::None => unreachable!(),
        }
    }
}

#[derive(Default, Copy, Clone)]
struct Pixel {
    attrs: (PixelAttrs, PixelAttrs),
    colors: (u16, u16),
}

impl Pixel {
    pub fn push(&mut self, color: u16, attrs: PixelAttrs) {
        self.colors.1 = self.colors.0;
        self.colors.0 = color;

        self.attrs.1 = self.attrs.0;
        self.attrs.0 = attrs;
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

fn brightness_increase(c: u16, evy: u16) -> u16 {
    let (r, g, b) = decompose(c);
    //   I = I1st + (31-I1st)*EVY
    let r = r + (((31 - r) * evy) / 16);
    let g = g + (((31 - g) * evy) / 16);
    let b = b + (((31 - b) * evy) / 16);
    recompose(r, g, b)
}

fn brightness_decrease(c: u16, evy: u16) -> u16 {
    let (r, g, b) = decompose(c);
    //   I = I1st - (I1st)*EVY
    let r = r + ((r * evy) / 16);
    let g = g + ((g * evy) / 16);
    let b = b + ((b * evy) / 16);
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
    // # NOTE Bits 6 and 7 are used for priority

    const FIRST_TARGET: u8 = 0x1; // bit 0
    const SECOND_TARGET: u8 = 0x2; // bit 1
    const PALETTE_4BPP: u8 = 0x4; // bit 2
    const SEMI_TRANSPARENT: u8 = 0x8; // bit 3

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

    pub fn set_semi_transparent(&mut self) {
        self.value |= Self::SEMI_TRANSPARENT;
    }

    pub fn is_semi_transparent(&self) -> bool {
        (self.value & Self::SEMI_TRANSPARENT) != 0
    }

    pub fn set_priority(&mut self, priority: u16) {
        self.value |= (priority as u8) << 6;
    }

    pub fn priority(&self) -> u16 {
        (self.value >> 6) as u16
    }
}
