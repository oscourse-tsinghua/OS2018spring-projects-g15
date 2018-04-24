pub use self::area_frame_allocator::AreaFrameAllocator;
pub use arch::paging::*;
pub use self::stack_allocator::Stack;
pub use self::address::*;
pub use self::frame::*;

use multiboot2::BootInformation;
use arch::paging;
use arch::paging::EntryFlags;

use consts::KERNEL_OFFSET;

mod area_frame_allocator;
pub mod heap_allocator;
mod stack_allocator;
mod address;
mod frame;

pub fn init(boot_info: &BootInformation) -> MemoryController {
    assert_has_not_been_called!("memory::init must be called only once");

    let memory_map_tag = boot_info.memory_map_tag().expect(
        "Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect(
        "Elf sections tag required");

    let kernel_start = PhysicalAddress(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.start_address()).min().unwrap() as u64);
    let kernel_end = PhysicalAddress::from_kernel_virtual(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.end_address()).max().unwrap());

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
        println!("  addr: {:#x}, size: {:#x}", area.base_addr, area.length);
    }    

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start, kernel_end,
        boot_info_start, boot_info_end,
        memory_map_tag.memory_areas());

    let mut active_table = remap_the_kernel(&mut frame_allocator, boot_info);

    use self::paging::Page;
    use consts::{KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE};

    let heap_start_page = Page::containing_address(KERNEL_HEAP_OFFSET);
    let heap_end_page = Page::containing_address(KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE-1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_alloc_start,
                                                      stack_alloc_end);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };
    
    MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    }
}


pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
    -> ActivePageTable
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page::containing_address(0xcafebabe), allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
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
            assert!(section.start_address() % PAGE_SIZE == 0,
                    "sections need to be page aligned");

            println!("mapping section at addr: {:#x}, size: {:#x}",
                section.addr, section.size);

            let flags = EntryFlags::from_elf_section_flags(section);

            fn to_physical_frame(addr: usize) -> Frame {
                Frame::containing_address(
                    if addr < KERNEL_OFFSET { addr } 
                    else { addr - KERNEL_OFFSET })
            }

            let start_frame = to_physical_frame(section.start_address());
            let end_frame = to_physical_frame(section.end_address() - 1);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                let page = Page::containing_address(frame.start_address().to_kernel_virtual());
                mapper.map_to(page, frame, flags, allocator);
            }
        }

        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator);

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);
    println!("NEW TABLE!!!");

    // turn the stack bottom into a guard page
    extern { fn stack_bottom(); }
    let stack_bottom = PhysicalAddress(stack_bottom as u64).to_kernel_virtual();
    let stack_bottom_page = Page::containing_address(stack_bottom);
    active_table.unmap(stack_bottom_page, allocator);
    println!("guard page at {:#x}", stack_bottom_page.start_address());

    active_table
}

pub struct MemoryController {
    active_table: paging::ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController { ref mut active_table,
                                    ref mut frame_allocator,
                                    ref mut stack_allocator } = self;
        stack_allocator.alloc_stack(active_table, frame_allocator,
                                    size_in_pages)
    }
    
    pub fn map_page_identity(&mut self, addr: usize) {
        let frame = Frame::containing_address(addr);
        let flags = EntryFlags::WRITABLE;
        self.active_table.identity_map(frame, flags, &mut self.frame_allocator);
    }

    pub fn map_page_p2v(&mut self, addr: PhysicalAddress) {
        let page = Page::containing_address(addr.to_kernel_virtual());
        let frame = Frame::containing_address(addr.get());
        let flags = EntryFlags::WRITABLE;
        self.active_table.map_to(page, frame, flags, &mut self.frame_allocator);
    }
    pub fn print_page_table(&self) {
        debug!("{:?}", self.active_table);
    }
}
