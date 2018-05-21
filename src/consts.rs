pub const MAX_CPU_NUM: usize = 8;

// Copy from Redox consts.rs:

// Because the memory map is so important to not be aliased, it is defined here, in one place
// The lower 256 PML4 entries are reserved for userspace
// Each PML4 entry references up to 512 GB of memory
// The top (511) PML4 is reserved for recursive mapping
// The second from the top (510) PML4 is reserved for the kernel
    /// The size of a single PML4
    pub const PML4_SIZE: usize = 0x0000_0080_0000_0000;
    pub const PML4_MASK: usize = 0x0000_ff80_0000_0000;

    /// Offset of recursive paging
    pub const RECURSIVE_PAGE_OFFSET: usize = (-(PML4_SIZE as isize)) as usize;
    pub const RECURSIVE_PAGE_PML4: usize = (RECURSIVE_PAGE_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset of kernel
    pub const KERNEL_OFFSET: usize = RECURSIVE_PAGE_OFFSET - PML4_SIZE;
    pub const KERNEL_PML4: usize = (KERNEL_OFFSET & PML4_MASK)/PML4_SIZE;

    pub const KERNEL_SIZE: usize = PML4_SIZE;

    /// Offset to kernel heap
    pub const KERNEL_HEAP_OFFSET: usize = KERNEL_OFFSET - PML4_SIZE;
    pub const KERNEL_HEAP_PML4: usize = (KERNEL_HEAP_OFFSET & PML4_MASK)/PML4_SIZE;
    /// Size of kernel heap
    pub const KERNEL_HEAP_SIZE: usize = 1 * 1024 * 1024; // 1 MB

    /// Offset to kernel percpu variables
    //TODO: Use 64-bit fs offset to enable this pub const KERNEL_PERCPU_OFFSET: usize = KERNEL_HEAP_OFFSET - PML4_SIZE;
    pub const KERNEL_PERCPU_OFFSET: usize = 0xC000_0000;
    /// Size of kernel percpu variables
    pub const KERNEL_PERCPU_SIZE: usize = 64 * 1024; // 64 KB

    /// Offset to user image
    pub const USER_OFFSET: usize = 0;
    pub const USER_PML4: usize = (USER_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user TCB
    pub const USER_TCB_OFFSET: usize = 0xB000_0000;

    /// Offset to user arguments
    pub const USER_ARG_OFFSET: usize = USER_OFFSET + PML4_SIZE/2;

    /// Offset to user heap
    pub const USER_HEAP_OFFSET: usize = USER_OFFSET + PML4_SIZE;
    pub const USER_HEAP_PML4: usize = (USER_HEAP_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user grants
    pub const USER_GRANT_OFFSET: usize = USER_HEAP_OFFSET + PML4_SIZE;
    pub const USER_GRANT_PML4: usize = (USER_GRANT_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user stack
    pub const USER_STACK_OFFSET: usize = USER_GRANT_OFFSET + PML4_SIZE;
    pub const USER_STACK_PML4: usize = (USER_STACK_OFFSET & PML4_MASK)/PML4_SIZE;
    /// Size of user stack
    pub const USER_STACK_SIZE: usize = 1024 * 1024; // 1 MB

    /// Offset to user sigstack
    pub const USER_SIGSTACK_OFFSET: usize = USER_STACK_OFFSET + PML4_SIZE;
    pub const USER_SIGSTACK_PML4: usize = (USER_SIGSTACK_OFFSET & PML4_MASK)/PML4_SIZE;
    /// Size of user sigstack
    pub const USER_SIGSTACK_SIZE: usize = 256 * 1024; // 256 KB

    /// Offset to user TLS
    pub const USER_TLS_OFFSET: usize = USER_SIGSTACK_OFFSET + PML4_SIZE;
    pub const USER_TLS_PML4: usize = (USER_TLS_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary image (used when cloning)
    pub const USER_TMP_OFFSET: usize = USER_TLS_OFFSET + PML4_SIZE;
    pub const USER_TMP_PML4: usize = (USER_TMP_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary heap (used when cloning)
    pub const USER_TMP_HEAP_OFFSET: usize = USER_TMP_OFFSET + PML4_SIZE;
    pub const USER_TMP_HEAP_PML4: usize = (USER_TMP_HEAP_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary page for grants
    pub const USER_TMP_GRANT_OFFSET: usize = USER_TMP_HEAP_OFFSET + PML4_SIZE;
    pub const USER_TMP_GRANT_PML4: usize = (USER_TMP_GRANT_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary stack (used when cloning)
    pub const USER_TMP_STACK_OFFSET: usize = USER_TMP_GRANT_OFFSET + PML4_SIZE;
    pub const USER_TMP_STACK_PML4: usize = (USER_TMP_STACK_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary sigstack (used when cloning)
    pub const USER_TMP_SIGSTACK_OFFSET: usize = USER_TMP_STACK_OFFSET + PML4_SIZE;
    pub const USER_TMP_SIGSTACK_PML4: usize = (USER_TMP_SIGSTACK_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset to user temporary tls (used when cloning)
    pub const USER_TMP_TLS_OFFSET: usize = USER_TMP_SIGSTACK_OFFSET + PML4_SIZE;
    pub const USER_TMP_TLS_PML4: usize = (USER_TMP_TLS_OFFSET & PML4_MASK)/PML4_SIZE;

    /// Offset for usage in other temporary pages
    pub const USER_TMP_MISC_OFFSET: usize = USER_TMP_TLS_OFFSET + PML4_SIZE;
    pub const USER_TMP_MISC_PML4: usize = (USER_TMP_MISC_OFFSET & PML4_MASK)/PML4_SIZE;

pub mod irq {
	pub const T_IRQ0       : u8 = 32;   // IRQ 0 corresponds to int T_IRQ
    pub const T_DIVIDE     : u8 =  0;   // divide error
    pub const T_DEBUG      : u8 =  1;   // debug exception
    pub const T_NMI        : u8 =  2;   // non-maskable interrupt
    pub const T_BRKPT      : u8 =  3;   // breakpoint
    pub const T_OFLOW      : u8 =  4;   // overflow
    pub const T_BOUND      : u8 =  5;   // bounds check
    pub const T_ILLOP      : u8 =  6;   // illegal opcode
    pub const T_DEVICE     : u8 =  7;   // device not available
    pub const T_DBLFLT     : u8 =  8;   // double fault
    pub const T_TSS        : u8 = 10;  // invalid task switch segment
    pub const T_SEGNP      : u8 = 11;  // segment not present
    pub const T_STACK      : u8 = 12;  // stack exception
    pub const T_GPFLT      : u8 = 13;  // general protection fault
    pub const T_PGFLT      : u8 = 14;  // page fault
    pub const T_FPERR      : u8 = 16;  // floating point error
    pub const T_ALIGN      : u8 = 17;  // aligment check
    pub const T_MCHK       : u8 = 18;  // machine check
    pub const T_SIMDERR    : u8 = 19;  // SIMD floating point error

    pub const T_SYSCALL    : u8 = 0x80; // SYSCALL, ONLY FOR THIS PROJ
    pub const T_SWITCH_TOU : u8 = 120;  // user/kernel switch
    pub const T_SWITCH_TOK : u8 = 121;  // user/kernel switch

	pub const IRQ_TIMER    : u8 =  0;
	pub const IRQ_KBD      : u8 =  1;
	pub const IRQ_COM2     : u8 =  3;
	pub const IRQ_COM1     : u8 =  4;
    pub const IRQ_MOUSE    : u8 = 12;
	pub const IRQ_IDE1     : u8 = 14;
    pub const IRQ_IDE2     : u8 = 15;
	pub const IRQ_ERROR    : u8 = 19;
	pub const IRQ_SPURIOUS : u8 = 31;
}
