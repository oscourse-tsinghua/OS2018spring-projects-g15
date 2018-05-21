use super::*;
use super::stack::Stack;
use alloc::string::ToString;
use arch::interrupts::TrapFrame;

#[derive(Debug)]
pub struct Process {
    pub(in process) pid: Pid,
                    name: &'static str,
                    kstack: Stack,
    //    page_table: Box<PageTable>,
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
        let error_log = "cannot alloc stack of proc {}".to_string() + name;
        let kstack = Stack::new().expect(&error_log);
        let rsp = unsafe{ (kstack.top().0 as *mut TrapFrame).offset(-1) } as usize;

        let tf = unsafe{ &mut *(rsp as *mut TrapFrame) };
        *tf = TrapFrame::new_kernel_thread(entry, kstack.top().0 as usize);
        Process {
            pid: 0,
            name,
            kstack,
            status: Status::Ready,
            rsp,
        }
    }
    /// Make the first kernel thread `initproc`
    /// Should be called only once
    pub fn new_init() -> Self {
        assert_has_not_been_called!();
        Process {
            pid: 0,
            name: "init",
            kstack: Stack::new().expect("cannot alloc stack in initproc!"),
            status: Status::Running,
            rsp: 0, // will be set at first schedule
        }
    }
}