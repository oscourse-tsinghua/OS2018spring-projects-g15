use x86_64::structures::idt::{Idt, ExceptionStackFrame};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;
use x86_64::instructions::port::{inb, outb};
use memory::MemoryController;
use spin::Once;

mod gdt;

const DOUBLE_FAULT_IST_INDEX: usize = 0;
const IRQ_TIMER: usize = 0;

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.interrupts[IRQ_TIMER].set_handler_fn(timer_handler);
        // idt.interrupts[1].set_handler_fn(keyboard_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut ExceptionStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// our new double fault handler
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame, _error_code: u64)
{
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

static mut TICKS: usize = 0;
extern "x86-interrupt" fn timer_handler(
    _stack_frame: &mut ExceptionStackFrame)
{
    unsafe {
        TICKS = TICKS + 1;
        // if TICKS % 100 == 0 {
            println!("100 ticks");
        // }
    }
}

pub fn init(memory_controller: &mut MemoryController) {
    use x86_64::structures::gdt::SegmentSelector;
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    let double_fault_stack = memory_controller.alloc_stack(1)
        .expect("could not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = VirtualAddress(
            double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();
    unsafe {
        // reload code segment register
        set_cs(code_selector);
        // load TSS
        load_tss(tss_selector);
    }

    IDT.load();
}


const PIC1_CMD_PORT: u16 = 0x20;
const PIC1_DATA_PORT: u16 = 0x21;
const PIC2_CMD_PORT: u16 = 0xA0;
const PIC2_DATA_PORT: u16 = 0xA1;

const PIC1_OFFSET: u8 = 0x20;
const PIC2_OFFSET: u8 = 0x28;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;

pub static KEYS: &'static [u8] = b"\
\x00\x1B1234567890-=\x08\
\tqwertyuiop[]\n\
\x00asdfghjkl;'`\
\x00\\zxcvbnm,./\x00\
*\x00 ";

extern {
    fn keyboard_handler();
}

// pub fn set_keyboard_fn(keyboard_function: fn(u8)) {
//     unsafe {
//         keyboard_fn = keyboard_function;
//     }
// }

// pub fn keypress_main() {
//     unsafe {
//         outb(PIC1_CMD_PORT, PIC1_OFFSET);
        
//         let status: u8 = inb(KEYBOARD_STATUS_PORT);
        
//         if (status & 0x01) != 0 {
//             let keycode: u8 = inb(KEYBOARD_DATA_PORT);
//             if keycode < 0 as u8 {
//                 return;
//             }
//             match KEYS.get(keycode as usize) {
//                 Some(c) => keyboard_fn(*c),
//                 None => {}
//             }
//         }
//     }
// }