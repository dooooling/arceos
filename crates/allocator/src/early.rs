use crate::{align_up, AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::cmp::max;
use core::mem::size_of;
use core::ptr::NonNull;

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    size: usize,

    bytes_index: usize,
    page_index: usize,
    used_pages: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            bytes_index: 0,
            page_index: 0,
            size: 0,
            used_pages: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = self.start + size;
        self.bytes_index = self.start;
        self.page_index = self.end;
        self.size = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        if self.bytes_index + size <= self.page_index {
            if let Some(mem) = NonNull::new(self.bytes_index as *mut u8) {
                self.bytes_index += size;
                return Ok(mem);
            }
        }
        return Err(AllocError::NoMemory);
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {}

    fn total_bytes(&self) -> usize {
        self.size
    }

    fn used_bytes(&self) -> usize {
        self.bytes_index - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_index - self.bytes_index
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal | core::cmp::Ordering::Greater => {
                Some(self.page_index - PAGE_SIZE * num_pages)
            }
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| {
            self.used_pages += num_pages;
            self.page_index -= num_pages * PAGE_SIZE;
        })
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {}

    fn total_pages(&self) -> usize {
        let end = super::align_down(self.end, PAGE_SIZE);
        let start = super::align_up(self.start, PAGE_SIZE);
        (end - start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        let end = super::align_down(self.page_index, PAGE_SIZE);
        let start = super::align_up(self.bytes_index, PAGE_SIZE);
        (end - start) / PAGE_SIZE
    }
}
