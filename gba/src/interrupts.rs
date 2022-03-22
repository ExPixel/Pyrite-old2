use arm::CpuException;

use crate::{
    memory::io::{Interrupt, IoRegisters},
    scheduler::{EventTag, Scheduler},
    Gba,
};

pub fn raise(interrupt: Interrupt, ioregs: &mut IoRegisters, scheduler: &Scheduler) {
    if !ioregs.ime.enabled() || !ioregs.ie_reg.enabled(interrupt) {
        return;
    }

    if !ioregs.irq_pending.has_requests() {
        scheduler.schedule(process_irq, 0, EventTag::IRQ);
    }
    ioregs.irq_pending.request(interrupt);
}

fn process_irq(gba: &mut Gba) {
    let pending = gba.mem.ioregs.irq_pending;
    gba.mem.ioregs.irq_pending.clear();

    if gba.cpu.registers.getf_i() {
        return;
    }

    gba.mem.ioregs.if_reg.inherit(pending);
    gba.cpu.exception(CpuException::IRQ, &mut gba.mem);
}
