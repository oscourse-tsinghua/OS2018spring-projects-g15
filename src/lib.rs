#![feature(lang_items)]
#![feature(unique, const_unique_new)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(const_atomic_usize_new)]
#![feature(abi_x86_interrupt)]
#![no_std]

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;

#[macro_use]
extern crate bitflags;

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate once;
#[macro_use]
extern crate lazy_static;
extern crate bit_field;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

// use memory::heap_allocator::BumpAllocator;
use linked_list_allocator::LockedHeap;
#[global_allocator]
// static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START,
//     HEAP_START + HEAP_SIZE);
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();


#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page

    print_name();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    enable_nxe_bit();
    enable_write_protect_bit();
    
    // set up guard page and map the heap pages
    let mut memory_controller = memory::init(boot_info);

    // initialize the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1,2,3,4,5,6,7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    // for i in 0..10000 {
    //     format!("Some String");
    // }
    // use memory::FrameAllocator;
    // for i in 0.. {
    //     use memory::FrameAllocator;
    //     // println!("{:?}", frame_allocator.allocate_frame());
    //     if let None = frame_allocator.allocate_frame() {
    //         println!("allocated {} frames", i);
    //         break;
    //     }
    // }
    
    // initialize our IDT
    interrupts::init(&mut memory_controller);
    // interrupts::set_keyboard_fn(vga_buffer::WRITER.);

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }
    // trigger a stack overflow
    stack_overflow();

    println!("It did not crash!");

    loop{}
}

fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

fn enable_nxe_bit() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
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