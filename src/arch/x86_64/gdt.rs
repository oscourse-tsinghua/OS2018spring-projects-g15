use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::{PrivilegeLevel, VirtualAddress};
use spin::Once;

use alloc::boxed::Box;

use core::ptr::Unique;
use core::fmt;
use core::fmt::Debug;

// static TSS: Once<TaskStateSegment> = Once::new();
static TSS: Once<Box<TaskStateSegment>> = Once::new();
static GDT: Once<Gdt> = Once::new();

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

// Copied from xv6 x86_64
const KCODE: Descriptor = Descriptor::UserSegment(0x0020980000000000);  // EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE
const UCODE: Descriptor = Descriptor::UserSegment(0x0020F80000000000);  // EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT | LONG_MODE
const KDATA: Descriptor = Descriptor::UserSegment(0x0000920000000000);  // DATA_WRITABLE | USER_SEGMENT | PRESENT
const UDATA: Descriptor = Descriptor::UserSegment(0x0000F20000000000);  // DATA_WRITABLE | USER_SEGMENT | USER_MODE | PRESENT
const UCODE32: Descriptor = Descriptor::UserSegment(0x00cffa00_0000ffff);
// EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT
const UDATA32: Descriptor = Descriptor::UserSegment(0x00cff200_0000ffff);  // EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT

pub const KCODE_SELECTOR: SegmentSelector = SegmentSelector::new(1+1, PrivilegeLevel::Ring0);
pub const UCODE_SELECTOR: SegmentSelector = SegmentSelector::new(1+2, PrivilegeLevel::Ring3);
pub const KDATA_SELECTOR: SegmentSelector = SegmentSelector::new(1+3, PrivilegeLevel::Ring0);
pub const UDATA_SELECTOR: SegmentSelector = SegmentSelector::new(1+4, PrivilegeLevel::Ring3);
pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(1+5, PrivilegeLevel::Ring0);

static mut TSS_PTR: Unique<TaskStateSegment> = unsafe{ Unique::new_unchecked(0 as *mut _) };

/// 设置从Ring3跳到Ring0时，自动切换栈的地址
///
/// 每次进入用户态前，都要调用此函数，才能保证正确返回内核态
pub fn set_ring0_rsp(rsp: usize) {
    debug!("gdt.set_ring0_rsp: {:#x}", rsp);
    unsafe { 
        TSS_PTR.as_mut().privilege_stack_table[0] = VirtualAddress(rsp);
        // debug!("TSS:\n{:?}", TSS_PTR.as_ref());
    }
}

pub struct Gdt {
    table: [u64; 10],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Gdt {
        Gdt {
            table: [0; 10],
            next_free: 1,
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full");
        }
    }

    pub fn load(&'static self) {
        use x86_64::instructions::tables::{DescriptorTablePointer, lgdt};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

bitflags! {
    /// Reference: https://wiki.osdev.org/GDT
    struct DescriptorFlags: u64 {
        const ACCESSED          = 1 << 40;
        const DATA_WRITABLE     = 1 << 41;
        const CODE_READABLE     = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const USER_MODE         = 1 << 45 | 1 << 46;
        const PRESENT           = 1 << 47;
        const LONG_MODE         = 1 << 53;
    }
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        let flags = DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::EXECUTABLE | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use core::mem::size_of;
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;

        let mut low = DescriptorFlags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` in needed since the bound is inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

pub fn init() {
    use x86_64::structures::gdt::SegmentSelector;
    use x86_64::instructions::segmentation::{set_cs, load_ss};
    use x86_64::instructions::tables::load_tss;

    use alloc::boxed::Box;

    let double_fault_stack_top = Box::into_raw(Box::new([0u8; 4096])) as usize + 4096;
    debug!("Double fault stack top @ {:#x}", double_fault_stack_top);
    
    let tss = Box::new({
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = 
            VirtualAddress(double_fault_stack_top);
        tss
    });
    unsafe{ TSS_PTR = Unique::new_unchecked(Box::into_raw(tss)); }
    let tss = unsafe{ TSS_PTR.as_ref() };

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        // code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(KCODE);
        gdt.add_entry(UCODE);
        gdt.add_entry(KDATA);
        gdt.add_entry(UDATA);
        gdt.add_entry(UCODE32);
        gdt.add_entry(UDATA32);
        tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();
    unsafe {
        // reload code segment register
        // set_cs(code_selector);
        set_cs(KCODE_SELECTOR);
        load_ss(KDATA_SELECTOR);
        // load TSS
        load_tss(tss_selector);
    }
}

impl Debug for Descriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Descriptor::UserSegment(flags) => 
                write!(f, "UserSegment( {:?} )", DescriptorFlags{bits: *flags}),
            Descriptor::SystemSegment(low, high) =>
                write!(f, "SystemSegment{:?}", (low, high)),
        }
    }
}


pub mod test
{
    pub fn print_flags() {
        use super::*;
        // The following 4 GDT entries were copied from xv6 x86_64
        let list: [(&str, Descriptor); 4] = [
            ("KCODE", super::KCODE), // Code, DPL=0, R/X
            ("UCODE", super::UCODE), // Code, DPL=3, R/X
            ("KDATA", super::KDATA), // Data, DPL=0, W
            ("UDATA", super::UDATA), // Data, DPL=3, W
        ];
        // Let's see what that means
        println!("GDT Segments from xv6 x86_64:");
        for (name, desc) in list.iter() {
            println!("  {}: {:?}", name, desc);
        }
    }
}