use x86_64::structures::idt::{Idt, HandlerFunc};
use x86_64::PrivilegeLevel;

// use memory::MemoryController;
use arch::gdt;
use consts::irq::*;
use super::interrupts::irq::*;
use self::gdt::DOUBLE_FAULT_IST_INDEX;

use modules::ps2;

pub trait SetHandler {
    fn set_idt_handler(&mut self, idx: usize, handler_fn: HandlerFunc);
}

impl SetHandler for Idt {
    fn set_idt_handler(&mut self, idx: usize, handler_fn: HandlerFunc) {
        self.interrupts[idx].set_handler_fn(handler_fn);
    }
}

lazy_static! {
    pub static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.interrupts[IRQ_TIMER as usize].set_handler_fn(timer_handler);
        // idt.interrupts[IRQ_KBD as usize].set_handler_fn(keyboard_handler);
        idt.interrupts[IRQ_KBD as usize].set_handler_fn(ps2::handle_irq_kbd);
        idt.interrupts[IRQ_MOUSE as usize].set_handler_fn(ps2::handle_irq_mouse);
        idt.interrupts[IRQ_COM1 as usize].set_handler_fn(com1_handler);
        idt.interrupts[IRQ_COM2 as usize].set_handler_fn(com2_handler);
        idt.interrupts[T_SYSCALL as usize].set_handler_fn(syscall_handler);
        idt.interrupts[T_SWITCH_TOU as usize].set_handler_fn(to_user_handler)
            .set_present(true).set_privilege_level(PrivilegeLevel::from_u16(3));
        
        idt.interrupts[T_SWITCH_TOK as usize].set_handler_fn(to_kernel_handler)
            .set_present(true).set_privilege_level(PrivilegeLevel::from_u16(3));

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