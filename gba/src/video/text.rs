use crate::memory::{
    io::{BgControl, BgOffset},
    VRAM_SIZE,
};
use byteorder::{ByteOrder as _, LittleEndian as LE};
use util::mem::{read_u32_unchecked, read_u64_unchecked};

use super::line::LineBuffer;

type Vram = [u8; VRAM_SIZE as usize];

pub fn render_4bpp(
    buf: &mut LineBuffer,
    line: u16,
    bgidx: usize,
    bgcnt: BgControl,
    bgofs: BgOffset,
    vram: &Vram,
) {
    pub const BYTES_PER_TILE: usize = 32;
    pub const BYTES_PER_LINE: usize = 4;

    buf.layer_attrs_mut(bgidx).set_4bpp();

    // FIXME implement mosaic.
    let mosaic_x = 0;
    let mosaic_y = 0;

    let screen_size = bgcnt.screen_size();
    let screen_w = screen_size.width(false);
    let screen_h = screen_size.height(false);
    let char_base = bgcnt.character_base() as usize;

    let start_scx = bg_wrapped_x_offset(bgofs.x() as _, screen_w, mosaic_x);
    let scy = bg_wrapped_y_offset(bgofs.y() as _, screen_h, line as _, mosaic_y);
    let ty = scy as usize % 8;

    let mut dx = 0;
    let mut tile_loader = TileLoader::new(vram, bgcnt.screen_base(), start_scx, scy, screen_w);

    while dx < 240 {
        let scx = start_scx + dx;

        if scx % 8 == 0 {
            tile_loader.advance();
        }

        // try to do 8 pixels at a time if possible:
        if (scx % 8) == 0 && dx <= 232 {
            let pixel_offset =
                tile_loader.tile_pixel_offset(BYTES_PER_TILE, BYTES_PER_LINE, char_base, ty);
            let palette = tile_loader.tile_palette();

            // we read all 8 nibbles (4 bytes) in one go:
            let entries8 = unsafe { read_u32_unchecked(vram, pixel_offset) };
            if tile_loader.hflip() {
                let dx = dx as usize;
                buf.put_4bpp(bgidx, dx + 7, palette, (entries8 & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 6, palette, ((entries8 >> 4) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 5, palette, ((entries8 >> 8) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 4, palette, ((entries8 >> 12) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 3, palette, ((entries8 >> 16) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 2, palette, ((entries8 >> 20) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 1, palette, ((entries8 >> 24) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx, palette, ((entries8 >> 28) & 0xF) as u8);
            } else {
                let dx = dx as usize;
                buf.put_4bpp(bgidx, dx, palette, (entries8 & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 1, palette, ((entries8 >> 4) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 2, palette, ((entries8 >> 8) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 3, palette, ((entries8 >> 12) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 4, palette, ((entries8 >> 16) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 5, palette, ((entries8 >> 20) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 6, palette, ((entries8 >> 24) & 0xF) as u8);
                buf.put_4bpp(bgidx, dx + 7, palette, ((entries8 >> 28) & 0xF) as u8);
            }
            dx += 8;
        } else {
            let mut pixel_offset =
                tile_loader.tile_pixel_offset(BYTES_PER_TILE, BYTES_PER_LINE, char_base, ty);

            // get the x offset of the pixel:
            let tx = if tile_loader.hflip() {
                7 - (scx % 8)
            } else {
                scx % 8
            };
            pixel_offset += tx as usize / 2;

            let palette = tile_loader.tile_palette();

            let entry = (vram[pixel_offset as usize] >> ((tx % 2) << 2)) & 0xF;
            buf.put_4bpp(bgidx, dx as usize, palette, entry);
            dx += 1;
        }
    }
}

pub fn render_8bpp(
    buf: &mut LineBuffer,
    line: u16,
    bgidx: usize,
    bgcnt: BgControl,
    bgofs: BgOffset,
    vram: &Vram,
) {
    pub const BYTES_PER_TILE: usize = 64;
    pub const BYTES_PER_LINE: usize = 8;

    buf.layer_attrs_mut(bgidx).set_4bpp();

    // FIXME implement mosaic.
    let mosaic_x = 0;
    let mosaic_y = 0;

    let screen_size = bgcnt.screen_size();
    let screen_w = screen_size.width(false);
    let screen_h = screen_size.height(false);
    let char_base = bgcnt.character_base() as usize;

    let start_scx = bg_wrapped_x_offset(bgofs.x() as _, screen_w, mosaic_x);
    let scy = bg_wrapped_y_offset(bgofs.y() as _, screen_h, line as _, mosaic_y);
    let ty = scy as usize % 8;

    let mut dx = 0;
    let mut tile_loader = TileLoader::new(vram, bgcnt.screen_base(), start_scx, scy, screen_w);

    while dx < 240 {
        let scx = start_scx + dx;

        if scx % 8 == 0 {
            tile_loader.advance();
        }

        // try to do 8 pixels at a time if possible:
        if (scx % 8) == 0 && dx <= 232 {
            let pixel_offset =
                tile_loader.tile_pixel_offset(BYTES_PER_TILE, BYTES_PER_LINE, char_base, ty);

            if pixel_offset < 0x10000 {
                let mut entries = unsafe { read_u64_unchecked(vram, pixel_offset as usize) };
                if tile_loader.hflip() {
                    entries = entries.swap_bytes();
                }
                buf.put_8bpp_8(bgidx, dx as usize, entries);
            }

            dx += 8;
        } else {
            let mut pixel_offset =
                tile_loader.tile_pixel_offset(BYTES_PER_TILE, BYTES_PER_LINE, char_base, ty);

            if pixel_offset < 0x10000 {
                if tile_loader.hflip() {
                    pixel_offset += 7 - (scx as usize % 8);
                } else {
                    pixel_offset += scx as usize % 8;
                }
                let entry = vram[pixel_offset as usize];
                buf.put_8bpp(bgidx, dx as usize, entry);
            }

            dx += 1;
        }
    }
}

/// Returns the real x offset of a text mode background taking into account wrapping and
/// the mosaic register x value.
fn bg_wrapped_x_offset(offset: u32, width: u32, mosaic: u32) -> u32 {
    let wrapped = offset & (width - 1);
    if mosaic > 0 {
        wrapped - (wrapped % mosaic)
    } else {
        wrapped
    }
}

/// Returns the real y offset of a text mode background taking into account wrapping and
/// the mosaic register x value.
fn bg_wrapped_y_offset(offset: u32, height: u32, line: u32, mosaic: u32) -> u32 {
    let wrapped = (offset + line) & (height - 1);
    if mosaic > 0 {
        wrapped - (wrapped & mosaic)
    } else {
        wrapped
    }
}

struct TileLoader<'v> {
    vram: &'v Vram,
    block: u64,
    offset: usize,
    line_end: usize,
    next_area: usize,
}

impl<'v> TileLoader<'v> {
    /// This function expects that X and Y do not exceed the width and height of the screen map.
    fn new(vram: &'v Vram, base: u32, x: u32, y: u32, width: u32) -> TileLoader<'v> {
        let area = Self::get_area(x, y, width) as usize;
        // Get the x and y coordinates within the current area:
        let (area_x, area_y) = (x % 256, y % 256);
        // Get the x and y TILE coordinates within the current area:
        let (area_tx, area_ty) = (area_x / 8, area_y / 8);

        let mut offset =
            base as usize + (area * 0x800) + (area_ty as usize * 64) + (area_tx as usize * 2);
        let line_end = (offset & !0x3F) + 64;

        let misalignment = offset % 8;
        let block = if misalignment != 0 {
            let v = LE::read_u64(&vram[offset & !0x7..]); // aligned load
            if x % 8 != 0 {
                offset += 2;
                v >> (misalignment * 8)
            } else {
                // Because this is aligned, the extra offset increment and shift that is done above
                // will be done by the immediate call to advance.
                v >> ((misalignment - 2) * 8)
            }
        } else if x % 8 != 0 {
            // We're block aligned, but since we're not tile aligned, we have to preload the block
            // because a call to advance won't be done before the next pixel offset is read.
            let v = LE::read_u64(&vram[offset..]);
            offset += 2;
            v
        } else {
            0
        };

        TileLoader {
            vram,
            block,
            offset,
            line_end,
            next_area: if width > 256 {
                if area % 2 == 0 {
                    // this is on the left and we want to increment the area
                    0x800
                } else {
                    // this is on the right and we want to decrement the area
                    (-0x800isize) as usize
                }
            } else {
                0
            },
        }
    }

    /// This should be called any time we're going to draw a pixel at a tile aligned boundary.
    /// It will correctly load in the next tile (or the next block/area if necessary).
    fn advance(&mut self) {
        // We load a new block because the offset is 8 byte aligned.
        // This all works because the TileLoader does not bother loading any data to start with if
        // the first pixel being drawn is aligned to the left edge of a tile. So offset % 8 will be
        // 0 and the first call to next will load a block.

        if self.offset % 8 == 0 {
            if self.offset == self.line_end {
                self.offset = (self.offset.wrapping_sub(2) & !0x3F).wrapping_add(self.next_area);
                self.line_end = self.offset + 64;
                self.block = LE::read_u64(&self.vram[self.offset..]);
            } else {
                self.block = LE::read_u64(&self.vram[self.offset..]);
            }
        } else {
            self.block >>= 16;
        }
        self.offset += 2;
    }

    fn tile_palette(&self) -> u8 {
        ((self.block >> 12) & 0xF) as u8
    }

    fn hflip(&self) -> bool {
        (self.block & 0x400) != 0
    }

    fn tile_pixel_offset(
        &self,
        bytes_per_tile: usize,
        bytes_per_line: usize,
        char_base: usize,
        ty: usize,
    ) -> usize {
        let tile_number = (self.block & 0x3FF) as usize;
        let vflip = (self.block & 0x800) != 0;
        let ty = if vflip { 7 - ty } else { ty };
        let tile_data_start = char_base + (bytes_per_tile * tile_number);

        (tile_data_start + (ty * bytes_per_line)) as usize
    }

    fn get_area(x: u32, y: u32, width: u32) -> u32 {
        let area_y_inc = if width > 256 { 2 } else { 1 };
        (if x < 256 { 0 } else { 1 }) + (if y < 256 { 0 } else { area_y_inc })
    }
}
