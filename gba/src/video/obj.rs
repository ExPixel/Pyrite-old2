use util::{
    bitfields,
    bits::Bits,
    fixedpoint::{FixedPoint16, FixedPoint32},
    mem::read_u16,
    primitive_enum,
};

use super::line::LineBuffer;
use crate::{
    memory::{
        io::{IoRegisters, ObjCharVramMapping},
        OAM_SIZE, VRAM_SIZE,
    },
    video::line::{PixelAttrs, OBJ},
};

pub fn render(
    line: u16,
    buf: &mut LineBuffer,
    ioregs: &IoRegisters,
    oam: &[u8; OAM_SIZE as usize],
    vram: &[u8; VRAM_SIZE as usize],
) {
    let mut cycles = if ioregs.dispcnt.hblank_interval_free() {
        954
    } else {
        1210
    };

    for obj_idx in (0..128).rev() {
        let attrs_index = obj_idx as usize * 8;
        let attr0 = ObjAttr0::new(read_u16(oam, attrs_index));
        if attr0.disabled() && !attr0.rotscale() {
            continue;
        }
        let attr1 = ObjAttr1::new(read_u16(oam, attrs_index + 2));
        let attr2 = ObjAttr2::new(read_u16(oam, attrs_index + 4));

        let (width, height) = attr1.size(attr0.shape());
        let (display_width, display_height) = if attr0.rotscale() && attr0.double_size() {
            (width * 2, height * 2)
        } else {
            (width, height)
        };

        let mut left = attr1.x();
        let top = attr0.y();
        let bottom = (top + display_height - 1) % 256;

        let in_bounds_vertical = top <= bottom && top <= line && bottom >= line;
        let in_bounds_vertical_wrapped = top > bottom && (top <= line || bottom >= line);
        if !in_bounds_vertical && !in_bounds_vertical_wrapped {
            continue;
        }

        let mut right = if let Some(right) =
            consume_obj_cycles(&mut cycles, attr0.rotscale(), display_width, left)
        {
            right
        } else {
            break;
        };

        let in_bounds_horizontal = left < 240 || right < 240;
        if !in_bounds_horizontal {
            continue;
        }

        let origin_x = FixedPoint32::from(display_width / 2);
        let origin_y = FixedPoint32::from(display_height / 2);
        let xdraw_start;
        if left < right {
            xdraw_start = FixedPoint32::from(0u32);
            right = right.min(239);
        } else {
            // we have wrapped here so we need to start drawing farther to the right
            // of the object, but there will always be enough space on screen to draw the
            // object to the end.
            left = 0;
            xdraw_start = FixedPoint32::from(display_width - right - 1);
        }
        let ydraw_start = FixedPoint32::from(if line > bottom {
            line - top
        } else {
            display_height - (bottom - line) - 1
        });

        let mut xdraw_start_distance = xdraw_start - origin_x;
        let mut ydraw_start_distance = ydraw_start - origin_y;

        let (dx, dmx, dy, dmy);

        if attr0.rotscale() {
            let params_idx = attr1.rotscale_param() as usize;
            dx = FixedPoint32::from(FixedPoint16::raw(
                (read_u16(oam, 0x06 + (params_idx * 32))) as i16,
            ));
            dmx = FixedPoint32::from(FixedPoint16::raw(
                (read_u16(oam, 0x0E + (params_idx * 32))) as i16,
            ));
            dy = FixedPoint32::from(FixedPoint16::raw(
                (read_u16(oam, 0x16 + (params_idx * 32))) as i16,
            ));
            dmy = FixedPoint32::from(FixedPoint16::raw(
                (read_u16(oam, 0x1E + (params_idx * 32))) as i16,
            ));
        } else {
            dy = FixedPoint32::from(0u32);
            dmx = FixedPoint32::from(0u32);
            dmy = FixedPoint32::from(1u32);

            if attr1.horizontal_flip() {
                dx = FixedPoint32::from(-1i32);
                // NOTE: add 1 so that we start on the other side of the center line.
                xdraw_start_distance += FixedPoint32::from(1u32);
            } else {
                dx = FixedPoint32::from(1u32);
            }

            if attr1.vertical_flip() {
                ydraw_start_distance = -ydraw_start_distance;
            }
        }

        // Down here we use the real width and height for the origin instead of the double sized
        // because I randomly wrote it and it works. Maybe one day I'll actually do the math and
        // come up with an exact reason as to why. For now I just had a feeling and I was right.
        let mut x = FixedPoint32::from(width / 2)
            + (ydraw_start_distance * dmx)
            + (xdraw_start_distance * dx);
        let mut y = FixedPoint32::from(height / 2)
            + (ydraw_start_distance * dmy)
            + (xdraw_start_distance * dy);

        let tile_data = &vram[0x10000..];

        // The number of characters (tiles) we have to jump to reach the next
        // line of the object.
        let char_stride: usize =
            if ioregs.dispcnt.obj_char_vram_mapping() == ObjCharVramMapping::OneDimensional {
                width as usize / 8
            } else if attr0.palette256() {
                16
            } else {
                32
            };
        let first_tile_index = attr2.character_name() as usize;

        let mut attrs = PixelAttrs::default();
        if ioregs.bldcnt.is_first_target(OBJ) {
            attrs.set_first_target();
        }
        if ioregs.bldcnt.is_second_target(OBJ) {
            attrs.set_second_target();
        }
        if attr0.mode() == ObjMode::SemiTransparent {
            attrs.set_semi_transparent();
        }

        // TODO: Implement mosaic
        let mosaic_x = 0;
        let mosaic_y = 0;

        if attr0.palette256() {
            const BYTES_PER_LINE: usize = 8;
            const BYTES_PER_TILE: usize = 64;

            let width = width as usize;
            let height = height as usize;

            attrs.set_8bpp();
            attrs.set_priority(attr2.priority());

            for screen_x in left as usize..=right as usize {
                let mut xi = x.integer() as usize;
                let mut yi = y.integer() as usize;
                if xi < width && yi < height {
                    if mosaic_x > 0 {
                        xi -= xi % mosaic_x;
                    }

                    if mosaic_y > 0 {
                        yi -= yi % mosaic_y;
                    }

                    // FIXME Lower bit of the tile should be ignored. From GBATEK:
                    //
                    //     When using the 256 Colors/1 Palette mode, only each second tile may be used,
                    //     the lower bit of the tile number should be zero (in 2-dimensional mapping mode,
                    //     the bit is completely ignored).
                    let tile =
                        (((first_tile_index / 2) as usize) + ((yi / 8) * char_stride) + (xi / 8))
                            & 0x3FF;

                    // When using BG Mode 3-5 (Bitmap Modes), only tile numbers 512-1023 may be used.
                    // That is because lower 16K of OBJ memory are used for BG. Attempts to use tiles 0-511 are ignored (not displayed).
                    if tile < 512 && ioregs.dispcnt.is_bitmap_mode() {
                        continue;
                    }

                    let pixel_offset =
                        (tile * BYTES_PER_TILE) + ((yi % 8) * BYTES_PER_LINE) + (xi % 8);
                    let entry = tile_data[pixel_offset as usize];
                    buf.put_obj_8bpp(attrs, screen_x, entry);
                }

                x += dx;
                y += dy;
            }
        } else {
            const BYTES_PER_LINE: usize = 4;
            const BYTES_PER_TILE: usize = 32;

            let width = width as usize;
            let height = height as usize;

            attrs.set_4bpp();
            attrs.set_priority(attr2.priority());

            for screen_x in left as usize..=right as usize {
                let mut xi = x.integer() as usize;
                let mut yi = y.integer() as usize;
                if xi < width && yi < height {
                    if mosaic_x > 0 {
                        xi -= xi % mosaic_x;
                    }

                    if mosaic_y > 0 {
                        yi -= yi % mosaic_y;
                    }

                    let tile = (first_tile_index + ((yi / 8) * char_stride) + (xi / 8)) & 0x3FF;
                    let pixel_offset =
                        (tile * BYTES_PER_TILE) + ((yi % 8) * BYTES_PER_LINE) + (xi % 8) / 2;
                    let entry = (tile_data[pixel_offset] >> ((xi % 2) << 2)) & 0xF;
                    buf.put_obj_4bpp(attrs, screen_x, attr2.palette() as _, entry);
                }

                x += dx;
                y += dy;
            }
        }
    }
}

/// Consumes the cycles required to render and object line and returns the rightmost pixel's
/// x position if there were enough cycles to render an object at all.
fn consume_obj_cycles(cycles: &mut u16, rs: bool, width: u16, left: u16) -> Option<u16> {
    let right;
    if rs {
        // affine objects require 10 cycles to start
        *cycles = cycles.saturating_sub(10);
        if *cycles == 0 {
            return None;
        }

        if width * 2 > *cycles {
            right = (left + (*cycles / 2)) % 512;
            *cycles = 0;
        } else {
            right = (left + width) % 512;
            *cycles -= width * 2;
        }
    } else if width > *cycles {
        right = (left + *cycles - 1) % 512;
        *cycles = 0;
    } else {
        right = (left + width - 1) % 512;
        *cycles -= width;
    }

    Some(right)
}

bitfields! {
    struct ObjAttr0: u16 {
        [0,7]   y, set_y: u16,
        [8]     rotscale, set_rotscale: bool,

        // When rotscale flag is set:
        [9]     double_size, set_double_size: bool,

        // When rotscale flag is clear:
        [9]     disabled, set_disabled: bool,

        [10,11] mode, set_mode: ObjMode,
        [12]    mosaic, set_mosaic: bool,
        [13]    palette256, set_palette256: bool,
        [14,15] shape, set_shape: ObjShape,
    }
}

bitfields! {
    struct ObjAttr1: u16 {
        [0,8]   x, set_x: u16,

        // When rotsccale flag is set in attr0:
        [9,13]  rotscale_param, set_rotscale_param: u16,

        // When rotscale flag is clear in attr0:
        [12]    horizontal_flip, set_horizontal: bool,
        [13]    vertical_flip, set_vertical_flip: bool,
    }
}

impl ObjAttr1 {
    pub fn size(&self, shape: ObjShape) -> (u16, u16) {
        //  Size  Square   Horizontal  Vertical
        //  0     8x8      16x8        8x16
        //  1     16x16    32x8        8x32
        //  2     32x32    32x16       16x32
        //  3     64x64    64x32       32x64
        match (self.value.bits(14, 15), shape) {
            (0, ObjShape::Square) => (8, 8),
            (1, ObjShape::Square) => (16, 16),
            (2, ObjShape::Square) => (32, 32),
            (3, ObjShape::Square) => (64, 64),

            (0, ObjShape::Horizontal) => (16, 8),
            (1, ObjShape::Horizontal) => (32, 8),
            (2, ObjShape::Horizontal) => (32, 16),
            (3, ObjShape::Horizontal) => (64, 32),

            (0, ObjShape::Vertical) => (8, 16),
            (1, ObjShape::Vertical) => (8, 32),
            (2, ObjShape::Vertical) => (16, 32),
            (3, ObjShape::Vertical) => (32, 64),

            _ => (8, 8),
        }
    }
}

bitfields! {
    struct ObjAttr2: u16 {
        [0,9]   character_name, set_character_name: u16,
        [10,11] priority, set_priority: u16,
        [12,15] palette, set_palette: u16,
    }
}

primitive_enum! {
    pub enum ObjMode: u16 {
        Normal,
        SemiTransparent,
        ObjWindow,
        Invalid,
    }
}

primitive_enum! {
    pub enum ObjShape: u16 {
        Square,
        Horizontal,
        Vertical,
        Prohibited,
    }
}
