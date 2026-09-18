//! Audio-thread allocation auditing.
//!
//! Install [`AuditAllocator`] as the process global allocator, then enter an
//! [`AudioThreadGuard`] around the callback. Allocations are counted rather
//! than panicking because panic formatting may itself allocate.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

thread_local! {
    static AUDIO_THREAD_DEPTH: Cell<usize> = const { Cell::new(0) };
}

static AUDIO_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

pub struct AuditAllocator;

unsafe impl GlobalAlloc for AuditAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: this allocator delegates directly to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` and `layout` came from the delegated system allocator.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: this allocator delegates directly to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation();
        // SAFETY: arguments came from the delegated system allocator.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn record_allocation() {
    AUDIO_THREAD_DEPTH.with(|depth| {
        if depth.get() > 0 {
            AUDIO_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// Marks the current scope as audio-thread work.
pub struct AudioThreadGuard;

impl AudioThreadGuard {
    pub fn enter() -> Self {
        AUDIO_THREAD_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

impl Drop for AudioThreadGuard {
    fn drop(&mut self) {
        AUDIO_THREAD_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

pub fn reset_audio_allocation_count() {
    AUDIO_ALLOCATIONS.store(0, Ordering::Relaxed);
}

pub fn audio_allocation_count() -> usize {
    AUDIO_ALLOCATIONS.load(Ordering::Relaxed)
}
