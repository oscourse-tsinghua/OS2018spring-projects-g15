use x86_64::structures::idt::Idt;

// use memory::MemoryController;
use arch::gdt;
use consts::irq::*;
use super::interrupts::irq::*;
use self::gdt::DOUBLE_FAULT_IST_INDEX;

lazy_static! {
    pub static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.interrupts[IRQ_TIMER as usize].set_handler_fn(timer_handler);
        idt.interrupts[IRQ_KBD as usize].set_handler_fn(keyboard_handler);
        idt.interrupts[IRQ_COM1 as usize].set_handler_fn(com1_handler);
        idt.interrupts[IRQ_COM2 as usize].set_handler_fn(com2_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

pub fn init() {
    gdt::init();
    IDT.load();
}