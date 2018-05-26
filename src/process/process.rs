use super::*;
use xmas_elf::{ElfFile, program::{Flags, ProgramHeader}, header::HeaderPt2};
use core::slice;
use alloc::boxed::Box;
use alloc::string::ToString;
use arch::interrupts::TrapFrame;
use arch::paging::{ActivePageTable,InactivePageTable,EntryFlags};
use consts::{USER_STACK_OFFSET, USER_STACK_SIZE};
use rlibc::memcpy;
use memory;
use memory::Stack;
use memory::memory_set::{MemoryArea,MemorySet};
use memory::PAddr;
use memory::address::FromToVirtualAddress;

#[derive(Debug)]
pub struct Process {
    pub(in process) pid: Pid,
                    name: &'static str,
                    kstack: Stack,
                    //kstack: Box<[u8]>,
    //    page_table: Box<PageTable>,
    pub(in process) memory_set: Option<MemorySet>,
    pub(in process) page_table: Option<InactivePageTable>,
    pub(in process) status: Status,
    pub(in process) rsp: usize,
    pub(in process) is_user: bool,
}

pub type Pid = usize;

#[derive(Debug)]
pub enum Status {
    Ready, Running, Sleeping(usize), Exited
}

impl Process {
    /// Make a new kernel thread
    pub fn new(name: &'static str, entry: extern fn()) -> Self {
        //deug!("new proc");
        let error_log = "cannot alloc stack of proc {}".to_string() + name;
        // let kstack = Stack::new().expect(&error_log);
        // let rsp = unsafe{ (kstack.top().0 as *mut TrapFrame).offset(-1) } as usize;
        //let kstack = Box::new([0u8; 1<<12]);
        let kstack = memory::alloc_stacks(7).unwrap();
        let tf = TrapFrame::new_kernel_thread(entry, kstack.top());
        let rsp = kstack.push_at_top(tf);/*
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        //deug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        //let rsp=0xffffff000010cea0;
        //let rsp=0xffffe80000003d60;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };

        let tf = unsafe{ &mut *(rsp as *mut TrapFrame) };
        // *tf = TrapFrame::new_kernel_thread(entry, kstack.top().0 as usize);
        *tf = TrapFrame::new_kernel_thread(entry, stack_top);*/
        Process {
            pid: 0,
            name,
            kstack,
            memory_set: None,
            page_table: None,
            status: Status::Ready,
            rsp,
            is_user: false,
        }
    }
    /// Make the first kernel thread `initproc`
    /// Should be called only once
    pub fn new_init() -> Self {
        let kstack=memory::alloc_stacks(7).unwrap();
        //deug!("stack bottom: {:#x}, stack top: {:#x}", kstack.bottom(), kstack.top());
        /*
        //deug!("new_init proc");
        assert_has_not_been_called!();
        let kstack = Box::new([0u8; 1<<12]);
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        //deug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        // let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };*/
        Process {
            pid: 0,
            name: "init",
            // kstack: Stack::new().expect("cannot alloc stack in initproc!"),
            kstack: kstack,
            memory_set: None,
            page_table: None,
            status: Status::Running,
            rsp: 0, // will be set at first schedule
            is_user: false,
        }
    }

    /// Make a new user thread
    /// The program elf data is placed at [begin, end)
    pub fn new_user(begin: usize, end: usize, act: &mut ActivePageTable) -> Self {
        //deug!("new user\nbegin={:#x} end={:#x}",begin,end);
        // Parse elf
        let slice = unsafe{ slice::from_raw_parts(begin as *const u8, end - begin) };
        let elf = ElfFile::new(slice).expect("failed to read elf");

        // Make page table
        use consts::{USER_STACK_OFFSET, USER_STACK_SIZE};
        let mut memory_set = MemorySet::from(&elf);
        memory_set.push(MemoryArea::new(USER_STACK_OFFSET, USER_STACK_OFFSET + USER_STACK_SIZE,
                                        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE | EntryFlags::USER_ACCESSIBLE, "user_stack"));
        let page_table = memory::make_page_table(&memory_set,act);
        //deug!("{:#x?}", memory_set);
        //deug!("{:?}", page_table);

        // Temporary switch to it, in order to copy data
        let backup = act.switch(page_table);
        //deug!("switch page_table over");
        for ph in elf.program_iter() {
            let ph = match ph {
                ProgramHeader::Ph64(ph) => ph,
                _ => unimplemented!(),
            };
            unsafe { memcpy(ph.virtual_addr as *mut u8, (begin + ph.offset as usize) as *mut u8, ph.file_size as usize) };
        }
        let page_table=act.switch(backup);
        //deug!("switch backup over");

        let entry_addr = match elf.header.pt2 {
            HeaderPt2::Header64(header) => header.entry_point,
            _ => {
                //deug!("elf header fail");
                unimplemented!();
            },
        } as usize;
        //deug!("entry_addr:{:#x}",entry_addr);

        // Allocate kernel stack and push trap frame
        let kstack = memory::alloc_stacks(7).unwrap();
        let tf = TrapFrame::new_user_thread(entry_addr, USER_STACK_OFFSET + USER_STACK_SIZE);
        //deug!("begin:{:#x}, end: {:#x}",begin,end);
        //deug!("entry_addr:{:#x}, rsp: {:#x}, rip: {:#x}",entry_addr, tf.rsp, tf.rip);
        let rsp = kstack.push_at_top(tf);

        Process {
            pid: 0,
            name: "user",
            kstack,
            memory_set: Some(memory_set),
            page_table: Some(page_table),
            status: Status::Ready,
            rsp,
            is_user: true,
        }
    }


    /// Fork
    pub fn fork(&self, stf: &TrapFrame, act: &mut ActivePageTable) -> Self {
        ////deug!("fork:{}",self.pid);
        let curr_rsp: usize;
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
        }
        //deug!("currsp={:#x} stf.rsp={:#x}",curr_rsp,stf.rsp);
        let kstack = memory::alloc_stacks(7).unwrap();
        //deug!("stack bottom: {:#x}, stack top: {:#x}", kstack.bottom(), kstack.top());
        let mut tf = stf.clone();
        // let tf2 = TrapFrame::new_user_thread(tf.rip, USER_STACK_OFFSET + USER_STACK_SIZE);
        // use core::mem::size_of;
        // //tf.rsp=kstack.top()-size_of::<TrapFrame>();
        // tf.rsp=tf2.rsp;
        // tf.cs=tf2.cs;
        // tf.ss=tf2.ss;
        // tf.rflags=tf2.rflags;
        let rsp = kstack.push_at_top(tf);
        //kstack.push_at_top(tf.rsp);
        //deug!("rsp={:#x}",rsp);
        // tf.rsp=rsp;
        /*
        let kstack = Box::new([0u8; 1<<12]);
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        //deug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        //let rsp=0xffffff000010cea0;
        //let rsp=0xffffe80000003d60;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };

        let tf = unsafe{ &mut *(rsp as *mut TrapFrame) };
        // *tf = TrapFrame::new_kernel_thread(entry, kstack.top().0 as usize);
        *tf = (*stf).clone();
        tf.rsp=stack_top;*/
        ////deug!("rsp={:#x} rip={:#x} cs={:#x} ss={:#x}",tf.rsp,tf.rip,tf.cs,tf.ss);
        //deug!("finish fork");
        Process {
            pid: 0,
            name: "fork",
            kstack,
            memory_set: None,
            page_table: None,
            status: Status::Ready,
            rsp,
            is_user: true,
        }
    }
}

impl<'a> From<&'a ElfFile<'a>> for MemorySet {
    fn from(elf: &'a ElfFile<'a>) -> Self {
        let mut set = MemorySet::new();
        for ph in elf.program_iter() {
            let ph = match ph {
                ProgramHeader::Ph64(ph) => ph,
                _ => unimplemented!(),
            };
            set.push(MemoryArea::new(
                ph.virtual_addr as usize,
                (ph.virtual_addr + ph.mem_size) as usize,
                EntryFlags::from(ph.flags),
                ""));
        }
        set
    }
}

impl From<Flags> for EntryFlags {
    fn from(elf_flags: Flags) -> Self {
        let mut flags = EntryFlags::PRESENT | EntryFlags::USER_ACCESSIBLE;
        if elf_flags.is_write() {
            flags = flags | EntryFlags::WRITABLE;
        }
        if !elf_flags.is_execute() {
            flags = flags | EntryFlags::NO_EXECUTE;
        }
        flags
    }
}