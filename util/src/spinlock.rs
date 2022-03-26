use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinLock<T>
where
    T: ?Sized + Send,
{
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        self.acquire_lock();
        SpinLockGuard {
            data_ref: unsafe { &mut *self.data.get() },
            lock_ref: &self.locked,
        }
    }

    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if !self.try_acquire_lock() {
            return None;
        }

        Some(SpinLockGuard {
            data_ref: unsafe { &mut *self.data.get() },
            lock_ref: &self.locked,
        })
    }
}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        SpinLock {
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false),
        }
    }

    pub fn release(_guard: SpinLockGuard<T>) {
        /* NOP */
    }
}

impl<T: ?Sized> SpinLock<T> {
    // Acquire the lock with exponential backoff.
    fn acquire_lock(&self) {
        // Try a simple busy loop for a bit (a few nanoseconds.)
        for _ in 0..5 {
            if self.try_acquire_lock() {
                return;
            }
        }

        // Try another busy loop, but this time pause between each iteration.
        for _ in 0..10 {
            if self.try_acquire_lock() {
                return;
            }
            std::hint::spin_loop();
        }

        loop {
            for _ in 0..3000 {
                if self.try_acquire_lock() {
                    return;
                }

                // Unrolled so that the compiler generates multiple `pause` instructions
                // in order to increase the idle time.
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
                std::hint::spin_loop();
            }

            // A thread isn't letting fo of the lock. Maybe it got preempted, let's wait.
            std::thread::yield_now();
        }
    }

    fn try_acquire_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
}

pub struct SpinLockGuard<'lock, T: ?Sized> {
    data_ref: &'lock mut T,
    lock_ref: &'lock AtomicBool,
}

impl<'lock, T: ?Sized> Deref for SpinLockGuard<'lock, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data_ref
    }
}

impl<'lock, T: ?Sized> DerefMut for SpinLockGuard<'lock, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_ref
    }
}

impl<'lock, T: ?Sized> Drop for SpinLockGuard<'lock, T> {
    fn drop(&mut self) {
        self.lock_ref.store(false, Ordering::Release);
    }
}

impl<T> Default for SpinLock<T>
where
    T: Default,
{
    fn default() -> Self {
        SpinLock::new(T::default())
    }
}

unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}
