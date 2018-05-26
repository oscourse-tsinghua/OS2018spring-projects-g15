use spin::{Once, Mutex};

use self::process::*;
use self::processor::*;
use arch::paging::{ActivePageTable,InactivePageTable};
use vfs;

mod process;
mod processor;
mod stack;

/// 平台相关依赖：struct TrapFrame
///
/// ## 必须实现的特性
///
/// * Debug: 用于Debug输出
use arch::interrupts::TrapFrame;

#[cfg(feature = "link_user_program")]
extern {
    fn _binary_user_forktest_start();
    fn _binary_user_forktest_end();
}

pub fn init(mut act:ActivePageTable) {
    PROCESSOR.call_once(|| {Mutex::new({
        let initproc = Process::new_init();
        debug!("after new init");
        let idleproc = Process::new("idle", idle_thread);
        #[cfg(feature = "link_user_program")]
        //let forktest = Process::new_user(_binary_user_forktest_start as usize,
        //                                 _binary_user_forktest_end as usize,&mut act);
        let mut processor = Processor::new(act);
        debug!("after processor new");
        processor.add(initproc);
        processor.add(idleproc);
        // processor.add(forktest);
        processor
    })});
}

static PROCESSOR: Once<Mutex<Processor>> = Once::new();

/// Called by timer handler in arch
/// 设置rsp，指向接下来要执行线程的 内核栈顶
/// 之后中断处理例程会重置rsp，恢复对应线程的上下文
pub fn schedule(rsp: &mut usize) {
    debug!("schedule rsp={:#x}",rsp);
    PROCESSOR.try().unwrap().lock().schedule(rsp);
}

/// Fork the current process
pub fn fork(tf: &TrapFrame) {
    let curr_rsp: usize;
    unsafe{
        asm!("" : "={rsp}"(curr_rsp) : : : "intel", "volatile");
    }
    debug!("currsp={:#x} tf.rsp={:#x}",curr_rsp,tf.rsp);
    PROCESSOR.try().unwrap().lock().fork(tf);
}

extern fn idle_thread() {
    println!("I'm idle");
    // let a=vec![0,1,2,233,4];
    // for i in 0..5{
    //     println!("a[{}]={}",i,a[i]);
    // }
    // use arch::syscall;
    // syscall::fork();
    // println!("idle: finish fork!");
    loop {
        println!("idle ...");
/*
		let mut dst: [u32;10]=[0;10];
		vfs::readFile("/system/1.TXT",&mut dst);
		for j in 0..10{
            println!("dst[{}]={}",j,dst[j]);
		}
*/
        let mut i = 0;
        while i < 1 << 23 {
            i += 1;
        }
    }
}

