use alloc::BTreeMap;
use core::cell::RefCell;
use arch::paging::{ActivePageTable,InactivePageTable};
use super::*;

#[derive(Debug)]
pub struct Processor {
    active_table: RefCell<ActivePageTable>,
    procs: BTreeMap<Pid, Process>,
    current_pid: Pid,
}

impl Processor {
    pub fn new(act: ActivePageTable) -> Self {
        Processor {
            active_table: RefCell::new(act),
            procs: BTreeMap::<Pid, Process>::new(),
            current_pid: 0,
        }
    }

    fn alloc_pid(&self) -> Pid {
        let mut next: Pid = 0;
        for &i in self.procs.keys() {
            if i != next {
                return next;
            } else {
                next = i + 1;
            }
        }
        return next;
    }

    pub fn add(&mut self, mut process: Process) {
        let pid = self.alloc_pid();
        ////deug!("finish add");
        process.pid = pid;
        ////deug!("finish add");
        ////deug!("add:{}",pid);
        self.procs.insert(pid, process);
        ////deug!("finish add");
    }

    pub fn schedule(&mut self, rsp: &mut usize) {
        let pid = self.find_next();
        self.switch_to(pid, rsp);
    }

    fn find_next(&self) -> Pid {
        *self.procs.keys()
            .find(|&&i| i > self.current_pid)
            .unwrap_or(self.procs.keys().nth(0).unwrap())
    }

    fn switch_to(&mut self, pid: Pid, rsp: &mut usize) {
        // for debug print
        let pid0 = self.current_pid;
        //deug!("switch to:{}->{}",pid0,pid);
        let rsp0 = *rsp;

        let curr_rsp: usize;
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
        }
        //deug!("currsp={:#x}",curr_rsp);

        if pid == self.current_pid {
            return;
        }
        {
            let current = self.procs.get_mut(&self.current_pid).unwrap();
            current.status = Status::Ready;
            current.rsp = *rsp;
        }
        {

            let process = self.procs.get_mut(&pid).unwrap();
            process.status = Status::Running;
            use arch::interrupts::TrapFrame;

            let srsp=*rsp as usize;
            let tf = unsafe{ &mut *(srsp as *mut TrapFrame) };
            let stf = unsafe{ &mut *(process.rsp as *mut TrapFrame) };
            // if (pid==0){
            //     stf.ss=tf.ss;
            // }
            
            //stf.ss=0x20;
            //deug!("tf_addr={:#x} stf_addr={:#x}",srsp,process.rsp);
            //deug!("tf.rip={:#x} stf.rip={:#x}",tf.rip,stf.rip);
            //deug!("tf.cs={:#x} stf.cs={:#x}",tf.cs,stf.cs);
            //deug!("tf.rflags={:#x} stf.rflags={:#x}",tf.rflags,stf.rflags);
            //deug!("tf.rsp={:#x} stf.rsp={:#x}",tf.rsp,stf.rsp);
            //deug!("tf.ss={:#x} stf.ss={:#x}",tf.ss,stf.ss);
            //if pid0!=0 && pid!=0 {
                //stf.rip=tf.rip as usize;
            //}
            *rsp = process.rsp;
            /*if pid0==0 || pid==0 {
                *rsp = process.rsp;
            }else{
                //tf.rsp=stf.rsp;
                let srip=tf.rip;
                *tf=(*stf).clone();
                tf.rip=srip;
            }*/
            // *rsp = process.rsp + size_of::<TrapFrame>();
            // TODO switch page table
            /*let srsp=*rsp as usize;
            let tf = unsafe{ &mut *(srsp as *mut TrapFrame) };
            let stf = unsafe{ &*(process.rsp as *mut TrapFrame) };
            //deug!("proc.rsp={:#x} tf.rsp={:#x} stf.rsp={:#x}",process.rsp,tf.rsp,stf.rsp);
            //deug!("tf.rip={:#x} stf.rip={:#x}",tf.rip,stf.rip);
            //deug!("tf.rflags={:#x} stf.rflags={:#x}",tf.rflags,stf.rflags);
            //deug!("tf.cs={:#x} stf.cs={:#x}",tf.cs,stf.cs);
            //deug!("tf.ss={:#x} stf.ss={:#x}",tf.ss,stf.ss);*/
            //*tf=(*stf).clone();
            // tf.rsp=stf.rsp;
            // tf.rip=stf.rip;
            // tf.rflags=stf.rflags;
            // tf.cs=stf.cs;
           // tf.ss=stf.ss;
        }
        self.current_pid = pid;
        //deug!("Processor: switch from {} to {}\n  rsp: {:#x} -> {:#x}", pid0, pid, rsp0, rsp);
    }

    /// Fork the current process
    pub fn fork(&mut self, tf: &TrapFrame) {/*
        let mut curr_rsp: usize;
        let mut curr_rip: usize;
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
            asm!("" : "={rip}"(curr_rip) : : : "intel", "volatile");
        }
        //deug!("porser currsp={:#x} currip={:#x} tf.rsp={:#x}",curr_rsp,curr_rip,tf.rsp);
        let new2 = self.procs.get_mut(&self.current_pid).unwrap();//.fork(tf);
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
            asm!("" : "={rip}"(curr_rip) : : : "intel", "volatile");
        }
        //deug!("porser2 currsp={:#x} currip={:#x} tf.rsp={:#x}",curr_rsp,curr_rip,tf.rsp);
        let new = new2.fork(tf);*/
        let new = self.procs.get_mut(&self.current_pid).unwrap().fork(tf,&mut self.active_table.borrow_mut());
        self.add(new);
        //deug!("rip={:#x}",tf.rip);
        //deug!("finish fork");
    }
}