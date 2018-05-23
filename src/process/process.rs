use super::*;
use super::stack::Stack;
use alloc::boxed::Box;
use alloc::string::ToString;
use arch::interrupts::TrapFrame;
use memory::memory_set::{MemoryArea,MemorySet};
use arch::paging::InactivePageTable;

#[derive(Debug)]
pub struct Process {
    pub(in process) pid: Pid,
                    name: &'static str,
                    // kstack: Stack,
                    kstack: Box<[u8]>,
    //    page_table: Box<PageTable>,
    pub(in process) memory_set: Option<MemorySet>,
    pub(in process) page_table: Option<InactivePageTable>,
    pub(in process) status: Status,
    pub(in process) rsp: usize,
}

pub type Pid = usize;

#[derive(Debug)]
pub enum Status {
    Ready, Running, Sleeping(usize), Exited
}

impl Process {
    /// Make a new kernel thread
    pub fn new(name: &'static str, entry: extern fn()) -> Self {
        debug!("new proc");
        let error_log = "cannot alloc stack of proc {}".to_string() + name;
        // let kstack = Stack::new().expect(&error_log);
        // let rsp = unsafe{ (kstack.top().0 as *mut TrapFrame).offset(-1) } as usize;
        let kstack = Box::new([0u8; 1<<12]);
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        debug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        //let rsp=0xffffff000010cea0;
        //let rsp=0xffffe80000003d60;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };

        let tf = unsafe{ &mut *(rsp as *mut TrapFrame) };
        // *tf = TrapFrame::new_kernel_thread(entry, kstack.top().0 as usize);
        *tf = TrapFrame::new_kernel_thread(entry, stack_top);
        Process {
            pid: 0,
            name,
            kstack,
            memory_set: None,
            page_table: None,
            status: Status::Ready,
            rsp,
        }
    }
    /// Make the first kernel thread `initproc`
    /// Should be called only once
    pub fn new_init() -> Self {
        debug!("new_init proc");
        assert_has_not_been_called!();
        let kstack = Box::new([0u8; 1<<12]);
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        debug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        // let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };
        Process {
            pid: 0,
            name: "init",
            // kstack: Stack::new().expect("cannot alloc stack in initproc!"),
            kstack: kstack,
            memory_set: None,
            page_table: None,
            status: Status::Running,
            rsp: 0, // will be set at first schedule
        }
    }

    /// Fork
    pub fn fork(&self, stf: &TrapFrame) -> Self {
        //debug!("fork:{}",self.pid);
        let curr_rsp: usize;
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
        }
        debug!("currsp={:#x} stf.rsp={:#x}",curr_rsp,stf.rsp);
        let kstack = Box::new([0u8; 1<<12]);
        let stack_bottom = Box::into_raw(kstack);
        let stack_top = (1<<12) + stack_bottom as usize;
        debug!("stack bottom: {:#x}, stack top: {:#x}", stack_bottom as usize, stack_top);
        let rsp = unsafe{ (stack_top as *mut TrapFrame).offset(-1) } as usize;
        //let rsp=0xffffff000010cea0;
        //let rsp=0xffffe80000003d60;
        let kstack = unsafe{ Box::from_raw(stack_bottom) };

        let tf = unsafe{ &mut *(rsp as *mut TrapFrame) };
        // *tf = TrapFrame::new_kernel_thread(entry, kstack.top().0 as usize);
        *tf = (*stf).clone();
        tf.rsp=stack_top;
        debug!("rsp={:#x} rip={:#x} cs={:#x} ss={:#x}",tf.rsp,tf.rip,tf.cs,tf.ss);
        debug!("finish fork");
        Process {
            pid: 0,
            name: "fork",
            kstack,
            memory_set: None,
            page_table: None,
            status: Status::Ready,
            rsp,
        }
    }
}