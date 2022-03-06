use super::set_preserve_bits;
use util::bits::Bits as _;

macro_rules! register {
    (
        $(#[$meta:meta])* $visibility:vis
        struct $Name:ident: $InnerType:ident {
            $( [$field_start:literal$(, $field_end:literal)?] $field_get:ident, $field_set:ident: $FieldType:ident ),* $(,)?
            $( readonly = $readonly:expr  $(,)?)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
        $visibility struct $Name {
            pub value: $InnerType,
        }

        impl $Name {
            pub const READONLY: $InnerType = 0 $(| $readonly)?;

            pub const fn new(inner_value: $InnerType) -> Self {
                $Name { value: inner_value }
            }

            pub fn set_preserve_bits(&mut self, value: $InnerType) {
                set_preserve_bits(&mut self.value, value, Self::READONLY);
            }

            $(
                pub fn $field_get(&self) -> $FieldType {
                    let bits = extract_bits!(self.value, $field_start $(, $field_end)?);
                    from_bits!(bits, $InnerType, $FieldType)
                }

                pub fn $field_set(&mut self, value: $FieldType) {
                    let new_bits = <$InnerType>::from(value);
                    self.value = replace_bits!(self.value, new_bits, $field_start $(, $field_end)?);
                }
            )*
        }

        impl From<$InnerType> for $Name {
            fn from(inner_value: $InnerType) -> $Name {
                $Name { value: inner_value }
            }
        }

        impl From<$Name> for $InnerType {
            fn from(v: $Name) -> $InnerType {
                v.value
            }
        }
    };
}

macro_rules! extract_bits {
    ($value:expr, $start:expr) => {
        $value.bits($start, $start)
    };

    ($value:expr, $start:expr, $end:expr) => {
        $value.bits($start, $end)
    };
}

macro_rules! replace_bits {
    ($dst:expr, $src:expr, $start:expr) => {
        $dst.replace_bits($start, $start, $src)
    };

    ($dst:expr, $src:expr, $start:expr, $end:expr) => {
        $dst.replace_bits($start, $end, $src)
    };
}

macro_rules! from_bits {
    ($bits:expr, u32, u16) => {
        $bits as u16
    };

    ($bits:expr, $SrcType:ty, bool) => {
        $bits != 0
    };

    ($bits:expr, $SrcType:ty, $DstType:ty) => {
        <$DstType as From<$SrcType>>::from($bits)
    };
}

register! {
    /// **4000000h - DISPCNT - LCD Control (Read/Write)**
    /// 0-2   BG Mode                (0-5=Video Mode 0-5, 6-7=Prohibited)
    /// 3     Reserved / CGB Mode    (0=GBA, 1=CGB; can be set only by BIOS opcodes)
    /// 4     Display Frame Select   (0-1=Frame 0-1) (for BG Modes 4,5 only)
    /// 5     H-Blank Interval Free  (1=Allow access to OAM during H-Blank)
    /// 6     OBJ Character VRAM Mapping (0=Two dimensional, 1=One dimensional)
    /// 7     Forced Blank           (1=Allow FAST access to VRAM,Palette,OAM)
    /// 8     Screen Display BG0  (0=Off, 1=On)
    /// 9     Screen Display BG1  (0=Off, 1=On)
    /// 10    Screen Display BG2  (0=Off, 1=On)
    /// 11    Screen Display BG3  (0=Off, 1=On)
    /// 12    Screen Display OBJ  (0=Off, 1=On)
    /// 13    Window 0 Display Flag   (0=Off, 1=On)
    /// 14    Window 1 Display Flag   (0=Off, 1=On)
    /// 15    OBJ Window Display Flag (0=Off, 1=On)
    pub struct LCDControl: u16 {
        [0,2]   bg_mode, set_bg_mode: u16,
        [3]     cgb_mode, set_cgb_mode: bool,
        [4]     frame, set_framae: u16,
        [5]     hblank_interval_free, set_hblank_interval_free: u16,
        [6]     obj_char_vram_mapping, set_obj_char_vram_mapping: ObjCharVramMapping,
        [7]     forced_blank, set_forced_blank: bool,
        [15]    obj_window_display, set_obj_window_display: bool,
    }
}

impl LCDControl {
    pub fn display_bg(&self, bg: u16) -> bool {
        if bg > 3 {
            return false;
        }
        self.value.is_bit_set(bg as u32 + 8)
    }

    pub fn display_obj(&self) -> bool {
        self.value.is_bit_set(12)
    }

    pub fn is_bitmap_mode(&self) -> bool {
        (3..6).contains(&self.bg_mode())
    }
}

register! {
    /// **4000004h - DISPSTAT - General LCD Status (Read/Write)**
    /// Display status and Interrupt control. The H-Blank conditions are generated once per scanline, including for the 'hidden' scanlines during V-Blank.
    /// Bit   Expl.
    /// 0     V-Blank flag   (Read only) (1=VBlank) (set in line 160..226; not 227)
    /// 1     H-Blank flag   (Read only) (1=HBlank) (toggled in all lines, 0..227)
    /// 2     V-Counter flag (Read only) (1=Match)  (set in selected line)     (R)
    /// 3     V-Blank IRQ Enable         (1=Enable)                          (R/W)
    /// 4     H-Blank IRQ Enable         (1=Enable)                          (R/W)
    /// 5     V-Counter IRQ Enable       (1=Enable)                          (R/W)
    /// 6     Not used (0) / DSi: LCD Initialization Ready (0=Busy, 1=Ready)   (R)
    /// 7     Not used (0) / NDS: MSB of V-Vcount Setting (LYC.Bit8) (0..262)(R/W)
    /// 8-15  V-Count Setting (LYC)      (0..227)                            (R/W)
    pub struct LCDStatus: u16 {
        [0]     vblank, set_vblank: bool,
        [1]     hblank, set_hblank: bool,
        [2]     vcounter_match, set_vcounter_match: bool,
        [3]     vblank_irq_enable, set_vblank_irq_enable: bool,
        [4]     hblank_irq_enable, set_hblank_irq_enable: bool,
        [5]     vcounter_irq_enable, set_vcounter_irq_enable: bool,
        [8,15]  vcount_setting, set_vcount_setting: u16,

        readonly = 0x0047,
    }
}

register! {
    /// **4000008h - BG0CNT - BG0 Control (R/W) (BG Modes 0,1 only)**
    /// **400000Ah - BG1CNT - BG1 Control (R/W) (BG Modes 0,1 only)**
    /// **400000Ch - BG2CNT - BG2 Control (R/W) (BG Modes 0,1,2 only)**
    /// **400000Eh - BG3CNT - BG3 Control (R/W) (BG Modes 0,2 only)**
    ///   Bit   Expl.
    ///   0-1   BG Priority           (0-3, 0=Highest)
    ///   2-3   Character Base Block  (0-3, in units of 16 KBytes) (=BG Tile Data)
    ///   4-5   Not used (must be zero) (except in NDS mode: MSBs of char base)
    ///   6     Mosaic                (0=Disable, 1=Enable)
    ///   7     Colors/Palettes       (0=16/16, 1=256/1)
    ///   8-12  Screen Base Block     (0-31, in units of 2 KBytes) (=BG Map Data)
    ///   13    BG0/BG1: Not used (except in NDS mode: Ext Palette Slot for BG0/BG1)
    ///   13    BG2/BG3: Display Area Overflow (0=Transparent, 1=Wraparound)
    ///   14-15 Screen Size (0-3)
    /// Internal Screen Size (dots) and size of BG Map (bytes):
    ///   Value  Text Mode      Rotation/Scaling Mode
    ///   0      256x256 (2K)   128x128   (256 bytes)
    ///   1      512x256 (4K)   256x256   (1K)
    ///   2      256x512 (4K)   512x512   (4K)
    ///   3      512x512 (8K)   1024x1024 (16K)
    /// In case that some or all BGs are set to same priority then BG0 is having the highest, and BG3 the lowest priority.
    pub struct BgControl: u16 {
        [0,1]   priority, set_priority: u16,
        [6]     mosaic, set_mosaic: bool,
        [7]     palette_256, set_palette_256: bool,
        [13]    wraparound, set_wraparound: bool,
        [14,15] screen_size, set_screen_size: ScreenSize,
    }
}

impl BgControl {
    /// Returns the character base block offset in bytes.
    pub fn character_base(&self) -> u32 {
        self.value.bits(2, 3) as u32 * 0x4000
    }

    /// Sets the character base block offset. Must be an increment of 16KiB less than or equal to 48KB.
    pub fn set_character_base(&mut self, offset: u32) {
        debug_assert!(offset & 0x3FFF == 0, "offset must be an increment of 16KiB");
        debug_assert!(offset <= 0xC000, "offset must be less than 48KiB");
        let block = offset / 0x4000;
        self.value = self.value.replace_bits(2, 3, block as u16);
    }

    /// Returns the screen base block offset in bytes.
    pub fn screen_base(&self) -> u32 {
        self.value.bits(8, 12) as u32 * 0x800
    }

    /// Sets the screen base block offset. Must be an increment of 2KiB less than or equal to 62KiB.
    pub fn set_screen_base(&mut self, offset: u32) {
        debug_assert!(offset & 0x7FF == 0, "offset must be an increment of 2KiB");
        debug_assert!(offset <= 0xF800, "offset must be less than 62KiB");
        let block = offset / 0x800;
        self.value = self.value.replace_bits(8, 12, block as u16);
    }
}

register! {
    pub struct BgOffset: u32 {
        [0,8]   x, set_x: u16,
        [16,24] y, set_y: u16,
    }
}

register! {
    /// 4000050h - BLDCNT - Color Special Effects Selection (R/W)
    ///   Bit   Expl.
    ///   0     BG0 1st Target Pixel (Background 0)
    ///   1     BG1 1st Target Pixel (Background 1)
    ///   2     BG2 1st Target Pixel (Background 2)
    ///   3     BG3 1st Target Pixel (Background 3)
    ///   4     OBJ 1st Target Pixel (Top-most OBJ pixel)
    ///   5     BD  1st Target Pixel (Backdrop)
    ///   6-7   Color Special Effect (0-3, see below)
    ///          0 = None                (Special effects disabled)
    ///          1 = Alpha Blending      (1st+2nd Target mixed)
    ///          2 = Brightness Increase (1st Target becomes whiter)
    ///          3 = Brightness Decrease (1st Target becomes blacker)
    ///   8     BG0 2nd Target Pixel (Background 0)
    ///   9     BG1 2nd Target Pixel (Background 1)
    ///   10    BG2 2nd Target Pixel (Background 2)
    ///   11    BG3 2nd Target Pixel (Background 3)
    ///   12    OBJ 2nd Target Pixel (Top-most OBJ pixel)
    ///   13    BD  2nd Target Pixel (Backdrop)
    ///   14-15 Not used
    pub struct ColorSpecialEffects: u16 {
        [6,7]   effect, set_effect: Effect,
    }
}

impl ColorSpecialEffects {
    pub fn is_first_target(&self, layer: usize) -> bool {
        if layer > 5 {
            return false;
        }
        self.value.is_bit_set(layer as _)
    }

    pub fn is_second_target(&self, layer: usize) -> bool {
        if layer > 5 {
            return false;
        }
        self.value.is_bit_set((layer + 8) as _)
    }
}

register! {
    /// 4000052h - BLDALPHA - Alpha Blending Coefficients (R/W) (not W)
    /// Used for Color Special Effects Mode 1, and for Semi-Transparent OBJs.
    ///   Bit   Expl.
    ///   0-4   EVA Coefficient (1st Target) (0..16 = 0/16..16/16, 17..31=16/16)
    ///   5-7   Not used
    ///   8-12  EVB Coefficient (2nd Target) (0..16 = 0/16..16/16, 17..31=16/16)
    ///   13-15 Not used
    pub struct AlphaBlendingCoeff: u16 {}
}

impl AlphaBlendingCoeff {
    pub fn eva_coeff(&self) -> u16 {
        self.value.bits(0, 4).min(16)
    }

    pub fn evb_coeff(&self) -> u16 {
        self.value.bits(8, 12).min(16)
    }

    pub fn set_eva_coeff(&mut self, eva_coeff: u16) {
        self.value = self.value.replace_bits(0, 4, eva_coeff.min(16));
    }

    pub fn set_evb(&mut self, evb_coeff: u16) {
        self.value = self.value.replace_bits(8, 12, evb_coeff.min(16));
    }
}

util::primitive_enum! {
    pub enum Effect: u16 {
        None = 0,
        AlphaBlending,
        BrightnessIncrease,
        BrightnessDecrease,
    }
}

util::primitive_enum! {
    pub enum ObjCharVramMapping: u16 {
        TwoDimensional = 0,
        OneDimensional,
    }
}

pub struct ScreenSize(u8);

impl ScreenSize {
    pub fn width(&self, rotscale: bool) -> u32 {
        //   Value  Text Mode  Rotation/Scaling Mode
        //   0      256        128
        //   1      512        256
        //   2      256        512
        //   3      512        1024

        if !rotscale {
            ((self.0 as u32 + 1) << 1) * 256
        } else {
            128 << (self.0 as u32)
        }
    }

    pub fn height(&self, rotscale: bool) -> u32 {
        //   Value  Text Mode  Rotation/Scaling Mode
        //   0      256        128
        //   1      256        256
        //   2      512        512
        //   3      512        1024
        if !rotscale {
            256 << ((self.0 as u32 & 0x2) >> 1)
        } else {
            128 << (self.0 as u32)
        }
    }
}

impl From<u16> for ScreenSize {
    fn from(primitive: u16) -> ScreenSize {
        ScreenSize(primitive as u8)
    }
}

impl From<ScreenSize> for u16 {
    fn from(size: ScreenSize) -> u16 {
        size.0 as u16
    }
}
