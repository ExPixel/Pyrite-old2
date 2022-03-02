// LCD I/O
pub const DISPCNT: u32 = 0x04000000;
pub const GREENSWAP: u32 = 0x04000002;
pub const DISPSTAT: u32 = 0x04000004;
pub const VCOUNT: u32 = 0x04000006;
pub const BG0CNT: u32 = 0x04000008;
pub const BG1CNT: u32 = 0x0400000A;
pub const BG2CNT: u32 = 0x0400000C;
pub const BG3CNT: u32 = 0x0400000E;
pub const BG0HOFS: u32 = 0x04000010;
pub const BG0VOFS: u32 = 0x04000012;
pub const BG1HOFS: u32 = 0x04000014;
pub const BG1VOFS: u32 = 0x04000016;
pub const BG2HOFS: u32 = 0x04000018;
pub const BG2VOFS: u32 = 0x0400001A;
pub const BG3HOFS: u32 = 0x0400001C;
pub const BG3VOFS: u32 = 0x0400001E;
pub const BG2PA: u32 = 0x04000020;
pub const BG2PB: u32 = 0x04000022;
pub const BG2PC: u32 = 0x04000024;
pub const BG2PD: u32 = 0x04000026;
pub const BG2X: u32 = 0x04000028;
pub const BG2X_HI: u32 = 0x0400002A;
pub const BG2Y: u32 = 0x0400002C;
pub const BG2Y_HI: u32 = 0x0400002E;
pub const BG3PA: u32 = 0x04000030;
pub const BG3PB: u32 = 0x04000032;
pub const BG3PC: u32 = 0x04000034;
pub const BG3PD: u32 = 0x04000036;
pub const BG3X: u32 = 0x04000038;
pub const BG3X_HI: u32 = 0x0400003A;
pub const BG3Y: u32 = 0x0400003C;
pub const BG3Y_HI: u32 = 0x0400003E;
pub const WIN0H: u32 = 0x04000040;
pub const WIN1H: u32 = 0x04000042;
pub const WIN0V: u32 = 0x04000044;
pub const WIN1V: u32 = 0x04000046;
pub const WININ: u32 = 0x04000048;
pub const WINOUT: u32 = 0x0400004A;
pub const MOSAIC: u32 = 0x0400004C;
pub const MOSAIC_HI: u32 = 0x0400004E;
pub const BLDCNT: u32 = 0x04000050;
pub const BLDALPHA: u32 = 0x04000052;
pub const BLDY: u32 = 0x04000054;

// Sound Registers
pub const SOUND1CNT_L: u32 = 0x04000060;
pub const SOUND1CNT_H: u32 = 0x04000062;
pub const SOUND1CNT_X: u32 = 0x04000064;
pub const SOUND2CNT_L: u32 = 0x04000068;
pub const SOUND2CNT_H: u32 = 0x0400006C;
pub const SOUND3CNT_L: u32 = 0x04000070;
pub const SOUND3CNT_H: u32 = 0x04000072;
pub const SOUND3CNT_X: u32 = 0x04000074;
pub const SOUND4CNT_L: u32 = 0x04000078;
pub const SOUND4CNT_H: u32 = 0x0400007C;
pub const SOUNDCNT_L: u32 = 0x04000080;
pub const SOUNDCNT_H: u32 = 0x04000082;
pub const SOUNDCNT_X: u32 = 0x04000084;
pub const SOUNDCNT_X_H: u32 = 0x04000086;
pub const SOUNDBIAS: u32 = 0x04000088;
pub const SOUNDBIAS_H: u32 = 0x0400008A;
pub const FIFO_A: u32 = 0x040000A0;
pub const FIFO_B: u32 = 0x040000A4;

// Sound Registers (Using NR names)
pub const NR10: u32 = 0x04000060;
pub const NR11: u32 = 0x04000062;
pub const NR12: u32 = 0x04000063;
pub const NR13: u32 = 0x04000064;
pub const NR14: u32 = 0x04000065;

pub const NR21: u32 = 0x04000068;
pub const NR22: u32 = 0x04000069;
pub const NR23: u32 = 0x0400006C;
pub const NR24: u32 = 0x0400006D;

pub const NR30: u32 = 0x04000070;
pub const NR31: u32 = 0x04000072;
pub const NR32: u32 = 0x04000073;
pub const NR33: u32 = 0x04000074;
pub const NR34: u32 = 0x04000075;

pub const NR41: u32 = 0x04000078;
pub const NR42: u32 = 0x04000079;
pub const NR43: u32 = 0x0400007C;
pub const NR44: u32 = 0x0400007D;

pub const NR50: u32 = 0x04000080;
pub const NR51: u32 = 0x04000081;
pub const NR52: u32 = 0x04000084;

// DMA Transfer Channels
pub const DMA0SAD: u32 = 0x040000B0;
pub const DMA0SAD_H: u32 = 0x040000B2;
pub const DMA0DAD: u32 = 0x040000B4;
pub const DMA0DAD_H: u32 = 0x040000B6;
pub const DMA0CNT_L: u32 = 0x040000B8;
pub const DMA0CNT_H: u32 = 0x040000BA;
pub const DMA1SAD: u32 = 0x040000BC;
pub const DMA1SAD_H: u32 = 0x040000BE;
pub const DMA1DAD: u32 = 0x040000C0;
pub const DMA1DAD_H: u32 = 0x040000C2;
pub const DMA1CNT_L: u32 = 0x040000C4;
pub const DMA1CNT_H: u32 = 0x040000C6;
pub const DMA2SAD: u32 = 0x040000C8;
pub const DMA2SAD_H: u32 = 0x040000CA;
pub const DMA2DAD: u32 = 0x040000CC;
pub const DMA2DAD_H: u32 = 0x040000CE;
pub const DMA2CNT_L: u32 = 0x040000D0;
pub const DMA2CNT_H: u32 = 0x040000D2;
pub const DMA3SAD: u32 = 0x040000D4;
pub const DMA3SAD_H: u32 = 0x040000D6;
pub const DMA3DAD: u32 = 0x040000D8;
pub const DMA3DAD_H: u32 = 0x040000DA;
pub const DMA3CNT_L: u32 = 0x040000DC;
pub const DMA3CNT_H: u32 = 0x040000DE;

// Timer Registers
pub const TM0CNT_L: u32 = 0x04000100;
pub const TM0CNT_H: u32 = 0x04000102;
pub const TM1CNT_L: u32 = 0x04000104;
pub const TM1CNT_H: u32 = 0x04000106;
pub const TM2CNT_L: u32 = 0x04000108;
pub const TM2CNT_H: u32 = 0x0400010A;
pub const TM3CNT_L: u32 = 0x0400010C;
pub const TM3CNT_H: u32 = 0x0400010E;

// Serial Communication (1)_
pub const SIODATA32: u32 = 0x04000120;
pub const SIOMULTI0: u32 = 0x04000120;
pub const SIOMULTI1: u32 = 0x04000122;
pub const SIOMULTI2: u32 = 0x04000124;
pub const SIOMULTI3: u32 = 0x04000126;
pub const SIOCNT: u32 = 0x04000128;
pub const SIOMLT_SEND: u32 = 0x0400012A;
pub const SIODATA8: u32 = 0x0400012A;

// Keypad Input
pub const KEYINPUT: u32 = 0x04000130;
pub const KEYCNT: u32 = 0x04000132;

// Serial Communication (2)
pub const RCNT: u32 = 0x04000134;
pub const IR: u32 = 0x04000136;
pub const JOYCNT: u32 = 0x04000140;
pub const JOY_RECV: u32 = 0x04000150;
pub const JOY_TRANS: u32 = 0x04000154;
pub const JOYSTAT: u32 = 0x04000158;

// Interrupt, Waitstate, and Power-Down Control
pub const IE: u32 = 0x04000200;
pub const IF: u32 = 0x04000202;
pub const WAITCNT: u32 = 0x04000204;
pub const IME: u32 = 0x04000208;
pub const IME_HI: u32 = 0x040020A;
pub const POSTFLG: u32 = 0x04000300;
pub const HALTCNT: u32 = 0x04000301;
pub const BUG410: u32 = 0x04000410;
pub const IMC: u32 = 0x04000800;
pub const IMC_H: u32 = 0x04000802;
