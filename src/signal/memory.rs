use std::marker::PhantomPinned;

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

    fn slot_ptr(&mut self, id: usize) -> *mut T {
        &mut (self.slot[id]) as *mut T
    }

    pub fn write(&mut self, value: &T) {
        self.slot[self.read_id ^1] = value.clone();
    }

    pub fn write_in_place(&mut self, closure: &mut dyn FnMut(&mut T)) {
        closure(&mut self.slot[self.read_id ^1])
    }

    pub fn read_ptr(&mut self) -> *mut T {
        self.slot_ptr(self.read_id)
    }

    /// returns read ptr
    pub fn swap(&mut self) -> *mut T {
        // swap slot
        self.read_id ^= 1;
        self.read_ptr()
    }

}