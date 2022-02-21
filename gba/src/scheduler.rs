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
    #[inline(always)]
    pub fn schedule(&mut self, new_event: Event) {
        self.inner.borrow_mut().schedule(new_event);
    }

    #[inline(always)]
    pub fn advance(&mut self, cycles: Cycles) -> Option<(EventFn, Cycles)> {
        self.inner.borrow_mut().advance(cycles)
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
            if event.cycles_remaining > new_event.cycles_remaining {
                event.cycles_remaining -= new_event.cycles_remaining;
                insert_idx = idx + 1;
                break;
            }
            new_event.cycles_remaining -= event.cycles_remaining;
        }
        self.events.insert(insert_idx, new_event);
    }

    fn has_next_event(&self, cycles: Cycles) -> bool {
        if let Some(event) = self.events.front() {
            event.cycles_remaining <= cycles
        } else {
            false
        }
    }

    fn advance(&mut self, cycles: Cycles) -> Option<(EventFn, Cycles)> {
        if self.has_next_event(cycles) {
            let event = self.events.pop_front().unwrap();
            Some((event.callback, cycles - event.cycles_remaining))
        } else {
            None
        }
    }
}
