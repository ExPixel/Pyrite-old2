use super::Gba;
use arm::Cycles;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub type EventFn = fn(gba: &mut Gba, late: Cycles);

#[derive(Default, Clone)]
pub struct Scheduler {
    inner: Rc<RefCell<Inner>>,
}

impl Scheduler {
    pub fn schedule(&mut self, callback: EventFn, cycles: impl Into<Cycles>) {
        self.inner.borrow_mut().schedule(Event {
            callback,
            cycles_remaining: cycles.into(),
        });
    }

    pub fn advance(&mut self, cycles: impl Into<Cycles>) -> Option<(EventFn, Cycles)> {
        self.inner.borrow_mut().advance(cycles.into())
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
}

#[cfg(test)]
mod test {
    use arm::Cycles;

    use crate::Gba;

    use super::Scheduler;

    #[test]
    fn basic_scheduling() {
        let mut gba = Gba::default();
        let mut scheduler = Scheduler::default();

        scheduler.schedule(|_, _| data()[0] = 1, 10u32);
        scheduler.schedule(|_, _| data()[3] = 1, 17u32);
        scheduler.schedule(|_, _| data()[1] = 1, 13u32);
        scheduler.schedule(|_, _| data()[2] = 1, 13u32);
        scheduler.dump();

        assert!(scheduler.advance(6u32).is_none());
        assert_eq!(data()[0], 0);

        let (cb, cycles) = scheduler.advance(6u32).expect("expected event");
        assert_eq!(cycles, Cycles::from(2u32));
        cb(&mut gba, cycles);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 0, 0, 0]);

        let (cb, cycles) = scheduler.advance(Cycles::ONE).expect("expected event");
        assert_eq!(cycles, Cycles::ZERO);
        cb(&mut gba, cycles);
        assert_eq!(*data(), [1, 1, 0, 0]);
        let (cb, cycles) = scheduler.advance(cycles).expect("expected event");
        assert_eq!(cycles, Cycles::ZERO);
        cb(&mut gba, cycles);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 1, 1, 0]);

        let (cb, cycles) = scheduler.advance(8).expect("expected event");
        assert_eq!(cycles, Cycles::from(4u32));
        cb(&mut gba, cycles);
        assert!(scheduler.advance(cycles).is_none());
        assert_eq!(*data(), [1, 1, 1, 1]);

        static mut DATA: [u32; 4] = [0; 4];
        fn data() -> &'static mut [u32; 4] {
            unsafe { &mut DATA }
        }
    }
}
