#![feature(const_mut_refs)]

// Simple paging & allocator
use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

// -- Bump Allocator --
struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();
        
        // Lazy init check (stub logic, better to call init explicit)
        if bump.heap_start == 0 {
             return core::ptr::null_mut();
        }

        let start = bump.next;
        let align = layout.align();
        let size = layout.size();

        let align_mask = align - 1;
        let start_aligned = (start + align_mask) & !align_mask;
        let end = start_aligned + size;

        if end > bump.heap_end {
            core::ptr::null_mut()
        } else {
            bump.next = end;
            bump.allocations += 1;
            start_aligned as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

// -- Init --

pub fn init_heap() {
    // 512MB heap at 0x10000000
    unsafe {
        ALLOCATOR.lock().init(0x10000000, 512 * 1024 * 1024);
    }
}

pub fn init_paging() {
    // Check CR0.PG
    let cr0: u64;
    unsafe {
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
    }
    if (cr0 & 0x80000000) != 0 {
        crate::vga::write_string("[*] Paging enabled\n");
    }
}
