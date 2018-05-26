use super::trapframe::TrapFrame;

fn breakpoint() {
    debug!("\nEXCEPTION: Breakpoint");
}

fn double_fault() {
    debug!("\nEXCEPTION: Double Fault");
    loop {}
}

fn page_fault(tf: &mut TrapFrame) {
    use x86_64::registers::control_regs::cr2;
    let addr = cr2().0;
    debug!("\nEXCEPTION: Page Fault @ {:#x}, code: {:#x}", addr, tf.error_code);

    use memory::page_fault_handler;
    if page_fault_handler(addr) {
        return;
    }

    loop {}
}

fn general_protection_fault() {
    debug!("\nEXCEPTION: General Protection Fault");
    loop {}
}

fn invalid_opcode() {
    debug!("\nEXCEPTION: Invalid Opcode");
    loop {}
}

#[cfg(feature = "use_apic")]
use arch::driver::apic::ack;
#[cfg(not(feature = "use_apic"))]
use arch::driver::pic::ack;

use consts::irq::*;

fn keyboard() {
    // use arch::driver::keyboard;
    // debug!("\nInterupt: Keyboard");
    // let c = keyboard::get();
    // debug!("Key = '{}' {}", c as u8 as char, c);
    use modules::ps2::handle_irq_kbd;
    handle_irq_kbd();
}

fn mouse() {
    use modules::ps2::handle_irq_mouse;
    handle_irq_mouse();
}

fn com1() {
    use arch::driver::serial::COM1;
    debug!("\nInterupt: COM1");
    COM1.lock().receive();
}

fn com2() {
    use arch::driver::serial::COM2;
    debug!("\nInterupt: COM2");
    COM2.lock().receive();
}

fn timer(tf: &mut TrapFrame, rsp: &mut usize) {
    static mut tick: usize = 0;
    unsafe {
        tick += 1;
        if tick % 100 == 0 {
            debug!("tick 100");
            // use process;
            // process::schedule(rsp);
        }
    }
}

fn fork(tf: &mut TrapFrame) {
    use process;
    unsafe {
        let curr_rsp: usize;
        asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
        debug!("currsp={:#x} tf.rsp={:#x}",curr_rsp,tf.rsp);
        process::fork(tf);
    }
}

fn to_user(rsp: &mut usize, tf: &mut TrapFrame) {
    use arch::gdt;
    debug!("\nInterupt: To User");
    println!("{:?}", tf);
    println!("now rsp: {:#x}, ucs:{:#x}, uds: {:#x}, kcs: {:#x}, kds: {:#x}", *rsp as usize, gdt::UCODE_SELECTOR.0, gdt::UDATA_SELECTOR.0, gdt::KCODE_SELECTOR.0, gdt::KDATA_SELECTOR.0);
    tf.cs = gdt::UCODE_SELECTOR.0 as usize;
    tf.ss = gdt::UDATA_SELECTOR.0 as usize;
    tf.rflags |= 3 << 12;   // 设置EFLAG的I/O特权位，使得在用户态可使用in/out指令
}

fn to_kernel(tf: &mut TrapFrame) {
    use arch::gdt;
    debug!("\nInterupt: To Kernel");
    println!("{:?}", tf);    
    tf.cs = gdt::KCODE_SELECTOR.0 as usize;
    tf.ss = gdt::KDATA_SELECTOR.0 as usize;
}

#[no_mangle]
pub extern fn rust_trap(tf: &mut TrapFrame) -> usize {
    unsafe{ super::disable(); }
    let mut rsp = tf as *const _ as usize;

    // Dispatch
    match tf.trap_num as u8 {
        T_BRKPT => breakpoint(),
        T_DBLFLT => double_fault(),
        T_PGFLT => page_fault(tf),
        T_GPFLT => general_protection_fault(),
        T_IRQ0...64 => {
            let irq = tf.trap_num as u8 - T_IRQ0;
            match irq {
                IRQ_TIMER => timer(tf, &mut rsp),
                IRQ_KBD => keyboard(),
                IRQ_MOUSE => mouse(),
                IRQ_COM1 => com1(),
                IRQ_COM2 => com2(),
                _ => panic!("Invalid IRQ number."),
            }
            ack(irq);
        }
        T_SWITCH_TOK => to_kernel(tf),
        T_SWITCH_TOU => to_user(&mut rsp, tf),
        // T_SYSCALL => syscall(tf, &mut rsp),
        // 0x80 => syscall32(tf, &mut rsp),
        _ => panic!("Unhandled interrupt {:x}", tf.trap_num),
    }

    // Set return rsp if to user
    let tf = unsafe { &*(rsp as *const TrapFrame) };
    set_return_rsp(tf);
    unsafe{ super::enable(); }
    rsp
}

fn set_return_rsp(tf: &TrapFrame) {
    use core::mem::size_of;
    use arch::gdt;
    if tf.cs & 0x3 == 3 {
        gdt::set_ring0_rsp(tf as *const _ as usize + size_of::<TrapFrame>());
    }
}
