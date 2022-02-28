mod gamepak;
pub mod io;
mod sram;

use arm::{AccessType, Memory, Waitstates};
use byteorder::{ByteOrder as _, LittleEndian as LE};
use log::debug;
use util::array;

use self::io::IoRegisters;

pub const REGION_BIOS: u32 = 0x0;
pub const REGION_UNUSED_1: u32 = 0x1;
pub const REGION_EWRAM: u32 = 0x2;
pub const REGION_IWRAM: u32 = 0x3;
pub const REGION_IOREGS: u32 = 0x4;
pub const REGION_PAL: u32 = 0x5;
pub const REGION_VRAM: u32 = 0x6;
pub const REGION_OAM: u32 = 0x7;
pub const REGION_GAMEPAK0_LO: u32 = 0x8;
pub const REGION_GAMEPAK0_HI: u32 = 0x9;
pub const REGION_GAMEPAK1_LO: u32 = 0xA;
pub const REGION_GAMEPAK1_HI: u32 = 0xB;
pub const REGION_GAMEPAK2_LO: u32 = 0xC;
pub const REGION_GAMEPAK2_HI: u32 = 0xD;
pub const REGION_SRAM: u32 = 0xE;

pub const BIOS_SIZE: u32 = 0x4000;
pub const EWRAM_SIZE: u32 = 0x40000;
pub const IWRAM_SIZE: u32 = 0x8000;
pub const PAL_SIZE: u32 = 0x400;
pub const VRAM_SIZE: u32 = 0x18000;
pub const OAM_SIZE: u32 = 0x400;
pub const IOREGS_SIZE: u32 = 0x20A;

pub const EWRAM_MASK: u32 = 0x3FFFF;
pub const IWRAM_MASK: u32 = 0x7FFF;
pub const PAL_MASK: u32 = 0x3FF;
pub const OAM_MASK: u32 = 0x3FF;
pub const ROM_MAX_MASK: u32 = 0xFFFFFF;

pub struct GbaMemory {
    bios: Box<[u8; BIOS_SIZE as usize]>,
    ewram: Box<[u8; EWRAM_SIZE as usize]>,
    iwram: Box<[u8; IWRAM_SIZE as usize]>,
    palette: Box<[u8; PAL_SIZE as usize]>,
    vram: Box<[u8; 96 * VRAM_SIZE as usize]>,
    oam: Box<[u8; OAM_SIZE as usize]>,
    pub(crate) ioregs: Box<IoRegisters>,

    rom: Vec<u8>,

    allow_bios_access: bool,
    last_opcode: u32,

    prefetch_enabled: bool,
    gamepak_waitstates: [Waitstates; 6],
    ewram_waitstates: Waitstates,
    sram_waitstates: Waitstates,
}

impl GbaMemory {
    pub fn new() -> Self {
        GbaMemory {
            bios: array::boxed_copied(0),
            ewram: array::boxed_copied(0),
            iwram: array::boxed_copied(0),
            palette: array::boxed_copied(0),
            vram: array::boxed_copied(0),
            oam: array::boxed_copied(0),
            rom: Vec::new(),
            ioregs: Box::new(IoRegisters::default()),

            allow_bios_access: false,
            last_opcode: 0,

            prefetch_enabled: false,
            gamepak_waitstates: [0u8.into(); 6],
            ewram_waitstates: 2u8.into(),
            sram_waitstates: 8u8.into(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.store16_io(io::WAITCNT, 0x4317);
        self.ewram_waitstates = 2.into();
    }

    pub fn set_gamepak(&mut self, gamepak: Vec<u8>) {
        self.rom = gamepak;
    }

    pub fn ioregs(&self) -> &IoRegisters {
        &self.ioregs
    }

    pub fn ioregs_mut(&mut self) -> &mut IoRegisters {
        &mut self.ioregs
    }

    pub fn view8(&mut self, address: u32) -> u8 {
        match address >> 24 {
            REGION_BIOS => self.bios.get(address as usize).copied().unwrap_or(0),
            REGION_UNUSED_1 => 0,
            REGION_EWRAM => self.ewram[(address & EWRAM_MASK) as usize],
            REGION_IWRAM => self.iwram[(address & IWRAM_MASK) as usize],
            REGION_IOREGS => {
                if address < IOREGS_SIZE {
                    self.view8_io(address)
                } else {
                    0
                }
            }
            REGION_PAL => self.palette[(address & PAL_MASK) as usize],
            REGION_VRAM => self.vram[vram_offset(address)],
            REGION_OAM => self.oam[(address & OAM_MASK) as usize],

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI | REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI
            | REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                let masked = (address & ROM_MAX_MASK) as usize;
                if masked < self.rom.len() {
                    self.rom[masked]
                } else {
                    0
                }
            }

            _ => 0,
        }
    }

    pub fn view16(&mut self, mut address: u32) -> u16 {
        address &= !0x1;
        match address >> 24 {
            REGION_BIOS => {
                if (address as usize) < self.bios.len() {
                    LE::read_u16(&self.bios[address as usize..])
                } else {
                    0
                }
            }
            REGION_UNUSED_1 => 0,
            REGION_EWRAM => LE::read_u16(&self.ewram[(address & EWRAM_MASK) as usize..]),
            REGION_IWRAM => LE::read_u16(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => {
                if address < IOREGS_SIZE {
                    self.view16_io(address)
                } else {
                    0
                }
            }
            REGION_PAL => LE::read_u16(&self.palette[(address & PAL_MASK) as usize..]),
            REGION_VRAM => LE::read_u16(&self.vram[vram_offset(address)..]),
            REGION_OAM => LE::read_u16(&self.oam[(address & OAM_MASK) as usize..]),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI | REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI
            | REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                let masked = (address & ROM_MAX_MASK) as usize;
                if masked < self.rom.len() {
                    LE::read_u16(&self.rom[masked..])
                } else {
                    0
                }
            }

            _ => 0,
        }
    }

    pub fn view32(&mut self, mut address: u32) -> u32 {
        address &= !0x3;
        match address >> 24 {
            REGION_BIOS => {
                if (address as usize) < self.bios.len() {
                    LE::read_u32(&self.bios[address as usize..])
                } else {
                    0
                }
            }
            REGION_UNUSED_1 => 0,
            REGION_EWRAM => LE::read_u32(&self.ewram[(address & EWRAM_MASK) as usize..]),
            REGION_IWRAM => LE::read_u32(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => {
                if address < IOREGS_SIZE {
                    self.view32_io(address)
                } else {
                    0
                }
            }
            REGION_PAL => LE::read_u32(&self.palette[(address & PAL_MASK) as usize..]),
            REGION_VRAM => LE::read_u32(&self.vram[vram_offset(address)..]),
            REGION_OAM => LE::read_u32(&self.oam[(address & OAM_MASK) as usize..]),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI | REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI
            | REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                let masked = (address & ROM_MAX_MASK) as usize;
                if masked < self.rom.len() {
                    LE::read_u32(&self.rom[masked..])
                } else {
                    0
                }
            }

            _ => 0,
        }
    }

    fn load32_bios(&self, address: u32) -> u32 {
        if self.allow_bios_access && address <= 0x3FFC {
            LE::read_u32(&self.bios[address as usize..])
        } else {
            self.last_opcode
        }
    }

    fn load16_bios(&self, address: u32) -> u16 {
        if self.allow_bios_access && address <= 0x3FFE {
            LE::read_u16(&self.bios[address as usize..])
        } else {
            self.last_opcode as u16
        }
    }

    fn load8_bios(&self, address: u32) -> u8 {
        if self.allow_bios_access && address <= 0x3FFF {
            self.bios[address as usize]
        } else {
            self.last_opcode as u8
        }
    }
}

impl Default for GbaMemory {
    fn default() -> Self {
        Self::new()
    }
}

// Destructuring assignment until it is stabilized >:(
macro_rules! de_assign {
    ($a:ident, $b:ident, $ex:expr) => {{
        let tuple_value = $ex;
        $a = tuple_value.0;
        $b = tuple_value.1;
    }};
}

impl Memory for GbaMemory {
    fn fetch32(&mut self, address: u32, access: AccessType) -> (u32, Waitstates) {
        let (opcode, wait) = self.load32(address, access);
        self.allow_bios_access = address <= 0x4004;
        self.last_opcode = opcode;
        (opcode, wait)
    }

    fn fetch16(&mut self, address: u32, access: AccessType) -> (u16, Waitstates) {
        let (opcode, wait) = self.load16(address, access);
        self.allow_bios_access = address <= 0x4004;
        self.last_opcode = (self.last_opcode << 16) | opcode as u32;
        (opcode, wait)
    }

    fn load32(&mut self, mut address: u32, access: AccessType) -> (u32, Waitstates) {
        let value: u32;
        let mut wait = Waitstates::ZERO;
        let region = address >> 24;

        address &= !0x3; // align address
        match region {
            REGION_BIOS => value = self.load32_bios(address),
            REGION_UNUSED_1 => value = self.last_opcode,
            REGION_EWRAM => {
                value = LE::read_u32(&self.ewram[(address & EWRAM_MASK) as usize..]);
                wait = self.ewram_waitstates + self.ewram_waitstates;
            }
            REGION_IWRAM => value = LE::read_u32(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => value = self.load32_io(address),
            REGION_PAL => {
                value = LE::read_u32(&self.palette[(address & PAL_MASK) as usize..]);
                wait += Waitstates::ONE;
            }
            REGION_VRAM => {
                value = LE::read_u32(&self.vram[vram_offset(address)..]);
                wait += Waitstates::ONE;
            }
            REGION_OAM => value = LE::read_u32(&self.oam[(address & OAM_MASK) as usize..]),
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                de_assign!(value, wait, self.load32_gamepak(address, 0, access))
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                de_assign!(value, wait, self.load32_gamepak(address, 1, access))
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                de_assign!(value, wait, self.load32_gamepak(address, 2, access))
            }
            REGION_SRAM => de_assign!(value, wait, self.load32_sram(address, access)),
            _ => unreachable!(),
        }

        (value, wait)
    }

    fn load16(&mut self, mut address: u32, access: AccessType) -> (u16, Waitstates) {
        let mut value: u16;
        let mut wait = Waitstates::ZERO;
        let region = address >> 24;

        let unaligned_address = address;
        address &= !0x1; // align address
        match region {
            REGION_BIOS => value = self.load16_bios(address),
            REGION_UNUSED_1 => value = self.last_opcode as u16,
            REGION_EWRAM => {
                value = LE::read_u16(&self.ewram[(address & EWRAM_MASK) as usize..]);
                wait = self.ewram_waitstates;
            }
            REGION_IWRAM => value = LE::read_u16(&self.iwram[(address & IWRAM_MASK) as usize..]),
            REGION_IOREGS => value = self.load16_io(address),
            REGION_PAL => value = LE::read_u16(&self.palette[(address & PAL_MASK) as usize..]),
            REGION_VRAM => value = LE::read_u16(&self.vram[vram_offset(address)..]),
            REGION_OAM => value = LE::read_u16(&self.oam[(address & OAM_MASK) as usize..]),
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                de_assign!(value, wait, self.load16_gamepak(address, 0, access))
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                de_assign!(value, wait, self.load16_gamepak(address, 1, access))
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                de_assign!(value, wait, self.load16_gamepak(address, 2, access))
            }
            REGION_SRAM => de_assign!(value, wait, self.load16_sram(address, access)),
            _ => unreachable!(),
        }

        // Addresses in load16 can be unaligned. In this case the GBA just rotates the value at the aligned
        // address. This is done the same way that instructions that LDR rotate unaligned accesses.
        value = value.rotate_right((unaligned_address as u32 & 1) * 8);

        (value, wait)
    }

    fn load8(&mut self, address: u32, access: AccessType) -> (u8, Waitstates) {
        let value: u8;
        let mut wait = Waitstates::ZERO;
        let region = address >> 24;

        match region {
            REGION_BIOS => value = self.load8_bios(address),
            REGION_UNUSED_1 => value = self.last_opcode as u8,
            REGION_EWRAM => {
                value = self.ewram[(address & EWRAM_MASK) as usize];
                wait = self.ewram_waitstates;
            }
            REGION_IWRAM => value = self.iwram[(address & IWRAM_MASK) as usize],
            REGION_IOREGS => value = self.load8_io(address),
            REGION_PAL => value = self.palette[(address & PAL_MASK) as usize],
            REGION_VRAM => value = self.vram[vram_offset(address)],
            REGION_OAM => value = self.oam[(address & OAM_MASK) as usize],
            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                de_assign!(value, wait, self.load8_gamepak(address, 0, access))
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                de_assign!(value, wait, self.load8_gamepak(address, 1, access))
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                de_assign!(value, wait, self.load8_gamepak(address, 2, access))
            }
            REGION_SRAM => de_assign!(value, wait, self.load8_sram(address, access)),
            _ => unreachable!(),
        }

        (value, wait)
    }

    fn store32(&mut self, mut address: u32, value: u32, access: AccessType) -> Waitstates {
        let mut wait = Waitstates::ZERO;

        address &= !0x3;
        match address >> 24 {
            REGION_BIOS => debug!("write to BIOS 0x{:08X}=0x{:08X}", address, value),
            REGION_UNUSED_1 => debug!("write to UNUSED 0x{:08X}=0x{:08X}", address, value),
            REGION_EWRAM => {
                LE::write_u32(&mut self.ewram[(address & EWRAM_MASK) as usize..], value);
                wait = self.ewram_waitstates + self.ewram_waitstates;
            }
            REGION_IWRAM => {
                LE::write_u32(&mut self.iwram[(address & IWRAM_MASK) as usize..], value)
            }

            REGION_IOREGS => self.store32_io(address, value),
            REGION_PAL => LE::write_u32(&mut self.palette[(address & PAL_MASK) as usize..], value),
            REGION_VRAM => LE::write_u32(&mut self.vram[vram_offset(address)..], value),
            REGION_OAM => LE::write_u32(&mut self.oam[(address & OAM_MASK) as usize..], value),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                wait = self.store32_gamepak(address, value, 0, access)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                wait = self.store32_gamepak(address, value, 1, access)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                wait = self.store32_gamepak(address, value, 2, access)
            }
            REGION_SRAM => wait = self.store32_sram(address, value, access),

            _ => unreachable!(),
        }

        wait
    }

    fn store16(&mut self, mut address: u32, value: u16, access: AccessType) -> Waitstates {
        let mut wait = Waitstates::ZERO;

        address &= !0x1;
        match address >> 24 {
            REGION_BIOS => debug!("write to BIOS 0x{:08X}=0x{:08X}", address, value),
            REGION_UNUSED_1 => debug!("write to UNUSED 0x{:08X}=0x{:08X}", address, value),
            REGION_EWRAM => {
                LE::write_u16(&mut self.ewram[(address & EWRAM_MASK) as usize..], value);
                wait = self.ewram_waitstates;
            }
            REGION_IWRAM => {
                LE::write_u16(&mut self.iwram[(address & IWRAM_MASK) as usize..], value)
            }

            REGION_IOREGS => self.store16_io(address, value),
            REGION_PAL => LE::write_u16(&mut self.palette[(address & PAL_MASK) as usize..], value),
            REGION_VRAM => LE::write_u16(&mut self.vram[vram_offset(address)..], value),
            REGION_OAM => LE::write_u16(&mut self.oam[(address & OAM_MASK) as usize..], value),

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                wait = self.store16_gamepak(address, value, 0, access)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                wait = self.store16_gamepak(address, value, 1, access)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                wait = self.store16_gamepak(address, value, 2, access)
            }
            REGION_SRAM => wait = self.store16_sram(address, value, access),

            _ => unreachable!(),
        }

        wait
    }

    fn store8(&mut self, address: u32, value: u8, access: AccessType) -> Waitstates {
        let mut wait = Waitstates::ZERO;

        match address >> 24 {
            REGION_BIOS => debug!("write to BIOS 0x{:08X}=0x{:08X}", address, value),
            REGION_UNUSED_1 => debug!("write to UNUSED 0x{:08X}=0x{:08X}", address, value),
            REGION_EWRAM => {
                self.ewram[(address & EWRAM_MASK) as usize] = value;
                wait = self.ewram_waitstates;
            }
            REGION_IWRAM => self.iwram[(address & IWRAM_MASK) as usize] = value,

            REGION_IOREGS => self.store8_io(address, value),
            REGION_PAL => self.palette[(address & PAL_MASK) as usize] = value,
            REGION_VRAM => self.vram[vram_offset(address)] = value,
            REGION_OAM => self.oam[(address & OAM_MASK) as usize] = value,

            REGION_GAMEPAK0_LO | REGION_GAMEPAK0_HI => {
                wait = self.store8_gamepak(address, value, 0, access)
            }
            REGION_GAMEPAK1_LO | REGION_GAMEPAK1_HI => {
                wait = self.store8_gamepak(address, value, 1, access)
            }
            REGION_GAMEPAK2_LO | REGION_GAMEPAK2_HI => {
                wait = self.store8_gamepak(address, value, 2, access)
            }
            REGION_SRAM => wait = self.store8_sram(address, value, access),

            _ => unreachable!(),
        }

        wait
    }

    fn stall(&mut self, _cycles: arm::Cycles) {
        /* NOP */
    }
}

/// Converts an address in the range [0x06000000, 0x06FFFFFF] into an offset in VRAM accounting
/// for VRAM mirroring.
const fn vram_offset(address: u32) -> usize {
    // Even though VRAM is sized 96K (64K+32K), it is repeated in steps of 128K (64K+32K+32K,
    // the two 32K blocks itself being mirrors of each other).
    let vram128 = address % (128 * 1024); // offset in a 128KB block

    if vram128 >= (96 * 1024) {
        // this means that this address is in the later 32KB block so we just subtract 32KB to
        // mirror the previous one:
        vram128 as usize - (32 * 1024)
    } else {
        vram128 as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Most internal memory regions are mirrored across their entire 24bit address spaces.
    /// This includes `EWRAM` at `0x02XXXXXX`, `IWRAM` at `0x03XXXXXX`, `Palette RAM` at
    /// `0x05XXXXXX`, and OAM at `0x07XXXXXX`. VRAM is mirrored as well but that test is handled
    /// in [`gba_vram_memory_mirrors`].
    #[test]
    pub fn simple_gba_memory_mirrors() {
        let mut memory = GbaMemory::new();

        // EWRAM 256K mirrors:
        assert_eq!(memory.load32(0x02000000, AccessType::Seq).0, 0);
        memory.store32(0x02000000, 0xCACBCDCE, AccessType::Seq);
        assert_eq!(memory.load8(0x02000001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load8(0x02040001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load16(0x02000002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load16(0x02040002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load32(0x02000000, AccessType::Seq).0, 0xCACBCDCE);
        assert_eq!(memory.load32(0x02040000, AccessType::Seq).0, 0xCACBCDCE);

        // IWRAM 32K mirrors:
        assert_eq!(memory.load32(0x03000000, AccessType::Seq).0, 0);
        memory.store32(0x03000000, 0xCACBCDCE, AccessType::Seq);
        assert_eq!(memory.load8(0x03000001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load8(0x03008001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load16(0x03000002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load16(0x03008002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load32(0x03000000, AccessType::Seq).0, 0xCACBCDCE);
        assert_eq!(memory.load32(0x03008000, AccessType::Seq).0, 0xCACBCDCE);

        // PAL RAM 1K mirrors:
        assert_eq!(memory.load32(0x05000000, AccessType::Seq).0, 0);
        memory.store32(0x05000000, 0xCACBCDCE, AccessType::Seq);
        assert_eq!(memory.load8(0x05000001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load8(0x05000401, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load16(0x05000002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load16(0x05000402, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load32(0x05000000, AccessType::Seq).0, 0xCACBCDCE);
        assert_eq!(memory.load32(0x05000400, AccessType::Seq).0, 0xCACBCDCE);

        // OAM 1K mirrors:
        assert_eq!(memory.load32(0x07000000, AccessType::Seq).0, 0);
        memory.store32(0x07000000, 0xCACBCDCE, AccessType::Seq);
        assert_eq!(memory.load8(0x07000001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load8(0x07000401, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load16(0x07000002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load16(0x07000402, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load32(0x07000000, AccessType::Seq).0, 0xCACBCDCE);
        assert_eq!(memory.load32(0x07000400, AccessType::Seq).0, 0xCACBCDCE);
    }

    /// Like other internal memory regions `VRAM` is also mirrored across its 24bit address space.
    /// `VRAM` is `96K` in size it is mirrored in `128K` steps where the last `32K` chunk of each
    /// step is a mirror of the previous `32K`.
    ///
    /// ```none
    /// [          128K Region      ]  [          128K Region      ]
    /// [ A: 64K] [ B: 32K] [B': 32K]  [ A: 64K] [ B: 32K] [B': 32K]
    /// ```
    #[test]
    pub fn gba_vram_memory_mirrors() {
        let mut memory = GbaMemory::new();

        assert_eq!(memory.load32(0x06000000, AccessType::Seq).0, 0);
        memory.store32(0x06000000, 0xCACBCDCE, AccessType::Seq);
        memory.store32(0x06010004, 0xABACADAE, AccessType::Seq);

        // Check 128K mirror:
        assert_eq!(memory.load8(0x06000001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load8(0x06020001, AccessType::Seq).0, 0xCD);
        assert_eq!(memory.load16(0x06000002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load16(0x06020002, AccessType::Seq).0, 0xCACB);
        assert_eq!(memory.load32(0x06000000, AccessType::Seq).0, 0xCACBCDCE);
        assert_eq!(memory.load32(0x06020000, AccessType::Seq).0, 0xCACBCDCE);

        // Check 32K+32K mirror
        assert_eq!(memory.load8(0x06010005, AccessType::Seq).0, 0xAD);
        assert_eq!(memory.load8(0x06018005, AccessType::Seq).0, 0xAD);
        assert_eq!(memory.load16(0x06010006, AccessType::Seq).0, 0xABAC);
        assert_eq!(memory.load16(0x06018006, AccessType::Seq).0, 0xABAC);
        assert_eq!(memory.load32(0x06010004, AccessType::Seq).0, 0xABACADAE);
        assert_eq!(memory.load32(0x06018004, AccessType::Seq).0, 0xABACADAE);
    }
}
