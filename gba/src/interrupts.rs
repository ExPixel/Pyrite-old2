use arm::CpuException;

use crate::{
    memory::io::{Interrupt, IoRegisters},
    scheduler::Scheduler,
    Gba,
};

pub fn raise(interrupt: Interrupt, ioregs: &mut IoRegisters, scheduler: &Scheduler) {
    if !ioregs.ime.enabled() || !ioregs.ie_reg.enabled(interrupt) {
        return;
    }

    if !ioregs.irq_pending.has_requests() {
        scheduler.schedule(|gba, _| process_irq(gba), 0);
    }
    ioregs.irq_pending.request(interrupt);
}

fn process_irq(gba: &mut Gba) {
    let pending = gba.mem.ioregs.irq_pending;
    gba.mem.ioregs.irq_pending.clear();

    // If IRQs are disabled by the CPU itself, we just clear the pending IRQs and get out of here.
    if gba.cpu.registers.getf_i() {
        return;
    }

    gba.mem.ioregs.if_reg.inherit(pending);
    gba.cpu.exception(CpuException::IRQ, &mut gba.mem);
}
