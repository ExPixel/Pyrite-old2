use crate::memory::io::PSGChannel;

use super::Gba;
use arm::Cycles;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EventTag {
    // Use this tag for events that don't really need to be rescheduled
    // or inspected ever.
    None,

    HDraw,
    HBlank,

    Timer0,
    Timer1,
    Timer2,
    Timer3,

    DMA0,
    DMA1,
    DMA2,
    DMA3,

    IRQ,

    Stop,
    Halt,

    LengthEndPSG1,
    LengthEndPSG2,
    LengthEndPSG3,
    LengthEndPSG4,
    EnvelopeTickPSG1,
    EnvelopeTickPSG2,
    EnvelopeTickPSG4,
    SweepTickPSG1,
    SamplePSG3,
}

impl EventTag {
    pub fn timer(timer: usize) -> EventTag {
        match timer {
            0 => Self::Timer0,
            1 => Self::Timer1,
            2 => Self::Timer2,
            3 => Self::Timer3,
            _ => panic!("invalid timer for event tag"),
        }
    }

    pub fn psg_length_end(psg: PSGChannel) -> EventTag {
        match psg {
            PSGChannel::Sound1 => Self::LengthEndPSG1,
            PSGChannel::Sound2 => Self::LengthEndPSG2,
            PSGChannel::Sound3 => Self::LengthEndPSG3,
            PSGChannel::Sound4 => Self::LengthEndPSG4,
        }
    }

    pub fn psg_envelope_tick(psg: PSGChannel) -> EventTag {
        match psg {
            PSGChannel::Sound1 => Self::EnvelopeTickPSG1,
            PSGChannel::Sound2 => Self::EnvelopeTickPSG2,
            PSGChannel::Sound3 => panic!("invalid PSG for envelope tick"),
            PSGChannel::Sound4 => Self::EnvelopeTickPSG4,
        }
    }

    // pub fn dma(dma: usize) -> EventTag {
    //     match dma {
    //         0 => EventTag::DMA0,
    //         1 => EventTag::DMA1,
    //         2 => EventTag::DMA2,
    //         3 => EventTag::DMA3,
    //         _ => panic!("invalid DMA channel for event tag"),
    //     }
    // }
}

pub type EventFn = fn(gba: &mut Gba);

#[derive(Default, Clone)]
pub struct Scheduler {
    inner: Rc<RefCell<Inner>>,
}

impl Scheduler {
    pub fn schedule(&self, callback: EventFn, cycles: impl Into<Cycles>, tag: EventTag) {
        self.inner
            .borrow_mut()
            .schedule(cycles.into(), callback, tag);
    }

    pub fn contains_tag(&self, tag: EventTag) -> bool {
        self.inner
            .borrow()
            .contains_matching(|event| event.tag == tag)
    }

    pub fn reschedule(&self, callback: EventFn, cycles: impl Into<Cycles>, tag: EventTag) {
        let mut sched = self.inner.borrow_mut();
        sched.unschedule(tag);
        sched.schedule(cycles.into(), callback, tag);
    }

    pub fn reschedule_ealier(&self, callback: EventFn, cycles: impl Into<Cycles>, tag: EventTag) {
        self.inner
            .borrow_mut()
            .reschedule_ealier(cycles.into(), callback, tag);
    }

    pub fn unschedule(&self, tag: EventTag) {
        self.inner.borrow_mut().unschedule(tag);
    }

    pub fn unschedule_matching<F>(&self, predicate: F)
    where
        F: FnMut(&Event) -> bool,
    {
        self.inner.borrow_mut().unschedule_matching(predicate);
    }

    pub fn next(&self, new_time: u64) -> Option<(EventFn, u64)> {
        self.inner.borrow_mut().next(new_time)
    }

    pub fn cycles_until_next_event(&self, now: u64) -> Option<Cycles> {
        self.inner.borrow().cycles_until_next_event(now)
    }

    pub fn time(&self) -> u64 {
        self.inner.borrow().time
    }

    pub(crate) fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    #[cfg(test)]
    pub fn dump(&self) {
        for (idx, event) in self.inner.borrow().events.iter().enumerate() {
            println!("EVENT{idx}: {}", event.when);
        }
    }
}

pub struct Event {
    pub when: u64,
    callback: EventFn,
    pub tag: EventTag,
}

#[derive(Default)]
struct Inner {
    events: VecDeque<Event>,
    time: u64,
}

impl Inner {
    fn schedule(&mut self, cycles: arm::Cycles, cb: EventFn, tag: EventTag) {
        let when = self.time + u32::from(cycles) as u64;

        let mut insert_idx = self.events.len();
        for (idx, event) in self.events.iter().enumerate() {
            if event.when <= when {
                continue;
            }
            insert_idx = idx;
            break;
        }

        let event = Event {
            when,
            callback: cb,
            tag,
        };
        self.events.insert(insert_idx, event);
    }

    fn reschedule_ealier(&mut self, cycles: arm::Cycles, cb: EventFn, tag: EventTag) {
        let when = self.time + u32::from(cycles) as u64;

        let mut found_matching_tag = false;
        let mut unscheduled_matching = false;
        self.unschedule_matching(|event| {
            let matching_tag = event.tag == tag;
            let remove = matching_tag && event.when > when;
            found_matching_tag |= matching_tag;
            unscheduled_matching |= remove;
            remove
        });

        if unscheduled_matching || !found_matching_tag {
            self.schedule(cycles, cb, tag);
        }
    }

    fn contains_matching<F>(&self, predicate: F) -> bool
    where
        F: FnMut(&Event) -> bool,
    {
        self.events.iter().any(predicate)
    }

    fn unschedule_matching<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Event) -> bool,
    {
        self.events.retain(|event| !predicate(event));
    }

    fn unschedule(&mut self, tag: EventTag) {
        self.events.retain(|event| event.tag != tag);
    }

    fn next(&mut self, new_time: u64) -> Option<(EventFn, u64)> {
        if let Some(event) = self.events.front() {
            if event.when > new_time {
                self.time = new_time;
                return None;
            }
        } else {
            self.time = new_time;
            return None;
        }

        let event = self.events.pop_front().unwrap();
        self.time = event.when;
        Some((event.callback, event.when))
    }

    fn cycles_until_next_event(&self, now: u64) -> Option<Cycles> {
        if let Some(event) = self.events.front() {
            if event.when > now {
                Some(Cycles::from((event.when - now) as u32))
            } else {
                Some(Cycles::ZERO)
            }
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod test {
    use crate::{scheduler::EventTag, Gba};

    use super::Scheduler;

    #[test]
    fn basic_scheduling() {
        let mut gba = Gba::default();
        let scheduler = Scheduler::default();

        scheduler.schedule(|_| data()[0] = 1, 10u32, EventTag::HBlank);
        scheduler.schedule(|_| data()[3] = 1, 17u32, EventTag::HBlank);
        scheduler.schedule(|_| data()[1] = 1, 13u32, EventTag::HBlank);
        scheduler.schedule(|_| data()[2] = 1, 13u32, EventTag::HBlank);
        scheduler.dump();

        assert!(scheduler.next(6).is_none());
        assert_eq!(data()[0], 0);

        let (cb, now) = scheduler.next(10).expect("expected event");
        assert_eq!(now, 10);
        cb(&mut gba);
        assert!(scheduler.next(10).is_none());
        assert_eq!(*data(), [1, 0, 0, 0]);

        let (cb, now) = scheduler.next(13).expect("expected event");
        assert_eq!(now, 13);
        cb(&mut gba);
        assert_eq!(*data(), [1, 1, 0, 0]);
        let (cb, now) = scheduler.next(13).expect("expected event");
        assert_eq!(now, 13);
        cb(&mut gba);
        assert_eq!(*data(), [1, 1, 1, 0]);
        assert!(scheduler.next(13).is_none());

        let (cb, now) = scheduler.next(20).expect("expected event");
        assert_eq!(now, 17);
        cb(&mut gba);
        assert!(scheduler.next(20).is_none());
        assert_eq!(scheduler.time(), 20);
        assert_eq!(*data(), [1, 1, 1, 1]);

        static mut DATA: [u32; 4] = [0; 4];
        fn data() -> &'static mut [u32; 4] {
            unsafe { &mut DATA }
        }
    }
}
