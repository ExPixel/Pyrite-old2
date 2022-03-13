use arm::Memory;

use crate::{Gba, GbaMemory};

pub const TRANSFER_16BIT: bool = false;
pub const TRANSFER_32BIT: bool = true;

#[derive(Default)]
pub struct GbaDMA {
    source: u32,
    destination: u32,
    count: u32,
    ongoing: bool,
    first_transfer: bool,
}

impl GbaDMA {
    fn both_src_and_dest_in_gamepak(&self) -> bool {
        let src_region = self.source >> 24;
        let dst_region = self.destination >> 24;

        src_region >= 0x08 && src_region <= 0x0E && dst_region >= 0x08 && dst_region <= 0x0E
    }

    fn processing_cycles(&self) -> arm::Cycles {
        if self.both_src_and_dest_in_gamepak() {
            arm::Cycles::from(2u32)
        } else {
            arm::Cycles::from(4u32)
        }
    }
}

pub fn dma_enabled<const DMA: usize>(gba: &mut Gba, _: arm::Cycles) {
    log::debug!("DMA{DMA} enabled");
}

pub fn step<const DMA: usize, const TRANSFER_TYPE: bool>(gba: &mut Gba) -> arm::Cycles {
    let dma = &mut gba.dma[DMA];

    let access;
    let processing_cycles;
    if !dma.first_transfer {
        access = arm::AccessType::Seq;
        processing_cycles = arm::Cycles::ZERO;
    } else {
        access = arm::AccessType::NonSeq;
        processing_cycles = dma.processing_cycles();
    };

    let waitstates = if TRANSFER_TYPE == TRANSFER_32BIT {
        let (value, src_wait) = gba.mem.load32(dma.source, access);
        let dst_wait = gba.mem.store32(dma.destination, value, access);
        src_wait + dst_wait
    } else {
        let (value, src_wait) = gba.mem.load16(dma.source, access);
        let dst_wait = gba.mem.store16(dma.destination, value, access);
        src_wait + dst_wait
    };
    let cycles = arm::Cycles::from(2u32) + processing_cycles + waitstates;

    dma.count -= 1;
    if dma.count == 0 {
        on_dma_transfer_end::<DMA>(gba);
    }

    cycles
}

fn on_dma_transfer_end<const DMA: usize>(gba: &mut Gba) {}
