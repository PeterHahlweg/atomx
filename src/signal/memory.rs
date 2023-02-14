use std::marker::PhantomPinned;

use haphazard::AtomicPtr;

// Memory should be boxed and pinned
pub struct Memory<T: Default> {
    slot: [T;2],
    read_id: usize,
    _marker: PhantomPinned
}

impl<T> Memory<T> where T: Clone + Default {

    pub fn new(value: T) -> Self {
        Memory {
            slot: [T::default(), value],
            read_id: 1,
            _marker: PhantomPinned
        }
    }

    pub fn new_read_ptr(&mut self) -> AtomicPtr<T> {
        unsafe{AtomicPtr::new(std::ptr::addr_of_mut!(self.slot[self.read_id]))}
    }

    pub fn write(&mut self, value: &T) {
        self.slot[self.write_id()] = value.clone();
    }

    pub fn write_in_place(&mut self, closure: &mut dyn FnMut(&mut T)) {
        closure(&mut self.slot[self.write_id()])
    }

    pub fn swap(&mut self, read_ptr: &AtomicPtr<T>) {
        self.swap_read_id();
        unsafe{read_ptr.store_ptr(std::ptr::addr_of_mut!(self.slot[self.read_id]))};
    }

    fn write_id(&self) -> usize {
        self.read_id ^1
    }

    fn swap_read_id(&mut self) {
        self.read_id = self.write_id()
    }

}