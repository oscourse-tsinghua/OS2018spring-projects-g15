use spin::{Once, Mutex};

use self::process::*;
use self::processor::*;

mod process;
mod processor;
mod stack;

/// 平台相关依赖：struct TrapFrame
///
/// ## 必须实现的特性
///
/// * Debug: 用于Debug输出
use arch::interrupts::TrapFrame;

pub fn init() {
    PROCESSOR.call_once(|| {Mutex::new({
        let mut processor = Processor::new();
        debug!("after processor new");
        let initproc = Process::new_init();
        debug!("after new init");
        //let idleproc = Process::new("idle", idle_thread);
        processor.add(initproc);
        //processor.add(idleproc);
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
    let a=vec![0,1,2,233,4];
    for i in 0..5{
        println!("a[{}]={}",i,a[i]);
    }
    use arch::syscall;
    syscall::fork();
    println!("idle: finish fork!");
    loop {
        println!("idle ...");
        let mut i = 0;
        while i < 1 << 23 {
            i += 1;
        }
    }
}

