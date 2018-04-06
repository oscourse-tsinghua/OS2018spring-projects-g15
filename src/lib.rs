#![feature(lang_items)]
#![feature(unique, const_unique_new)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![no_std]

#[macro_use]
mod vga_buffer;
mod memory;

#[macro_use]
extern crate bitflags;

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page

    print_name();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag required");
    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length: 0x{:x}",
            area.base_addr, area.length);
    }

    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("Elf-sections tag required");
    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.addr, section.size, section.flags);
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr)
        .min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
        .max().unwrap();

    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    println!("kernel start: 0x{:x}, kernel end: 0x{:x}", {kernel_start}, {kernel_end});
    println!("multiboot start: 0x{:x}, multiboot end: 0x{:x}", {multiboot_start}, {multiboot_end});
    
    let mut frame_allocator = memory::AreaFrameAllocator::new(
    kernel_start as usize, kernel_end as usize, multiboot_start,
    multiboot_end, memory_map_tag.memory_areas());
    // use memory::FrameAllocator;
    // for i in 0.. {
    //     use memory::FrameAllocator;
    //     // println!("{:?}", frame_allocator.allocate_frame());
    //     if let None = frame_allocator.allocate_frame() {
    //         println!("allocated {} frames", i);
    //         break;
    //     }
    // }

    memory::test_paging(&mut frame_allocator);

    loop{}
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}

#[lang = "panic_fmt"] #[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\n !! KERNEL PANIC !!");
    println!("{} at line {}:", file, line);
    println!("    {}", fmt);
    loop{}
}

fn print_name() {
    vga_buffer::clear_screen();
    println!(" _______     __     __    _______    ______     _______     ________  ");
    println!("|  ____  \\  |  |   |  |  /  _____|  / _____ \\  |  _____ \\  |  ______| ");
    println!("| |____  |  |  |   |  | |  |       | |     | | | |_____ |  | |______  ");
    println!("|  ___  _/  |  |   |  | |  |       | |     | | |  ___  _/  |  ______| ");
    println!("| |   \\ \\   |  |   |  | |  |       | |     | | | |   \\ \\   | |        ");
    println!("| |    \\ \\  |  \\___/  | |  |_____  | |_____| | | |    \\ \\  | |______  ");
    println!("|_/     \\_\\  \\_______/   \\_______|  \\_______/  |_/     \\_\\ |________| ");
}