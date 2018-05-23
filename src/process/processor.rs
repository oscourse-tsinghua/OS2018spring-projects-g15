use alloc::BTreeMap;
use super::*;

#[derive(Debug)]
pub struct Processor {
    procs: BTreeMap<Pid, Process>,
    current_pid: Pid,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
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
        //debug!("finish add");
        process.pid = pid;
        //debug!("finish add");
        //debug!("add:{}",pid);
        self.procs.insert(pid, process);
        //debug!("finish add");
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
        debug!("switch to:{}->{}",pid0,pid);
        let rsp0 = *rsp;

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
            debug!("proc.rsp={:#x} tf.rsp={:#x} stf.rsp={:#x}",process.rsp,tf.rsp,stf.rsp);
            debug!("tf.rip={:#x} stf.rip={:#x}",tf.rip,stf.rip);
            debug!("tf.rflags={:#x} stf.rflags={:#x}",tf.rflags,stf.rflags);
            debug!("tf.cs={:#x} stf.cs={:#x}",tf.cs,stf.cs);
            debug!("tf.ss={:#x} stf.ss={:#x}",tf.ss,stf.ss);
            //if pid0!=0 && pid!=0 {
                stf.rip=tf.rip;
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
            debug!("proc.rsp={:#x} tf.rsp={:#x} stf.rsp={:#x}",process.rsp,tf.rsp,stf.rsp);
            debug!("tf.rip={:#x} stf.rip={:#x}",tf.rip,stf.rip);
            debug!("tf.rflags={:#x} stf.rflags={:#x}",tf.rflags,stf.rflags);
            debug!("tf.cs={:#x} stf.cs={:#x}",tf.cs,stf.cs);
            debug!("tf.ss={:#x} stf.ss={:#x}",tf.ss,stf.ss);*/
            //*tf=(*stf).clone();
            // tf.rsp=stf.rsp;
            // tf.rip=stf.rip;
            // tf.rflags=stf.rflags;
            // tf.cs=stf.cs;
           // tf.ss=stf.ss;
        }
        self.current_pid = pid;
        debug!("Processor: switch from {} to {}\n  rsp: {:#x} -> {:#x}", pid0, pid, rsp0, rsp);
    }

    /// Fork the current process
    pub fn fork(&mut self, tf: &TrapFrame) {
        let mut curr_rsp: usize;
        let mut curr_rip: usize;
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
            asm!("" : "={rip}"(curr_rip) : : : "intel", "volatile");
        }
        debug!("porser currsp={:#x} currip={:#x} tf.rsp={:#x}",curr_rsp,curr_rip,tf.rsp);
        let new2 = self.procs.get_mut(&self.current_pid).unwrap();//.fork(tf);
        unsafe{
            asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
            asm!("" : "={rip}"(curr_rip) : : : "intel", "volatile");
        }
        debug!("porser2 currsp={:#x} currip={:#x} tf.rsp={:#x}",curr_rsp,curr_rip,tf.rsp);
        let new = new2.fork(tf);
        //self.add(new);
        debug!("rip={:#x}",tf.rip);
        debug!("finish fork");
    }
}