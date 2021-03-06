
#[derive(Debug, Clone, Default)]
pub struct TrapFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    pub trap_num: usize,
    pub error_code: usize,

    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,

    pub rsp: usize,
    pub ss: usize,
}

impl TrapFrame {
    pub fn new_kernel_thread(code: extern fn(), rsp: usize) -> Self {
        use arch::gdt;
        let mut tf = TrapFrame::default();
        println!("KCODE_SELECTOR={:#x} KDATA_SELECTOR={:#x}",gdt::KCODE_SELECTOR.0,gdt::KDATA_SELECTOR.0);
        println!("UCODE_SELECTOR={:#x} UDATA_SELECTOR={:#x}",gdt::UCODE_SELECTOR.0,gdt::UDATA_SELECTOR.0);
        tf.cs = gdt::KCODE_SELECTOR.0 as usize;
        tf.rip = code as usize;
        tf.ss = gdt::KDATA_SELECTOR.0 as usize;
        tf.rsp = rsp;
        tf.rflags = 0x282;
        tf
    }
    pub fn new_user_thread(entry_addr: usize, rsp: usize) -> Self {
        use arch::gdt;
        let mut tf = TrapFrame::default();
        tf.cs = gdt::UCODE_SELECTOR.0 as usize;
        tf.rip = entry_addr;
        tf.ss = gdt::UDATA_SELECTOR.0 as usize;
        tf.rsp = rsp;
        tf.rflags = 0x3282;
        tf
    }
}
