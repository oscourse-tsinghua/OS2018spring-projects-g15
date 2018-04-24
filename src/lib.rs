#![feature(lang_items)]
#![feature(unique, const_unique_new)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(const_atomic_usize_new)]
#![feature(abi_x86_interrupt)]
#![feature(iterator_step_by)]
#![feature(core_intrinsics)]
#![feature(match_default_bindings)]
#![no_std]

#[macro_use]    // test!
mod test_utils;
#[macro_use]
mod io;
mod memory;
// mod interrupts;
mod lang;
mod utils;
mod consts;

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


extern crate syscall;

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

use lang::{print_name, eh_personality, panic_fmt};

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
    arch::init();

    println!("MP = {:?}", arch::driver::mp::find_mp());
    println!("RDSP = {:?}", arch::driver::acpi::find_rsdp());

    arch::driver::init();

    // set up guard page and map the heap pages
    let mut memory_controller = memory::init(boot_info);

    // initialize the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    // initialize our IDT
    arch::idt::init(&mut memory_controller);

    test!(global_allocator);
    test!(alloc_sth);
    // test!(guard_page);

    println!("It did not crash!");

    loop{}
    // test_end!();
}

mod test {
    pub fn extern_func() {
        extern {
            fn foo(x: i32) -> i32;
        }

        println!("extern fn foo(2): {}", unsafe{foo(2)});
    }
    pub fn global_allocator() {
        for i in 0..10000 {
            format!("Some String");
        }
    }

    pub fn alloc_sth() {
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
    }

    pub fn guard_page() {
        use x86_64;
        // invoke a breakpoint exception
        x86_64::instructions::interrupts::int3();

        fn stack_overflow() {
            stack_overflow(); // for each recursion, the return address is pushed
        }

        // trigger a stack overflow
        stack_overflow();
    }
}
