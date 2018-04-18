// Rust language features implementions

use core;
use arch::cpu;

use vga_buffer;

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}

#[lang = "panic_fmt"] #[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\n !! KERNEL PANIC !!");
    println!("{} at line {}:", file, line);
    println!("    {}", fmt);
    if cfg!(feature = "qemu_auto_exit") {
        unsafe{ cpu::exit_in_qemu(3) }
    } else {
        loop{}
    }
}

pub fn print_name() {
    vga_buffer::clear_screen();
    println!(" _______     __     __    _______    ______     _______     ________  ");
    println!("|  ____  \\  |  |   |  |  /  _____|  / _____ \\  |  _____ \\  |  ______| ");
    println!("| |____  |  |  |   |  | |  |       | |     | | | |_____ |  | |______  ");
    println!("|  ___  _/  |  |   |  | |  |       | |     | | |  ___  _/  |  ______| ");
    println!("| |   \\ \\   |  |   |  | |  |       | |     | | | |   \\ \\   | |        ");
    println!("| |    \\ \\  |  \\___/  | |  |_____  | |_____| | | |    \\ \\  | |______  ");
    println!("|_/     \\_\\  \\_______/   \\_______|  \\_______/  |_/     \\_\\ |________| ");
}