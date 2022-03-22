use super::Gba;
use arm::Cycles;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EventTag {
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
}

impl EventTag {
    pub fn timer(timer: usize) -> EventTag {
        match timer {
            0 => EventTag::Timer0,
            1 => EventTag::Timer1,
            2 => EventTag::Timer2,
            3 => EventTag::Timer3,
            _ => panic!("invalid timer for event tag"),
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
        self.inner.borrow_mut().schedule(Event {
            callback,
            cycles_remaining: cycles.into(),
            tag,
        });
    }

    pub fn unschedule(&self, tag: EventTag) {
        self.inner.borrow_mut().unschedule(tag);
    }

    pub fn advance(&self, cycles: impl Into<Cycles>) -> Option<(EventFn, Cycles)> {
        self.inner.borrow_mut().advance(cycles.into())
    }

    pub fn next_event_cycles(&self) -> Option<arm::Cycles> {
        self.inner.borrow().next_event_cycles()
    }

    pub(crate) fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    #[cfg(test)]
    pub fn dump(&self) {
        for (idx, event) in self.inner.borrow().events.iter().enumerate() {
            println!("EVENT{}: {} cycles", idx, event.cycles_remaining);
        }
    }
}

pub struct Event {
    callback: EventFn,
    cycles_remaining: Cycles,
    tag: EventTag,
}

#[derive(Default)]
struct Inner {
    events: VecDeque<Event>,
}

impl Inner {
    fn schedule(&mut self, mut new_event: Event) {
        let mut insert_idx = self.events.len();
        for (idx, event) in self.events.iter_mut().enumerate() {
            if new_event.cycles_remaining < event.cycles_remaining {
                event.cycles_remaining -= new_event.cycles_remaining;
                insert_idx = idx;
                break;
            }
            new_event.cycles_remaining -= event.cycles_remaining;
        }
        self.events.insert(insert_idx, new_event);
    }

    fn unschedule(&mut self, tag: EventTag) {
        self.events.retain(|event| event.tag != tag);
    }

    fn advance(&mut self, cycles: Cycles) -> Option<(EventFn, Cycles)> {
        if let Some(event) = self.events.front_mut() {
            if event.cycles_remaining > cycles {
                event.cycles_remaining -= cycles;
                return None;
            }
        } else {
            return None;
        }

        let event = self.events.pop_front().unwrap();
        Some((event.callback, cycles - event.cycles_remaining))
    }

    fn next_event_cycles(&self) -> Option<arm::Cycles> {
        self.events.front().map(|event| event.cycles_remaining)
    }

    fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod test {
    use arm::Cycles;

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

        assert!(scheduler.advance(6u32).is_none());
        assert_eq!(data()[0], 0);

        let (cb, cycles) = scheduler.advance(6u32).expect("expected event");
        assert_eq!(cycles, Cycles::from(2u32));
        cb(&mut gba);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 0, 0, 0]);

        let (cb, cycles) = scheduler.advance(Cycles::ONE).expect("expected event");
        assert_eq!(cycles, Cycles::ZERO);
        cb(&mut gba);
        assert_eq!(*data(), [1, 1, 0, 0]);
        let (cb, cycles) = scheduler.advance(cycles).expect("expected event");
        assert_eq!(cycles, Cycles::ZERO);
        cb(&mut gba);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 1, 1, 0]);

        let (cb, cycles) = scheduler.advance(8).expect("expected event");
        assert_eq!(cycles, Cycles::from(4u32));
        cb(&mut gba);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 1, 1, 1]);

        static mut DATA: [u32; 4] = [0; 4];
        fn data() -> &'static mut [u32; 4] {
            unsafe { &mut DATA }
        }
    }
}
