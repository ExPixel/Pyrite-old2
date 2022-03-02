use super::set_preserve_bits;
use util::bits::Bits as _;

macro_rules! register {
    (
        $(#[$meta:meta])* $visibility:vis
        struct $Name:ident: $InnerType:ty {
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
                    from_bits!(bits, $FieldType)
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
    ($bits:expr, bool) => {
        $bits != 0
    };

    ($bits:expr, $Type:ty) => {
        <$Type>::from($bits)
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
    /// 4000004h - DISPSTAT - General LCD Status (Read/Write)
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

util::primitive_enum! {
    pub enum ObjCharVramMapping: u16 {
        TwoDimensional = 0,
        OneDimensional,
    }
}
