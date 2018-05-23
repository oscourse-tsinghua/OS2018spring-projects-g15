// pub use self::area_frame_allocator::AreaFrameAllocator;
pub use arch::paging::*;
pub use self::stack_allocator::Stack;
pub use self::address::*;
pub use self::frame::*;

use multiboot2::{BootInformation, MemoryArea, MemoryAreaIter};
use arch::paging::EntryFlags;
use self::bump_allocator::BumpAllocator;
use self::recycle_allocator::RecycleAllocator;
use self::stack_allocator::StackAllocator;
use spin::Mutex;

use consts::*;

// mod area_frame_allocator;
pub mod recycle_allocator;
pub mod bump_allocator;
mod stack_allocator;
mod address;
mod frame;
pub mod memory_set;

pub static ALLOCATOR: Mutex<Option<RecycleAllocator<BumpAllocator>>> = Mutex::new(None);
pub static STACK_ALLOCATOR: Mutex<Option<StackAllocator>> = Mutex::new(None);

pub fn page_fault_handler(addr: VirtualAddress) -> bool {
    return false;
}

pub fn init(boot_info: &BootInformation) -> ActivePageTable {
    assert_has_not_been_called!("memory::init must be called only once");
    debug!("boot info: {:?}", boot_info);

    let memory_map_tag = boot_info.memory_map_tag().expect(
        "Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect(
        "Elf sections tag required");

    let kernel_start = PhysicalAddress(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.start_address()).min().unwrap() as u64);
    let kernel_end = PhysicalAddress::from_kernel_virtual(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.end_address()).max().unwrap() as usize);

    let boot_info_start = PhysicalAddress(boot_info.start_address() as u64);
    let boot_info_end = PhysicalAddress(boot_info.end_address() as u64);

    println!("kernel start: {:#x}, kernel end: {:#x}",
             kernel_start,
             kernel_end);
    println!("multiboot start: {:#x}, multiboot end: {:#x}",
             boot_info_start,
             boot_info_end);
    println!("memory area:");
    for area in memory_map_tag.memory_areas() {
        println!("{:?}", area);
    }    

    // let mut frame_allocator = AreaFrameAllocator::new(
    //     kernel_start, kernel_end,
    //     boot_info_start, boot_info_end,
    //     memory_map_tag.memory_areas());

    // // Copy memory map from bootloader location
    // unsafe {
    //     for (i, entry) in MEMORY_MAP.iter_mut().enumerate() {
    //         *entry = *(0x500 as *const MemoryArea).offset(i as isize);
    //         if entry._type != MEMORY_AREA_NULL {
    //             println!("index {}, entry: {},{},{},{}", i, entry.base_addr, entry.length, entry._type, entry.acpi);
    //         }
    //     }
    // }
    // *ALLOCATOR.lock() = Some(RecycleAllocator::new(BumpAllocator::new(kernel_start.0 as usize, kernel_end.0 as usize, MemoryAreaIter::new(MEMORY_AREA_FREE))));
    *ALLOCATOR.lock() = Some(RecycleAllocator::new(BumpAllocator::new(kernel_start.0 as usize, kernel_end.0 as usize, memory_map_tag.memory_areas())));

    unsafe{ init_pat(); }
    let active_table = remap_the_kernel(boot_info);

    let stack_alloc_range = Page::range_of(KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE,
                                            KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE + 0x1000000);
    *STACK_ALLOCATOR.lock() = Some(stack_allocator::StackAllocator::new(stack_alloc_range));

    active_table
}

/// Setup page attribute table
unsafe fn init_pat() {
    use x86_64::registers::msr;
    let uncacheable = 0;
    let write_combining = 1;
    let write_through = 4;
    //let write_protected = 5;
    let write_back = 6;
    let uncached = 7;

    let pat0 = write_back;
    let pat1 = write_through;
    let pat2 = uncached;
    let pat3 = uncacheable;

    let pat4 = write_combining;
    let pat5 = pat1;
    let pat6 = pat2;
    let pat7 = pat3;

    msr::wrmsr(msr::IA32_PAT, pat7 << 56 | pat6 << 48 | pat5 << 40 | pat4 << 32
                            | pat3 << 24 | pat2 << 16 | pat1 << 8 | pat0);
}

/// Init memory module after core
/// Must be called once, and only once,
pub unsafe fn init_noncore() {
    if let Some(ref mut allocator) = *ALLOCATOR.lock() {
        allocator.set_noncore(true)
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Get the number of frames available
pub fn free_frames() -> usize {
    if let Some(ref allocator) = *ALLOCATOR.lock() {
        allocator.free_frames()
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Get the number of frames used
pub fn used_frames() -> usize {
    if let Some(ref allocator) = *ALLOCATOR.lock() {
        allocator.used_frames()
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Allocate a range of frames
pub fn allocate_frames(count: usize) -> Option<Frame> {
    if let Some(ref mut allocator) = *ALLOCATOR.lock() {
        allocator.allocate_frames(count)
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Deallocate a range of frames frame
pub fn deallocate_frames(frame: Frame, count: usize) {
    if let Some(ref mut allocator) = *ALLOCATOR.lock() {
        allocator.deallocate_frames(frame, count)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn remap_the_kernel(boot_info: &BootInformation) -> ActivePageTable
{
    let mut temporary_page = TemporaryPage::new(Page::containing_address(0xcafebabe));

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocate_frames(1).expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }
            assert!(section.start_address() as usize % PAGE_SIZE == 0,
                    "sections need to be page aligned");

            println!("mapping section at addr: {:#x}, size: {:#x}",
                section.start_address(), section.size());

            let flags = EntryFlags::from_elf_section_flags(&section);

            fn to_physical_frame(addr: usize) -> Frame {
                Frame::containing_address(
                    if addr < KERNEL_OFFSET { addr } 
                    else { addr - KERNEL_OFFSET })
            }

            let start_frame = to_physical_frame(section.start_address() as usize);
            let end_frame = to_physical_frame(section.end_address() as usize - 1);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                let page = Page::containing_address(frame.start_address().to_kernel_virtual());
                let result = mapper.map_to(page, frame, flags);
                // The flush can be ignored as this is not the active table. See later active_table.switch
                unsafe{ result.ignore(); }
            }
        }

        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        let result = mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE);
        unsafe{ result.ignore(); }

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            let result = mapper.identity_map(frame, EntryFlags::PRESENT);
            unsafe{ result.ignore(); }
        }
    });

    let old_table = active_table.switch(new_table);
    println!("NEW TABLE!!!");

    // extern { fn stack_bottom(); }
    // let stack_bottom = PhysicalAddress(stack_bottom as u64).to_kernel_virtual();
    // let stack_bottom_page = Page::containing_address(stack_bottom);
    // active_table.unmap(stack_bottom_page);
    // let kernel_stack = Stack::new(stack_bottom + 8 * PAGE_SIZE, stack_bottom + 1 * PAGE_SIZE);
    // debug!("guard page at {:#x}", stack_bottom_page.start_address());

    active_table
}
