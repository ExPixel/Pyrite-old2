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
    objwin: LineBits,
    layer_attrs: [LayerAttrs; 5],
}

impl Default for LineBuffer {
    fn default() -> Self {
        LineBuffer {
            pixels: [[0; 240]; 5],
            objwin: LineBits::zeroes(),
            layer_attrs: [LayerAttrs::default(); 5],
        }
    }
}

impl LineBuffer {
    pub(crate) fn put(&mut self, layer: usize, x: usize, pixel: u16) {
        self.pixels[layer][x] = pixel | 0x8000;
    }

    /// Applies a horizontal mosaic to a BG layer.
    pub(crate) fn mosaic(&mut self, layer: usize, mosaic: u32) {
        if mosaic == 0 {
            return;
        }

        self.pixels[layer]
            .chunks_mut(mosaic as usize)
            .for_each(|chunk| chunk.fill(chunk[0]));
    }

    pub(crate) fn put_obj_4bpp(&mut self, attrs: PixelAttrs, x: usize, palette: u8, entry: u8) {
        if entry == 0 {
            return;
        }
        self.pixels[OBJ][x] =
            (attrs.value as u16) | ((palette as u16) << 12) | ((entry as u16) << 8);
    }

    pub(crate) fn put_obj_8bpp(&mut self, attrs: PixelAttrs, x: usize, entry: u8) {
        if entry == 0 {
            return;
        }
        self.pixels[OBJ][x] = attrs.value as u16 | ((entry as u16) << 8);
    }

    pub(crate) fn put_obj_window(&mut self, x: usize) {
        self.objwin.put(x, true);
    }

    pub(crate) fn put_4bpp(&mut self, layer: usize, x: usize, palette: u8, entry: u8) {
        self.pixels[layer][x] = ((palette as u16) << 4) | (entry as u16);
    }

    pub(crate) fn put_8bpp(&mut self, layer: usize, x: usize, entry: u8) {
        self.pixels[layer][x] = entry as u16;
    }

    /// Same as put_8bpp but places 8 pixels at a time stored in a [`u64`].
    pub(crate) fn put_8bpp_8(&mut self, layer: usize, x: usize, entries: u64) {
        self.pixels[layer][x] = entries as u8 as u16;
        self.pixels[layer][x + 1] = (entries >> 8) as u8 as u16;
        self.pixels[layer][x + 2] = (entries >> 16) as u8 as u16;
        self.pixels[layer][x + 3] = (entries >> 24) as u8 as u16;
        self.pixels[layer][x + 4] = (entries >> 32) as u8 as u16;
        self.pixels[layer][x + 5] = (entries >> 40) as u8 as u16;
        self.pixels[layer][x + 6] = (entries >> 48) as u8 as u16;
        self.pixels[layer][x + 7] = (entries >> 56) as u8 as u16;
    }

    pub(crate) fn layer_attrs_mut(&mut self, layer: usize) -> &mut LayerAttrs {
        &mut self.layer_attrs[layer]
    }

    fn color_obj(&self, x: usize, palette: &Palette) -> Option<(u16, PixelAttrs)> {
        let entry = self.pixels[OBJ][x];
        let attrs = PixelAttrs { value: entry as u8 };

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

    fn in_win_bounds(v: u16, lo: u16, hi: u16) -> bool {
        if lo <= hi {
            v >= lo && v < hi
        } else {
            v >= lo || v < hi
        }
    }

    fn has_obj_window_pixel(&self, x: usize) -> bool {
        self.objwin.get(x)
    }

    fn generate_window_mask(&self, layer: usize, ioregs: &IoRegisters) -> WindowMask {
        if !ioregs.dispcnt.windows_enabled() {
            return WindowMask::new_all_enabled();
        }

        let win0_enabled = ioregs.dispcnt.win0_display();
        let win1_enabled = ioregs.dispcnt.win1_display();
        let winobj_enabled = ioregs.dispcnt.obj_window_display();

        let in_win0 = ioregs.wininout.win0_layer_enabled(layer);
        let in_win1 = ioregs.wininout.win1_layer_enabled(layer);
        let in_winout = ioregs.wininout.winout_layer_enabled(layer);
        let in_winobj = ioregs.wininout.winobj_layer_enabled(layer);
        if !(in_win0 | in_win1 | in_winout | in_winobj) {
            return WindowMask::new_all_disabled();
        }

        let win0_t = ioregs.winhv.win0_y1();
        let win0_b = ioregs.winhv.win0_y2();
        let win1_t = ioregs.winhv.win1_y1();
        let win1_b = ioregs.winhv.win1_y2();

        let line = ioregs.vcount;
        let in_win0_bounds_v = Self::in_win_bounds(line, win0_t, win0_b);
        let in_win1_bounds_v = Self::in_win_bounds(line, win1_t, win1_b);

        let win0_l = ioregs.winhv.win0_x1();
        let win0_r = ioregs.winhv.win0_x2();
        let win1_l = ioregs.winhv.win1_x1();
        let win1_r = ioregs.winhv.win1_x2();

        let win0_effects = ioregs.wininout.win0_effects_enabled();
        let win1_effects = ioregs.wininout.win1_effects_enabled();
        let winout_effects = ioregs.wininout.winout_effects_enabled();
        let winobj_effects = ioregs.wininout.winobj_effects_enabled();

        let mut mask = WindowMask::new_all_disabled();

        for x in 0..240 {
            let in_win0_bounds = in_win0_bounds_v && Self::in_win_bounds(x, win0_l, win0_r);
            if win0_enabled && in_win0_bounds {
                mask.set_visible(x as _, in_win0, win0_effects);
                continue;
            }

            let in_win1_bounds = in_win1_bounds_v && Self::in_win_bounds(x, win1_l, win1_r);
            if win1_enabled && in_win1_bounds {
                mask.set_visible(x as _, in_win1, win1_effects);
                continue;
            }

            if winobj_enabled && self.has_obj_window_pixel(x as usize) {
                mask.set_visible(x as _, in_winobj, winobj_effects);
                continue;
            }

            if in_winout {
                mask.set_visible(x as _, true, winout_effects);
            }
        }

        mask
    }

    fn backdrop_pixel(ioregs: &IoRegisters, palette: &Palette) -> Pixel {
        let backdrop_color = palette.get_bg256(0) | 0x8000;
        let mut backdrop_attrs = PixelAttrs::default();
        backdrop_attrs.set_priority(3);
        if ioregs.bldcnt.is_first_target(BACKDROP) {
            backdrop_attrs.set_first_target();
        }
        if ioregs.bldcnt.is_second_target(BACKDROP) {
            backdrop_attrs.set_second_target();
        }

        let mut empty_attrs = PixelAttrs::default();
        empty_attrs.set_priority(3);

        let mut backdrop = Pixel::default();
        backdrop.push(backdrop_color, empty_attrs);
        backdrop.push(backdrop_color, backdrop_attrs);

        backdrop
    }

    pub fn render(&self, output: &mut [u16], ioregs: &IoRegisters, palette: &Palette) {
        let mut pixels = [Self::backdrop_pixel(ioregs, palette); 240];

        for priority in (0..4).rev() {
            for bg in (0usize..4).rev() {
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
                attrs.set_priority(priority);

                let mask = self.generate_window_mask(bg, ioregs);

                (0..240).for_each(|x| {
                    if !mask.visible(x) {
                        return;
                    }

                    if let Some(color) = self.color_bg(bg, x, palette) {
                        pixels[x].push(color, attrs.effects_mask(mask.effects(x)));
                    }
                });
            }
        }

        let obj_mask = self.generate_window_mask(OBJ, ioregs);
        if ioregs.dispcnt.display_obj() {
            (0..240).for_each(|x| {
                if !obj_mask.visible(x) {
                    return;
                }

                if let Some((color, attrs)) = self.color_obj(x, palette) {
                    pixels[x].place_obj(color, attrs.effects_mask(obj_mask.effects(x)));
                }
            });
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
        self.colors.1 = std::mem::replace(&mut self.colors.0, color);
        self.attrs.1 = std::mem::replace(&mut self.attrs.0, attrs);
    }

    pub fn place_obj(&mut self, color: u16, attrs: PixelAttrs) {
        if attrs.priority() <= self.attrs.0.priority() {
            self.push(color, attrs);
            return;
        }

        if attrs.priority() <= self.attrs.1.priority() {
            self.colors.1 = color;
            self.attrs.1 = attrs;
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
    let r = r - ((r * evy) / 16);
    let g = g - ((g * evy) / 16);
    let b = b - ((b * evy) / 16);
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

    fn effects_mask(mut self, has_effects: bool) -> Self {
        if !has_effects {
            self.value &= !0xB; // mask out bits 0,1,3
        }
        self
    }

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

    /// Only used by OBJ layer pixels while calculating color.
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

#[derive(Default, Copy, Clone)]
struct LineBits {
    inner: [u8; 30],
}

impl LineBits {
    const fn ones() -> Self {
        LineBits { inner: [0xFF; 30] }
    }

    const fn zeroes() -> Self {
        LineBits { inner: [0x00; 30] }
    }

    fn put(&mut self, index: usize, value: bool) {
        if index < 240 {
            self.inner[index / 8] |= (value as u8) << (index % 8);
        }
    }

    fn get(&self, index: usize) -> bool {
        if index < 240 {
            (self.inner[index / 8] & (1 << (index % 8))) != 0
        } else {
            false
        }
    }
}

#[derive(Copy, Clone)]
struct WindowMask {
    visible: LineBits,
    effects: LineBits,
}

impl WindowMask {
    fn new_all_enabled() -> Self {
        WindowMask {
            visible: LineBits::ones(),
            effects: LineBits::ones(),
        }
    }

    fn new_all_disabled() -> Self {
        WindowMask {
            visible: LineBits::zeroes(),
            effects: LineBits::zeroes(),
        }
    }

    fn set_visible(&mut self, x: usize, visible: bool, effects: bool) {
        if x < 240 {
            self.visible.put(x, visible);
            self.effects.put(x, effects);
        }
    }

    fn visible(&self, x: usize) -> bool {
        self.visible.get(x)
    }

    fn effects(&self, x: usize) -> bool {
        self.effects.get(x)
    }
}
