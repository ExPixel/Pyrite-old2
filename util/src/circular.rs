use std::mem::MaybeUninit;

pub struct CircularBuffer<T, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T, const CAPACITY: usize> CircularBuffer<T, CAPACITY> {
    pub fn clear(&mut self) {
        if !std::mem::needs_drop::<T>() {
            self.head = 0;
            self.tail = 0;
            self.full = false;
            return;
        }

        while let Some(ptr) = self.pop_ptr() {
            unsafe { std::ptr::drop_in_place(ptr) };
        }
    }

    pub fn push(&mut self, value: T) {
        if self.full {
            if !std::mem::needs_drop::<T>() {
                self.tail = (self.tail + 1) % CAPACITY;
            } else {
                let _ = self.pop();
            }
        }
        self.data[self.head] = MaybeUninit::new(value);
        self.head = (self.head + 1) % CAPACITY;
        self.full = self.head == self.tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.pop_ptr().map(|ptr| unsafe { ptr.read() })
    }

    fn pop_ptr(&mut self) -> Option<*mut T> {
        if self.is_empty() {
            return None;
        }
        let tail = self.tail;
        self.tail = (self.tail + 1) % CAPACITY;
        self.full = false;
        Some(self.data[tail].as_mut_ptr())
    }

    pub fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn len(&self) -> usize {
        if self.full {
            CAPACITY
        } else {
            self.head.wrapping_sub(self.tail) % CAPACITY
        }
    }
}

impl<T, const CAPACITY: usize> Default for CircularBuffer<T, CAPACITY> {
    fn default() -> Self {
        CircularBuffer {
            // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
            data: unsafe { MaybeUninit::<[MaybeUninit<T>; CAPACITY]>::uninit().assume_init() },
            head: 0,
            tail: 0,
            full: false,
        }
    }
}

impl<T, const CAPACITY: usize> Drop for CircularBuffer<T, CAPACITY> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_circular_buffer() {
        circular_buffer_test_n::<32, 32>();
        circular_buffer_test_n::<37, 37>();
    }

    fn circular_buffer_test_n<const NU32: u32, const NUSIZE: usize>() {
        assert_eq!(NU32 as usize, NUSIZE);
        println!("using values NU32={NU32}, NUSIZE={NUSIZE}");

        let mut buffer = CircularBuffer::<CountDrop, NUSIZE>::default();
        assert!(buffer.is_empty());

        (0..(NU32 / 2)).for_each(|n| buffer.push(CountDrop::new(n)));
        assert_eq!(CountDrop::active(), NUSIZE / 2);
        assert_eq!(buffer.len(), NUSIZE / 2);
        assert!(!buffer.is_empty());
        assert!(!buffer.is_full());

        (0..(NU32 / 2)).for_each(|n| assert_eq!(Some(CountDrop::new(n)), buffer.pop()));
        assert_eq!(CountDrop::active(), 0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        (0..(NU32 / 2)).for_each(|n| buffer.push(CountDrop::new(n)));
        assert_eq!(CountDrop::active(), NUSIZE / 2);
        buffer.clear();
        assert_eq!(CountDrop::active(), 0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        (0..NU32).for_each(|n| buffer.push(CountDrop::new(n)));
        assert_eq!(CountDrop::active(), NUSIZE);
        assert_eq!(buffer.len(), NUSIZE);
        assert!(!buffer.is_empty());
        assert!(buffer.is_full());

        buffer.clear();
        assert_eq!(CountDrop::active(), 0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        (0..(NU32 * 2)).for_each(|n| buffer.push(CountDrop::new(n)));
        assert_eq!(CountDrop::active(), NUSIZE);
        assert_eq!(buffer.len(), NUSIZE);
        assert!(!buffer.is_empty());
        assert!(buffer.is_full());

        drop(buffer);
        assert_eq!(CountDrop::active(), 0);
    }

    static mut ACTIVE_COUNTDROPS: usize = 0;
    #[derive(PartialEq, Eq, Debug)]
    struct CountDrop(u32);
    impl CountDrop {
        fn new(value: u32) -> Self {
            unsafe { ACTIVE_COUNTDROPS += 1 };
            CountDrop(value)
        }

        fn active() -> usize {
            unsafe { ACTIVE_COUNTDROPS }
        }
    }
    impl Drop for CountDrop {
        fn drop(&mut self) {
            unsafe {
                ACTIVE_COUNTDROPS = ACTIVE_COUNTDROPS
                    .checked_sub(1)
                    .expect("overflow while decrementing drop count")
            };
        }
    }
}
