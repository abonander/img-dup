use std::{mem, ptr};
use std::rt::heap;
use std::sync::atomic::{AtomicUint, Ordering};
use std::sync::Arc;


pub struct ParQueue<T> where T: Send + Sync {
    ptr: *const T,
    len: uint,
    cap: uint,
    cur: AtomicUint,    
}

unsafe impl<T: Send + Sync> Send for ParQueue<T> {}

unsafe impl<T: Send + Sync> Sync for ParQueue<T> {}

impl<T> ParQueue<T> where T: Send + Sync {
    pub fn from_vec(vec: Vec<T>) -> ParQueue<T> {
        let len = vec.len();
        let cap = vec.capacity();
        let ptr = vec.as_slice().as_ptr();

        unsafe { mem::forget(vec); }

        ParQueue { ptr: ptr, len: len, cap: cap, cur: AtomicUint::new(0) }
    }

    pub fn pop(&self) -> Option<T> {
        let cur = self.cur.fetch_add(1, Ordering::Relaxed);
        if cur >= self.len { return None; }
        
        unsafe { // Adapted from `std::vec::MoveItems::next()` 
            if mem::size_of::<T>() == 0 {
                Some(ptr::read(mem::transmute(1u)))
            } else {
                Some(ptr::read(self.ptr.offset(cur as int)))
            }
        }             
    }
    
    pub fn into_iter(self) -> ParQueueIter<T> {
        ParQueueIter { queue: Arc::new(self) }    
    }
}

#[unsafe_destructor]
impl<T> Drop for ParQueue<T> where T: Send + Sync {
    fn drop(&mut self) {
        if self.cap != 0 {       
            while let Some(_) = self.pop() {}
            unsafe {
                dealloc(self.ptr, self.cap);
            }
        }
    }    
}

// Copied from `std::vec` source
#[inline]
unsafe fn dealloc<T>(ptr: *const T, len: uint) {
    if mem::size_of::<T>() != 0 {
        heap::deallocate(ptr as *mut u8,
                   len * mem::size_of::<T>(),
                   mem::min_align_of::<T>())
    }
}

pub struct ParQueueIter<T: Send + Sync> {
    queue: Arc<ParQueue<T>>,
}

impl<T> Clone for ParQueueIter<T> where T: Send + Sync {
    fn clone(&self) -> ParQueueIter<T> {
        ParQueueIter { queue: self.queue.clone() }    
    }    
}

impl<T> Iterator<T> for ParQueueIter<T> where T: Send + Sync {
    fn next(&mut self) -> Option<T> {
        self.queue.pop()    
    }
}


