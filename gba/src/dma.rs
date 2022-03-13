use arm::Memory;

use crate::{
    memory::io::{AddressControl, DMARegisters, Timing, TransferType},
    Gba,
};

pub const TRANSFER_16BIT: bool = false;
pub const TRANSFER_32BIT: bool = true;

#[derive(Default)]
pub struct GbaDMA {
    source: u32,
    destination: u32,
    count: u32,
    ongoing: bool,
    first_transfer: bool,
    repeating: bool,
    destination_inc: u32,
    source_inc: u32,
}

impl GbaDMA {
    fn copy_registers(&mut self, chan: usize, registers: &DMARegisters) {
        self.set_count(chan, registers.count);

        let (src_mask, dst_mask) = Self::address_masks(chan);
        if !self.repeating {
            self.source = registers.source.value & src_mask;
            self.destination = registers.destination.value & dst_mask;

            self.source_inc = match (
                registers.control.transfer_type(),
                registers.control.src_addr_control(),
            ) {
                (TransferType::Word, AddressControl::Increment) => 4,
                (TransferType::Word, AddressControl::Decrement) => 4i32 as u32,
                (TransferType::Halfword, AddressControl::Increment) => 2,
                (TransferType::Halfword, AddressControl::Decrement) => 2i32 as u32,
                _ => 0,
            };

            self.destination_inc = match (
                registers.control.transfer_type(),
                registers.control.dst_addr_control(),
            ) {
                (TransferType::Word, AddressControl::Increment) => 4,
                (TransferType::Word, AddressControl::Decrement) => 4i32 as u32,
                (TransferType::Word, AddressControl::IncrementReload) => 4,
                (TransferType::Halfword, AddressControl::Increment) => 2,
                (TransferType::Halfword, AddressControl::Decrement) => 2i32 as u32,
                (TransferType::Halfword, AddressControl::IncrementReload) => 2,
                _ => 0,
            };
        } else if registers.control.dst_addr_control() == AddressControl::IncrementReload {
            self.destination = registers.destination.value & dst_mask;
        }
    }

    pub fn increment(&mut self) {
        self.source = self.source.wrapping_add(self.source_inc);
        self.destination = self.destination.wrapping_add(self.destination_inc);
    }

    fn set_count(&mut self, chan: usize, count: u16) {
        self.count = if count == 0 && chan == 3 {
            0x10000
        } else if count == 0 {
            0x4000
        } else {
            count as u32
        };
    }

    fn address_masks(dma: usize) -> (u32, u32) {
        let src_mask = if dma == 0 { 0x07FFFFFF } else { 0x0FFFFFFF };
        let dst_mask = if dma == 3 { 0x0FFFFFFF } else { 0x07FFFFFF };
        (src_mask, dst_mask)
    }

    fn both_src_and_dest_in_gamepak(&self) -> bool {
        let src_region = self.source >> 24;
        let dst_region = self.destination >> 24;

        (0x08..=0x0E).contains(&src_region) && (0x08..=0x0E).contains(&dst_region)
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
    gba.dma[DMA].repeating = false;
    if gba.mem.ioregs.dma[DMA].control.timing() == Timing::Immediate {
        begin_dma::<DMA>(gba);
    }
}

pub fn dma_on_timing(gba: &mut Gba, timing: Timing) {
    try_start_dma_for_timing::<0>(gba, timing);
    try_start_dma_for_timing::<1>(gba, timing);
    try_start_dma_for_timing::<2>(gba, timing);
    try_start_dma_for_timing::<3>(gba, timing);
}

fn try_start_dma_for_timing<const DMA: usize>(gba: &mut Gba, timing: Timing) {
    if !gba.mem.ioregs.dma[DMA].control.enabled()
        || gba.mem.ioregs.dma[DMA].control.timing() != timing
    {
        return;
    }
    begin_dma::<DMA>(gba);
}

fn begin_dma<const DMA: usize>(gba: &mut Gba) {
    // FIXME Not really sure what happens here. What happens if an HBLANK
    //       DMA copies a lot of data and spills over into the next HBLANK?
    //       Is that even possible?
    if gba.dma[DMA].ongoing {
        return;
    }

    gba.dma[DMA].copy_registers(DMA, &gba.mem.ioregs.dma[DMA]);
    gba.dma[DMA].ongoing = true;

    // if gba.dma[DMA].repeating {
    //     let dst = gba.dma[DMA].destination;
    //     let src = gba.dma[DMA].source;
    //     let cnt = gba.dma[DMA].count;
    //     log::debug!("DMA{DMA} dst={dst:08X}, src={src:08X}, cnt={cnt}");
    // }

    // If any higher priority DMA channels are active, do nothing.
    // This DMA transfer will be activated when they are done.
    for higher_priority_dma in 0..DMA {
        if gba.dma[higher_priority_dma].ongoing {
            return;
        }
    }

    gba.step_fn = dma_step_fn(DMA, gba.mem.ioregs.dma[DMA].control.transfer_type());
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
    dma.increment();
    let cycles = arm::Cycles::from(2u32) + processing_cycles + waitstates;

    dma.count -= 1;
    if dma.count == 0 {
        on_dma_transfer_end::<DMA>(gba);
    }

    cycles
}

fn dma_step_fn(dma: usize, transfer_type: TransferType) -> fn(&mut Gba) -> arm::Cycles {
    match (dma, transfer_type) {
        (0, TransferType::Word) => step::<0, TRANSFER_32BIT>,
        (1, TransferType::Word) => step::<1, TRANSFER_32BIT>,
        (2, TransferType::Word) => step::<2, TRANSFER_32BIT>,
        (3, TransferType::Word) => step::<3, TRANSFER_32BIT>,

        (0, TransferType::Halfword) => step::<0, TRANSFER_16BIT>,
        (1, TransferType::Halfword) => step::<1, TRANSFER_16BIT>,
        (2, TransferType::Halfword) => step::<2, TRANSFER_16BIT>,
        (3, TransferType::Halfword) => step::<3, TRANSFER_16BIT>,

        _ => panic!("invalid DMA index"),
    }
}

fn on_dma_transfer_end<const DMA: usize>(gba: &mut Gba) {
    if !gba.mem.ioregs.dma[DMA].control.repeat() {
        gba.mem.ioregs.dma[DMA].control.set_enabled(false);
    } else {
        gba.dma[DMA].repeating = true;
    }
    gba.dma[DMA].ongoing = false;

    for dma in (DMA + 1)..4 {
        if !gba.dma[dma].ongoing {
            continue;
        }

        gba.step_fn = dma_step_fn(dma, gba.mem.ioregs.dma[dma].control.transfer_type());
        return;
    }

    gba.restore_step();
}
