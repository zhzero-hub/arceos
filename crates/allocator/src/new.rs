use crate::{AllocResult, BaseAllocator, ByteAllocator, AllocError};
use talc::*;

struct MyOomHandler {
    heap: Span,
}

impl OomHandler for MyOomHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: core::alloc::Layout) -> Result<(), ()> {
        // Talc doesn't have enough memory, and we just got called!
        // We'll go through an example of how to handle this situation.
    
        // We can inspect `layout` to estimate how much we should free up for this allocation
        // or we can extend by any amount (increasing powers of two has good time complexity).
        // (Creating another heap with `claim` will also work.)
    
        // This function will be repeatedly called until we free up enough memory or 
        // we return Err(()) causing allocation failure. Be careful to avoid conditions where 
        // the heap isn't sufficiently extended indefinitely, causing an infinite loop.
    
        // an arbitrary address limit for the sake of example
        const HEAP_TOP_LIMIT: *mut u8 = 0x80000000 as *mut u8;
    
        let old_heap: Span = talc.oom_handler.heap;
    
        // we're going to extend the heap upward, doubling its size
        // but we'll be sure not to extend past the limit
        let new_heap: Span = old_heap.extend(0, old_heap.size()).below(HEAP_TOP_LIMIT);
    
        if new_heap == old_heap {
            // we won't be extending the heap, so we should return Err
            return Err(());
        }
    
        unsafe {
            // we're assuming the new memory up to HEAP_TOP_LIMIT is unused and allocatable
            talc.oom_handler.heap = talc.extend(old_heap, new_heap);
        }
    
        Ok(())
    }
}

pub struct MyNewAllocator {
    talck: Talc<ErrOnOom>,
    total: usize,
    avail: usize,
}

impl MyNewAllocator {
    pub const fn new() -> MyNewAllocator {
        MyNewAllocator{talck: Talc::new(ErrOnOom), total: 0, avail: 0}
    }
}

impl BaseAllocator for MyNewAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.add_memory(start, size);
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // unsafe { self.inner.add_to_heap(start, start + size) };
        self.total = self.total + _size;
        self.avail = self.avail + _size;
        unsafe {
            self.talck.claim(Span::new(_start as *mut u8, (_start+_size) as *mut u8));
        }
        Ok(())
    }
}

impl ByteAllocator for MyNewAllocator {
    fn alloc(&mut self, layout: core::alloc::Layout) -> AllocResult<core::ptr::NonNull<u8>> {
        self.avail -= layout.size();
        unsafe {
            self.talck.malloc(layout).map_err(|_| AllocError::NoMemory)
        }
    }

    fn available_bytes(&self) -> usize {
        self.avail
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        self.avail += layout.size();
        unsafe {
            self.talck.free(pos, layout)
        }
    }

    fn total_bytes(&self) -> usize {
        self.total
    }

    fn used_bytes(&self) -> usize {
        self.total - self.avail
    }
}
