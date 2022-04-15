use crate::{
    audio, interrupts,
    memory::io::{Interrupt, IoRegisters, Timer},
    scheduler::{EventTag, Scheduler},
    Gba,
};

pub fn started(idx: usize, ioregs: &mut IoRegisters, scheduler: &Scheduler) {
    let timer = &mut ioregs.timers[idx];
    timer.origin = ioregs.time;
    timer.counter = timer.reload;

    if timer.control.count_up_timing() {
        return;
    }

    let event_tag = EventTag::timer(idx);
    let overflow_fn = match idx {
        0 => overflow::<0>,
        1 => overflow::<1>,
        2 => overflow::<2>,
        3 => overflow::<3>,
        _ => panic!("invalid timer"),
    };

    let overflow_cycles = (0x10000u32 - timer.counter as u32) << timer.control.prescaler_shift();
    scheduler.unschedule(event_tag);
    scheduler.schedule(overflow_fn, overflow_cycles, event_tag);
}

pub fn stopped(timer: usize, _timers: &mut [Timer; 4], scheduler: &Scheduler) {
    scheduler.unschedule(EventTag::timer(timer));
}

pub fn reschedule(idx: usize, timers: &mut [Timer; 4], scheduler: &Scheduler) {
    let event_tag = EventTag::timer(idx);
    let timer = &mut timers[idx];
    scheduler.unschedule(event_tag);

    if !timer.control.count_up_timing() {
        let overflow_fn = match idx {
            0 => overflow::<0>,
            1 => overflow::<1>,
            2 => overflow::<2>,
            3 => overflow::<3>,
            _ => panic!("invalid timer"),
        };

        let overflow_cycles =
            (0x10000u32 - timer.counter as u32) << timer.control.prescaler_shift();
        scheduler.schedule(overflow_fn, overflow_cycles, event_tag);
    }
}

pub fn flush(timer: &mut Timer, now: u64) {
    if !timer.control.started() || timer.control.count_up_timing() {
        return;
    }

    // This should NEVER overflow before the overflow function is called by the scheduler, which should reset the timer's counter.
    let delta = now - timer.origin;
    if delta >= (1 << timer.control.prescaler_shift()) {
        timer.counter += ((now - timer.origin) >> timer.control.prescaler_shift()) as u16;
        timer.origin = now;
    }
}

pub fn overflow<const TIMER: usize>(gba: &mut Gba) {
    let mut overflows = 1u32;

    {
        let timer = &mut gba.mem.ioregs.timers[TIMER];
        timer.counter = timer.reload;
        timer.origin = gba.mem.ioregs.time;

        let overflow_cycles =
            (0x10000u32 - timer.counter as u32) << timer.control.prescaler_shift();
        gba.scheduler
            .schedule(overflow::<TIMER>, overflow_cycles, EventTag::timer(TIMER));
    }
    after_overflow(TIMER, gba);

    let mut idx = TIMER + 1;
    while overflows > 0 && idx < 4 {
        let timer = &mut gba.mem.ioregs.timers[idx];
        if !timer.control.count_up_timing() || !timer.control.started() {
            break;
        }

        let mut rem = overflows;
        overflows = 0;
        while rem >= (0x10000u32 - timer.counter as u32) {
            overflows += 1;
            rem -= 0x10000u32 - timer.counter as u32;
            timer.counter = timer.reload;
        }
        timer.counter += rem as u16;
        for _ in 0..overflows {
            after_overflow(idx, gba);
        }
        idx += 1;
    }
}

fn after_overflow(idx: usize, gba: &mut Gba) {
    if gba.mem.ioregs.timers[idx].control.irq_enable() {
        interrupts::raise(Interrupt::timer(idx), &mut gba.mem.ioregs, &gba.scheduler);
    }

    if idx == 0 || idx == 1 {
        audio::check_fifo_timer_overflow(idx, gba);
    }
}
