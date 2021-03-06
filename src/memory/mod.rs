//pub use self::area_frame_allocator::AreaFrameAllocator;
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
pub mod address;
mod frame;
pub mod memory_set;

pub static FRAME_ALLOCATOR: Mutex<Option<RecycleAllocator<BumpAllocator>>> = Mutex::new(None);
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

    let kernel_start = PAddr(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.start_address()).min().unwrap() as u64);
    let kernel_end = PAddr::from_kernel_virtual(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.end_address()).max().unwrap() as usize);

    let boot_info_start = PAddr(boot_info.start_address() as u64);
    let boot_info_end = PAddr(boot_info.end_address() as u64);

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

    *FRAME_ALLOCATOR.lock() = Some(RecycleAllocator::new(BumpAllocator::new(kernel_start.0 as usize, kernel_end.0 as usize, memory_map_tag.memory_areas())));

    unsafe{ init_pat(); }
    let mut active_table = remap_the_kernel(boot_info);

    let stack_alloc_range = Page::range_of(KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE,
                                            KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE + 0x1000000);
    for page in Page::range_of(KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE,
                              KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE + 0x1000000) {
        let result = active_table.map(page, EntryFlags::WRITABLE);
        unsafe { result.ignore(); }
    }
    active_table.flush_all();
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
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.set_noncore(true)
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Get the number of frames available
pub fn free_frames() -> usize {
    if let Some(ref allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.free_frames()
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Get the number of frames used
pub fn used_frames() -> usize {
    if let Some(ref allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.used_frames()
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Allocate a range of frames
pub fn allocate_frames(count: usize) -> Option<Frame> {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.allocate_frames(count)
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Deallocate a range of frames frame
pub fn deallocate_frames(frame: Frame, count: usize) {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.deallocate_frames(frame, count)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn alloc_stacks(count: usize) -> Option<Stack> {
    if let Some(ref mut allocator) = *STACK_ALLOCATOR.lock() {
        allocator.alloc_stacks(count)
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn make_page_table(set: &memory_set::MemorySet, act: &mut ActivePageTable) -> InactivePageTable {
    // use x86_64::registers::control_regs;
    // let old_table = InactivePageTable {
    //         p4_frame: Frame::containing_address(
    //             control_regs::cr3().0 as usize
    //         ),
    //     };
    // return old_table;

    let mut temporary_page = TemporaryPage::new(Page::containing_address(0xcafebabe));
    //let mut page_table = InactivePageTable::new(allocate_frames(1), &mut act, &mut temporary_page);
    let mut page_table = {
        let frame = allocate_frames(1).expect("no more frames");
        InactivePageTable::new(frame, act, &mut temporary_page)
    };
    //return page_table;

    use consts::{KERNEL_HEAP_PML4, KERNEL_PML4};
    let e510 = act.p4()[KERNEL_PML4].clone();
    let e509 = act.p4()[KERNEL_HEAP_PML4].clone();
    debug!("make_page_table act: e510={:?} e509={:?}",e510,e509);
    act.with(&mut page_table, &mut temporary_page, |pt: &mut Mapper| {
        set.map(pt);

        pt.p4_mut()[KERNEL_PML4] = e510;
        pt.p4_mut()[KERNEL_HEAP_PML4] = e509;
        let res=pt.identity_map(Frame::
        containing_address(0xfee00000), EntryFlags::WRITABLE); // LAPIC
        unsafe{
            res.ignore();
        }
        //pt.identity_map(Frame::containing_address(0xfee00000), EntryFlags::WRITABLE);
    });
    act.flush_all();
    page_table
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
    // let stack_bottom = PAddr(stack_bottom as u64).to_kernel_virtual();
    // let stack_bottom_page = Page::containing_address(stack_bottom);
    // active_table.unmap(stack_bottom_page);
    // let kernel_stack = Stack::new(stack_bottom + 8 * PAGE_SIZE, stack_bottom + 1 * PAGE_SIZE);
    // debug!("guard page at {:#x}", stack_bottom_page.start_address());

    active_table
}
