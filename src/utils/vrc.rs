use std::ptr::addr_of_mut;

/// # Virtual Reference Cell
/// 虚引用
pub struct Vrc<T> {
    cell: *mut T,

    locking: bool,
    refcount: usize,
}

impl<T> Vrc<T> {
    pub fn new(mut cell: T) -> Self {
        Vrc {
            cell: addr_of_mut!(cell),
            locking: false,
            refcount: 1,
        }
    }

    pub fn lock(&mut self) -> VrcGuard<T> {
        todo!()
    }
}

impl<T> Clone for Vrc<T> {
    fn clone(&self) -> Self {
        todo!();
    }
}
unsafe impl<T> Send for Vrc<T> {}
unsafe impl<T> Sync for Vrc<T> {}

pub struct VrcGuard<T> {
    cell: *mut T,
}

impl<'a, T> VrcGuard<T> {
    pub fn unwrap(self) -> &'a mut T {
        todo!();
    }
}

impl<T> Drop for VrcGuard<T> {
    fn drop(&mut self) {
        todo!();
    }
}
