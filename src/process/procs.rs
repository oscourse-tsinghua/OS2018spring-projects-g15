use super::*;
use xmas_elf::{ElfFile, program::{Flags, ProgramHeader}, header::HeaderPt2};
use core::slice;
use alloc::rc::Rc;
use rlibc::memcpy;

// process's state in his life cycle
enum proc_state {
    Ready, Running, Sleeping(usize), Exited
};

// Saved registers for kernel context switches.
// Don't need to save all the %fs etc. segment registers,
// because they are constant across kernel contexts.
// Save all the regular registers so we don't need to care
// which are caller save, but not the return register %eax.
// (Not saving %eax just simplifies the switching code.)
// The layout of context must match code in switch.S.
struct context {
    eip:u32;
    esp:u32;
    ebx:u32;
    ecx:u32;
    edx:u32;
    esi:u32;
    edi:u32;
    ebp:u32;
};

let PROC_NAME_LEN=15;
let MAX_PROCESS=4096;
let MAX_PID=MAX_PROCESS * 2;

#[derive(Debug)]
pub struct proc_struct {
    pub(in process) pid: Pid,
                    //name: &'static str,
                    kstack: Stack,
    pub(in process) memory_set: Option<MemorySet>,
    pub(in process) page_table: Option<InactivePageTable>,
    pub(in process) status: proc_state,
    pub(in process) rsp: usize,
    pub(in process) is_user: bool,
}

impl proc_state{
    pub fn new(name: &'static str, entry: extern fn(), mc: &mut MemoryController) -> Self {
    }

    pub fn new_init(mc: &mut MemoryController) -> Self {
    }

    pub fn new_user(begin: usize, end: usize, mc: &mut MemoryController) -> Self {
    }
    
    pub fn fork(&self, tf: &TrapFrame, mc: &mut MemoryController) -> Self {
    }
}

let PF_EXITING=0x00000001;      // getting shutdown

let WT_CHILD=(0x00000001 | WT_INTERRUPTED);
let WT_INTERRUPTED=0x80000000;                    // the wait state could be interrupted


#define le2proc(le, member)         \
    to_struct((le), struct proc_struct, member)

let HASH_SHIFT=10;
let HASH_LIST_SIZE=1 << HASH_SHIFT;
let GOLDEN_RATIO_PRIME_32=0x9e370001;

pub fn hash32(val:usize, bits:usize) {
    let hash = val * GOLDEN_RATIO_PRIME_32;
    (hash >> (32 - bits))
}

pub fn pid_hashfn(x:usize){
    hash32(x, HASH_SHIFT)
}

// has list for process set based on pid
static list_entry_t hash_list[HASH_LIST_SIZE];

// idle proce
static idleproc:proc_struct;
// init proce
static initproc:proc_struct;
// current proce
static current:proc_struct;

let nr_process = 0;

void kernel_thread_entry(void);
void forkrets(struct trapframe *tf);
void switch_to(struct context *from, struct context *to);

// alloc_proc - alloc a proc_struct and init all fields of proc_struct
static struct proc_struct *
alloc_proc(void) {
    struct proc_struct *proce = kmalloc(sizeof(struct proc_struct));
    if (proce != NULL) {
        proce->state = PROC_UNINIT;
        proce->pid = -1;
        proce->runs = 0;
        proce->kstack = 0;
        proce->need_resched = 0;
        proce->parent = NULL;
        proce->mm = NULL;
        memset(&(proce->context), 0, sizeof(struct context));
        proce->tf = NULL;
        proce->cr3 = boot_cr3;
        proce->flags = 0;
        memset(proce->name, 0, PROC_NAME_LEN);
    }
    return proce;
}

// set_proc_name - set the name of proce
char *
set_proc_name(struct proc_struct *proce, const char *name) {
    memset(proce->name, 0, sizeof(proce->name));
    return memcpy(proce->name, name, PROC_NAME_LEN);
}

// get_proc_name - get the name of proce
char *
get_proc_name(struct proc_struct *proce) {
    static char name[PROC_NAME_LEN + 1];
    memset(name, 0, sizeof(name));
    return memcpy(name, proce->name, PROC_NAME_LEN);
}

// get_pid - alloc a unique pid for process
static int
get_pid(void) {
    static_assert(MAX_PID > MAX_PROCESS);
    struct proc_struct *proce;
    list_entry_t *list = &proc_list, *le;
    static int next_safe = MAX_PID, last_pid = MAX_PID;
    if (++ last_pid >= MAX_PID) {
        last_pid = 1;
        goto inside;
    }
    if (last_pid >= next_safe) {
    inside:
        next_safe = MAX_PID;
    repeat:
        le = list;
        while ((le = list_next(le)) != list) {
            proce = le2proc(le, list_link);
            if (proce->pid == last_pid) {
                if (++ last_pid >= next_safe) {
                    if (last_pid >= MAX_PID) {
                        last_pid = 1;
                    }
                    next_safe = MAX_PID;
                    goto repeat;
                }
            }
            else if (proce->pid > last_pid && next_safe > proce->pid) {
                next_safe = proce->pid;
            }
        }
    }
    return last_pid;
}

// proc_run - make process "proce" running on cpu
// NOTE: before call switch_to, should load  base addr of "proce"'s new PDT
void
proc_run(struct proc_struct *proce) {
    if (proce != current) {
        bool intr_flag;
        struct proc_struct *prev = current, *next = proce;
        local_intr_save(intr_flag);
        {
            current = proce;
            load_esp0(next->kstack + KSTACKSIZE);
            lcr3(next->cr3);
            switch_to(&(prev->context), &(next->context));
        }
        local_intr_restore(intr_flag);
    }
}

// forkret -- the first kernel entry point of a new thread/process
// NOTE: the addr of forkret is setted in copy_thread function
//       after switch_to, the current proce will execute here.
static void
forkret(void) {
    forkrets(current->tf);
}

// hash_proc - add proce into proce hash_list
static void
hash_proc(struct proc_struct *proce) {
    list_add(hash_list + pid_hashfn(proce->pid), &(proce->hash_link));
}

// find_proc - find proce frome proce hash_list according to pid
struct proc_struct *
find_proc(int pid) {
    if (0 < pid && pid < MAX_PID) {
        list_entry_t *list = hash_list + pid_hashfn(pid), *le = list;
        while ((le = list_next(le)) != list) {
            struct proc_struct *proce = le2proc(le, hash_link);
            if (proce->pid == pid) {
                return proce;
            }
        }
    }
    return NULL;
}

// kernel_thread - create a kernel thread using "fn" function
// NOTE: the contents of temp trapframe tf will be copied to 
//       proce->tf in do_fork-->copy_thread function
int
kernel_thread(int (*fn)(void *), void *arg, uint32_t clone_flags) {
    struct trapframe tf;
    memset(&tf, 0, sizeof(struct trapframe));
    tf.tf_cs = KERNEL_CS;
    tf.tf_ds = tf.tf_es = tf.tf_ss = KERNEL_DS;
    tf.tf_regs.reg_ebx = (uint32_t)fn;
    tf.tf_regs.reg_edx = (uint32_t)arg;
    tf.tf_eip = (uint32_t)kernel_thread_entry;
    return do_fork(clone_flags | CLONE_VM, 0, &tf);
}

// setup_kstack - alloc pages with size KSTACKPAGE as process kernel stack
static int
setup_kstack(struct proc_struct *proce) {
    struct Page *page = alloc_pages(KSTACKPAGE);
    if (page != NULL) {
        proce->kstack = (uintptr_t)page2kva(page);
        return 0;
    }
    return -E_NO_MEM;
}

// put_kstack - free the memory space of process kernel stack
static void
put_kstack(struct proc_struct *proce) {
    free_pages(kva2page((void *)(proce->kstack)), KSTACKPAGE);
}

// copy_mm - process "proce" duplicate OR share process "current"'s mm according clone_flags
//         - if clone_flags & CLONE_VM, then "share" ; else "duplicate"
static int
copy_mm(uint32_t clone_flags, struct proc_struct *proce) {
    assert(current->mm == NULL);
    /* do nothing in this project */
    return 0;
}

// copy_thread - setup the trapframe on the  process's kernel stack top and
//             - setup the kernel entry point and stack of process
static void
copy_thread(struct proc_struct *proce, uintptr_t esp, struct trapframe *tf) {
    proce->tf = (struct trapframe *)(proce->kstack + KSTACKSIZE) - 1;
    *(proce->tf) = *tf;
    proce->tf->tf_regs.reg_eax = 0;
    proce->tf->tf_esp = esp;
    proce->tf->tf_eflags |= FL_IF;

    proce->context.eip = (uintptr_t)forkret;
    proce->context.esp = (uintptr_t)(proce->tf);
}

/* do_fork -     parent process for a new child process
 * @clone_flags: used to guide how to clone the child process
 * @stack:       the parent's user stack pointer. if stack==0, It means to fork a kernel thread.
 * @tf:          the trapframe info, which will be copied to child process's proce->tf
 */
int
do_fork(uint32_t clone_flags, uintptr_t stack, struct trapframe *tf) {
    int ret = -E_NO_FREE_PROC;
    struct proc_struct *proce;
    if (nr_process >= MAX_PROCESS) {
        goto fork_out;
    }
    ret = -E_NO_MEM;
    //LAB4:EXERCISE2 YOUR CODE
    /*
     * Some Useful MACROs, Functions and DEFINEs, you can use them in below implementation.
     * MACROs or Functions:
     *   alloc_proc:   create a proce struct and init fields (lab4:exercise1)
     *   setup_kstack: alloc pages with size KSTACKPAGE as process kernel stack
     *   copy_mm:      process "proce" duplicate OR share process "current"'s mm according clone_flags
     *                 if clone_flags & CLONE_VM, then "share" ; else "duplicate"
     *   copy_thread:  setup the trapframe on the  process's kernel stack top and
     *                 setup the kernel entry point and stack of process
     *   hash_proc:    add proce into proce hash_list
     *   get_pid:      alloc a unique pid for process
     *   wakeup_proc:  set proce->state = PROC_RUNNABLE
     * VARIABLES:
     *   proc_list:    the process set's list
     *   nr_process:   the number of process set
     */

    //    1. call alloc_proc to allocate a proc_struct
    //    2. call setup_kstack to allocate a kernel stack for child process
    //    3. call copy_mm to dup OR share mm according clone_flag
    //    4. call copy_thread to setup tf & context in proc_struct
    //    5. insert proc_struct into hash_list && proc_list
    //    6. call wakeup_proc to make the new child process RUNNABLE
    //    7. set ret vaule using child proce's pid
    if ((proce = alloc_proc()) == NULL) {
        goto fork_out;
    }

    proce->parent = current;

    if (setup_kstack(proce) != 0) {
        goto bad_fork_cleanup_proc;
    }
    if (copy_mm(clone_flags, proce) != 0) {
        goto bad_fork_cleanup_kstack;
    }
    copy_thread(proce, stack, tf);

    bool intr_flag;
    local_intr_save(intr_flag);
    {
        proce->pid = get_pid();
        hash_proc(proce);
        list_add(&proc_list, &(proce->list_link));
        nr_process ++;
    }
    local_intr_restore(intr_flag);

    wakeup_proc(proce);

    ret = proce->pid;
fork_out:
    return ret;

bad_fork_cleanup_kstack:
    put_kstack(proce);
bad_fork_cleanup_proc:
    kfree(proce);
    goto fork_out;
}

// do_exit - called by sys_exit
//   1. call exit_mmap & put_pgdir & mm_destroy to free the almost all memory space of process
//   2. set process' state as PROC_ZOMBIE, then call wakeup_proc(parent) to ask parent reclaim itself.
//   3. call scheduler to switch to other process
int
do_exit(int error_code) {
    panic("process exit!!.\n");
}

// init_main - the second kernel thread used to create user_main kernel threads
static int
init_main(void *arg) {
    cprintf("this initproc, pid = %d, name = \"%s\"\n", current->pid, get_proc_name(current));
    cprintf("To U: \"%s\".\n", (const char *)arg);
    cprintf("To U: \"en.., Bye, Bye. :)\"\n");
    return 0;
}

// proc_init - set up the first kernel thread idleproc "idle" by itself and 
//           - create the second kernel thread init_main
void
proc_init(void) {
    int i;

    list_init(&proc_list);
    for (i = 0; i < HASH_LIST_SIZE; i ++) {
        list_init(hash_list + i);
    }

    if ((idleproc = alloc_proc()) == NULL) {
        panic("cannot alloc idleproc.\n");
    }

    idleproc->pid = 0;
    idleproc->state = PROC_RUNNABLE;
    idleproc->kstack = (uintptr_t)bootstack;
    idleproc->need_resched = 1;
    set_proc_name(idleproc, "idle");
    nr_process ++;

    current = idleproc;

    int pid = kernel_thread(init_main, "Hello world!!", 0);
    if (pid <= 0) {
        panic("create init_main failed.\n");
    }

    initproc = find_proc(pid);
    set_proc_name(initproc, "init");

    assert(idleproc != NULL && idleproc->pid == 0);
    assert(initproc != NULL && initproc->pid == 1);
}

// cpu_idle - at the end of kern_init, the first kernel thread idleproc will do below works
void
cpu_idle(void) {
    while (1) {
        if (current->need_resched) {
            schedule();
        }
    }
}

