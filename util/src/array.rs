use std::alloc::{alloc, Layout};

pub fn boxed_copied<T: Copy, const N: usize>(value: T) -> Box<[T; N]> {
    unsafe {
        let ptr = alloc(Layout::new::<[T; N]>()) as *mut T;
        for idx in 0..N {
            ptr.add(idx).write(value);
        }
        Box::from_raw(ptr as *mut [T; N])
    }
}

pub fn boxed<T: Clone, const N: usize>(value: T) -> Box<[T; N]> {
    unsafe {
        let ptr = alloc(Layout::new::<[T; N]>()) as *mut T;
        for idx in 0..N {
            ptr.add(idx).write(value.clone());
        }
        Box::from_raw(ptr as *mut [T; N])
    }
}

pub fn boxed_default<T: Clone + Default, const N: usize>() -> Box<[T; N]> {
    boxed(T::default())
}
