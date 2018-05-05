#![feature(lang_items)]
#![feature(unique, const_unique_new, const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(abi_x86_interrupt)]
#![feature(iterator_step_by)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(unboxed_closures)]
#![feature(match_default_bindings)]
#![no_std]

#[macro_use]    // test!
mod test_utils;
#[macro_use]
mod io;
#[macro_use]
mod macros;
mod memory;
// mod modules;
mod lang;
mod utils;
mod consts;
mod time;
pub mod allocator;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate once;
#[macro_use]
extern crate lazy_static;

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;
extern crate bit_field;

extern crate syscall;
extern crate raw_cpuid;
extern crate slab_allocator;

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

use lang::{print_name, eh_personality, panic_fmt};

#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page

    arch::cpu::init();
    print_name();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };

    // println!("MP = {:?}", arch::driver::mp::find_mp());
    // println!("RDSP = {:?}", arch::driver::acpi::find_rsdp());

    // set up guard page and map the heap pages
    let mut active_table = memory::init(&boot_info);
    unsafe{
        allocator::init(&mut active_table);
    }

    // initialize our IDT and GDT
    arch::idt::init();

    test!(global_allocator);
    test!(alloc_sth);
    // test!(find_mp);
    // test!(guard_page);

    // arch::driver::init(&mut active_table,
    //     |addr: usize| map_page_identity(addr));
    unsafe{
        use arch::driver::{pic, apic, acpi};
        pic::init();
        apic::local_apic::init(&mut active_table);
        acpi::init(&mut active_table);
    }
    // memory_controller.print_page_table();
    // arch::smp::start_other_cores(&acpi, &mut memory_controller);

    unsafe{ arch::interrupts::enable(); }

    println!("It did not crash!");

    loop{}
    test_end!();
}

#[no_mangle]
pub extern "C" fn other_main() -> ! {
    arch::cpu::init();
    // arch::idt::init(&mut memory_controller);
    
    arch::driver::apic::other_init();
    let cpu_id = arch::driver::apic::lapic_id();
    println!("Hello world! from CPU {}!", arch::driver::apic::lapic_id());
    // unsafe{ arch::smp::notify_started(cpu_id); }
    unsafe{ let a = *(0xdeadbeaf as *const u8); } // Page fault
    loop {}
}

mod test {
    pub fn extern_func() {
        extern {
            fn foo(x: i32) -> i32;
        }

        println!("extern fn foo(2): {}", unsafe{foo(2)});
    }
    pub fn global_allocator() {
        debug!("in global allocator");
        for i in 0..10000 {
            format!("Some String");
        }
        debug!("fin global alloc test");
    }

    // pub fn find_mp() {
    //     use arch;
    //     let mp = arch::driver::mp::find_mp();
    //     assert!(mp.is_some());
    // }

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
