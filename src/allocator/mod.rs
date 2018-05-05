use arch::paging::{ActivePageTable, Page};
use arch::paging::entry::EntryFlags;
use arch::paging::mapper::MapperFlushAll;

use consts::*;

#[cfg(not(feature="slab"))]
pub use self::linked_list::Allocator;

#[cfg(feature="slab")]
pub use self::slab::Allocator;

mod linked_list;
mod slab;

unsafe fn map_heap(active_table: &mut ActivePageTable, offset: usize, size: usize) {
    let mut flush_all = MapperFlushAll::new();

    let heap_start_page = Page::containing_address(offset);
    let heap_end_page = Page::containing_address(offset + size-1);
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        let result = active_table.map(page, EntryFlags::PRESENT | EntryFlags::GLOBAL | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE);
        flush_all.consume(result);
    }

    flush_all.flush(active_table);
}

pub unsafe fn init(active_table: &mut ActivePageTable) {
    let offset = KERNEL_HEAP_OFFSET;
    let size = KERNEL_HEAP_SIZE;

    // Map heap pages
    map_heap(active_table, offset, size);

    // Initialize global heap
    Allocator::init(offset, size);
}
