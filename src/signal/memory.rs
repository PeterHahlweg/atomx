use std::{marker::PhantomPinned, pin::Pin, ptr};

use haphazard::AtomicPtr;

pub struct Memory<T: Default> {
    slot: [T;2],
    read_id: usize,
    _marker: PhantomPinned
}

impl<T> Memory<T> where T: Clone + Default {

    pub fn new(value: T) -> Pin<Box<Self>> {
        Box::pin(Memory {
            slot: [T::default(), value],
            read_id: 1,
            _marker: PhantomPinned
        })
    }

    pub fn new_read_ptr(self: &mut Pin<Box<Self>>) -> AtomicPtr<T> {
        unsafe{
            // Safety:  - self is never moved here
            //          - only some data will be written to a memory slot
            //          - pin guaranties should not be violated by this
            let memory = self.as_mut().get_unchecked_mut();
            // Safety:  - self is always known to be properly initialized
            //          - and so is the slot array
            AtomicPtr::new(ptr::addr_of_mut!(memory.slot[memory.read_id]))
        }
    }

    pub fn write(self: &mut Pin<Box<Self>>, value: &T) {
        // Safety:  - self is never moved here
        //          - only some data will be written to a memory slot
        //          - pin guaranties should not be violated by this
        let memory = unsafe{self.as_mut().get_unchecked_mut()};
        memory.slot[memory.write_id()] = value.clone();
    }

    pub fn write_in_place(self: &mut Pin<Box<Self>>, closure: &mut dyn FnMut(&mut T)) {
        // Safety:  - self is never moved here
        //          - only some data will be written to a memory slot through a closure
        //          - pin guaranties should not be violated by this
        let memory = unsafe{self.as_mut().get_unchecked_mut()};
        closure(&mut memory.slot[memory.write_id()])
    }

    pub fn swap(self: &mut Pin<Box<Self>>, read_ptr: &AtomicPtr<T>) {
        // Safety:  - self is never moved here
        //          - only some data will be written to a memory slot
        //          - pin guaranties should not be violated by this
        let memory = unsafe{self.as_mut().get_unchecked_mut()};
        memory.swap_read_id();
        // Safety:  - self is always known to be properly initialized
        //          - and so is the slot array
        unsafe{read_ptr.store_ptr(std::ptr::addr_of_mut!(memory.slot[memory.read_id]))};
    }

    fn write_id(&self) -> usize {
        self.read_id ^1
    }

    fn swap_read_id(&mut self) {
        self.read_id = self.write_id()
    }

}