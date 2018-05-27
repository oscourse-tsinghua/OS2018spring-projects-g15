# 2018 OS：Rucore 课程设计报告

G15 小组：乔一凡 杨国炜

## 总体概述

本项目是 2018 年操作系统课程设计项目，希望通过 Rust 实现一个类似 ucore 的操作系统，目标平台为 x86_64，同时为操作系统提供必要的驱动程序。我们的源代码位于 [Rucore](https://github.com/oscourse-tsinghua/OS2018spring-projects-g15/)。

## 实验目标描述

### 实验目标

在课程设计最初方案时，我们计划的总体目标是使用 Rust 在 x86_64 实现 ucore 并增加 LKM 模块支持，具体的实现目标也较多：

* 首先使用 Rust 在 x86_64 下重新实现 ucore
* 之后使用 Rust 实现 ATA/SATA 硬盘驱动，以及PS2 键盘驱动。
* 使用 Rust 实现一些相关的 Driver，使 OS 能够支持更多硬件。目前的计划是实现 VGA 的图形 API。
* 在 ucore 的基础上增加 kernel module 的支持，并通过动态可加载模块实现驱动的模块化设计

在实验提出方案时，我们已经有了一个基于博客 [Write an OS in Rust](https://os.phil-opp.com/) 的很简单的 Rust base OS，但是在接下来的实验中，我们发现由于对 rust 不够熟悉，同时对于 os 这类复杂系统的 debug 能力不足，对于工作量和我们自身时间精力与能力的估计有严重的偏差，我们实际完成的工作于预期目标相差不小。

### 完成工作

1. 完成了 ucore 基本框架的移植
   * 对基本硬件的启动和初始化
   * 物理内存管理，物理页分配
   * 虚拟内存管理，x86_64 建立四级页表，有较为简单的虚存管理框架
   * 内核线程和用户进程，可以实现进程切换；但是目前我们的 fork 仍然不能工作
   * 调度器：简单的 Round Robin 算法
   * 没有实现同步互斥机制，使用开关中断的方法和rust自带的Mutex、RwLock实现互斥访问
2. 实现 IDE 硬盘驱动，能够完成 IO 操作
3. 完成一个简单文件系统

## 已有相关工作介绍

我们在工程初期主要参考了[Write an OS in Rust](https://os.phil-opp.com/) 中的 blog_os，建立起了基本的框架。这个简单 os 包括了简单内存管理框架，中断处理框架，并有完善的指导；

我们也参考了其他很多基于 rust 实现的 os，详情可以参考我们的 [wiki](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15) 页面调研情况。其中 Redox 是一个目前完成度较高的 kernel，同样基于 x86_64 且其内存管理框架与 blog_os 很接近，我们在实习过程中也参考了一些 Redox 的实现。

## 小组成员分工

杨国炜主要实现了 IDE 硬盘驱动程序和文件系统，以及进程与调度部分；

乔一凡主要实现了初期工程的框架建立，底层基本驱动，内存管理和中断处理等部分。

## 项目结构

我们使用 Cargo 进行项目管理，项目的主要结构如下所示：

```
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── Xargo.toml
├── build
│   ├── arch
│   │   └── x86_64
│   └── kernel-x86_64.bin
├── src
│   ├── allocator
│   ├── arch
│   │   └── x86_64
│   │       ├── boot
│   │       ├── cpu.rs
│   │       ├── driver
│   │       │   ├── acpi
│   │       │   ├── apic
│   │       │   ├── keyboard
│   │       │   ├── mod.rs
│   │       │   ├── pic
│   │       │   ├── pit
│   │       │   ├── serial
│   │       │   └── vga
│   │       ├── gdt.rs
│   │       ├── idt.rs
│   │       ├── interrupts
│   │       ├── mod.rs
│   │       └── paging
│   ├── consts.rs
│   ├── io
│   ├── lang.rs
│   ├── lib.rs
│   ├── macros.rs
│   ├── memory
│   ├── modules
│   │   └── ps2
│   ├── time.rs
│   └── utils.rs
├── target
└── x86_64-rucore.json
```

* `src/` 下是项目的主题工程代码，`build/` 下是编译生成的内核文件
* `src/allocator/` 实现了一个基于链表和 slab 算法的内存分配器，用于堆空间的分配
* `src/arch` 包含了架构相关的部分的实现，本次工程我们仅实现了 x86_64 下的部分
    * `src/arch/x86_64/boot` 实现了启动部分，包括启动，初始化 gdt，建立一个低地址区恒等映射简单页表，进入 `long mode`
    * `src/arch/x86_64/driver` 实现了底层驱动，包括 ACPI，ACIP，PIT，PIC，VGA 字符显示，串口
    * `src/arch/x86_64/gdt.rs` gdt 有关部分
    * `src/arch/x86_64/idt.rs` idt 表结构与设置部分，具体的中断处理在 `interrupts/` 下
    * `src/arch/x86_64/interrupts` 中断处理部分
    * `src/arch/x86_64/paging` 虚拟内存管理，分页机制
* `src/memory` 物理内存管理，对内存页帧等基本数据结构的定义
* `src/modules` 外部设备驱动，目前实现了 PS/2 键盘鼠标驱动
* `src/modules/ps2` PS/2 键盘鼠标驱动
* `src/consts.rs` 定义工程中使用的常量
* `src/io` 将 VGA 驱动与串口驱动封装，实现方便的 print 和 debug 输出调试信息宏
* `src/macros.rs` 定义了工程中一些常用宏

## 配置运行

运行 `rucore` 需要如下依赖 `rustc`, `rustup`, `cargo`, `xargo`, `x86_64-gcc`, `x86_64-binutils`。

由于 `Rust` 是一门相对年轻的语言，编译器更新很快，语言特性还在不断修改中，新版的 `rustc` 编译器可能无法顺利编译我们的代码。所以我们固定 `rustc` 版本为 `rustc 1.26.0-nightly (9c9424de5 2018-03-27)`
可以使用 `rustup` 进行 `rustc` 版本管理：

```bash
rustup default nightly-2018-01-09
rustup component add rust-src
```

我们使用 `cargo` 进行工程项目管理，在 `cargo.toml` 中声明了所有使用的外部 crate 以及版本，因此不必担心外部 crate 的版本问题。
我们使用 `xargo` 实现交叉编译

我们将工程的编译和运行过程写入 `makefile`，编译运行只需要如下命令：
```bash
make
make run
```

## 实现方案

### 完成了 ucore 基本框架的移植

- 对基本硬件的启动和初始化
- 物理内存管理，物理页分配
- 虚拟内存管理，x86_64 建立四级页表，有较为简单的虚存管理框架
- 内核线程和用户进程，可以实现进程切换；但是目前我们的 fork 仍然不能工作
- 调度器：简单的 Round Robin 算法
- 没有实现同步互斥机制，使用开关中断的方法和rust自带的Mutex、RwLock实现互斥访问

### 底层驱动支持

在最开始我们参考 blog_os 建立起的框架中已经有了 VGA 字符输出的实现，但是没有诸如 pit 等其他的底层驱动。为了实现 lab1 时钟中断 tick 的操作，首先需要补充底层驱动的支持。

其中，我们直接复制使用了 Redox 有关 pit，pic 和串口的驱动，完成对 pit，pic 和串口的初始化工作；

有关 APIC 部分的代码，我们参考了 11 组同学的实现，进行了 Local APIC 和 IO APIC 的初始化；

我们的 ACPI 驱动参考了 Redox 的 ACPI 驱动，针对我们的框架做了修改移植，同时删掉了一些较为复杂的我们不需要的部分（如 AML）的代码，简化了初始化的流程，但同时仍然能够检测 RSDT，FADT 等结构，为 PS2 的 8042 芯片初始化提供了可能。

### 内存管理

#### 物理内存管理

我们使用 GRUB Multiboot2 进行引导，为此我们需要提供满足其规范的 header。我们的 header 定义在`multiboot_header.asm` 中。主要内容是 magic number, 系统架构参数和一些 tags。

在成功引导进入系统后，我们可以读到 `multiboot` 为我们提供的 `Boot Information`，从而获取到目前的物理内存信息和内核文件位置。

在最开始基于 `blog_os` 的框架中，物理内存的分配算法是 FFMA，但是每次仅能分配一页且没有实现释放页帧的操作，比较不完善。我们基于这个框架改善我们内存分配的方法。

将每次分配一页扩展为分配多页的方法比较简单，只需要在遍历所有空闲连续空间时判断当前空间是否具有足够多页帧即可。在释放页帧并回收的实现上，我们参考了 `Redox` 的物理内存实现，采用了一个 `recycler` 进行释放页帧的收集。采用装饰器模式，使用 `recycler` 包裹之前简单的 `allocator`，在分配页帧时先查看 `recycler` 中是否具有足够释放的帧，如果有可以直接分配，否则再调用内部 allocator 进行帧的分配。这样的实现比较简单，同时提高了内存使用效率。

在具体的实现中，我们使用了一个全局变量 `FRAME_ALLOCATOR` 表示这段空间的帧分配器。

```rust
pub static FRAME_ALLOCATOR: Mutex<Option<RecycleAllocator<BumpAllocator>>> = Mutex::new(None);

pub fn init(boot_info: &BootInformation) -> ActivePageTable {
    // ...
    *FRAME_ALLOCATOR.lock() = Some(RecycleAllocator::new(BumpAllocator::new(kernel_start.0 as usize, kernel_end.0 as usize, memory_map_tag.memory_areas())));
    // ...
}
```

可以看到由于是全局变量，在访问中我们需要考虑到互斥访问的问题，因此使用 `Mutex` 对 `Allocator` 进行保护，这也是 `rust` 中是声明全局变量的常用方式。`RecycleAllocator` 即为上述的 `recycler`，内部的 `BumpAllocator` 为基本的 `FFMA Allocator`。在初始化时我们根据 `Boot Information` 提供的信息对分配器进行初始化。同样的，rust 不允许我们跨文件访问全局变量，因此我们需要将所有关于全局变量的操作封装成函数与其声明放在一起。例如分配/释放帧操作：

```rust
/// Allocate a range of frames
pub fn allocate_frames(count: usize) -> Option<Frame> {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.allocate_frames(count)
    } else {
        panic!("frame allocator not initialized");
    }
}

/// Deallocate a range of frames frame
pub fn deallocate_frames(frame: Frame, count: usize) {
    if let Some(ref mut allocator) = *FRAME_ALLOCATOR.lock() {
        allocator.deallocate_frames(frame, count)
    } else {
        panic!("frame allocator not initialized");
    }
}
```

这样虽然降低了编码的自由度，但是对于全局变量使用的限制也同样大大提高了对于全局变量访问的安全性保证。对每次操作加锁则保证了资源的互斥访问。

对于 allocator，我们可以通过 trait （类似接口类）实现对其功能的约束。我们定义 `trait FrameAllocator` 如下：

```rust
pub trait FrameAllocator {
    fn used_frames(& self) -> usize;
    fn free_frames(& self) -> usize;
    fn allocate_frames(&mut self, count: usize) -> Option<Frame>;
    fn deallocate_frames(&mut self, frame: Frame, count: usize);
}
```

可以看到是针对页帧的一些常用操作。我们对于所有的帧分配器都要实现这一 trait，提供相应的操作。

#### 虚拟内存管理

在虚拟内存管理中，我们首先完成段机制的设置。这部分主要设置 gdt 表和 idt 表。

我们的 GDT 表定义使用了 Once，保证对其的全局初始化操作仅能有一次，从而在根本上防止了错误操作可能导致的多次初始化：

```rust
// define
static GDT: Once<Gdt> = Once::new();

//usage
let gdt = GDT.call_once(|| {
    let mut gdt = Gdt::new();
    code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    gdt.add_entry(UCODE);
    gdt.add_entry(KDATA);
    gdt.add_entry(UDATA);
    tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
    gdt
});
```

在设置 gdt 表时，我最初犯了一个错误，将 gdt 表的第一项也设置为 GNULL 段，结果所有段描述符都向后移了一位，导致段选择子都选偏了一项。在加载 GDT 后我们也要相应设定各个段寄存器并加载 tss。

idt 表的设置也是一个大坑。在最初的框架中，我们使用了一个外部 crate 实现 idt 结构以及中断处理例程的设置。虽然这个 crate 将常用的操作封装得很完善，但是由于有的地方封装过于完善，设置起来十分不灵活。对于这个问题我们一直没有好的解决办法，最后我们参考了 G11 组的王润基实现的类似 ucore 中断处理分发机制，解决了这个问题。

在页机制中，我们定义如下四级页表：

```rust
pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}
```

根据 Blog_os，我们相应如下定义 `Mapper`, `ActivePageTable`, `InactivePageTable`，分别表示一个页表结构（包括了对于页表的所有操作），当前活跃页表和一个非活跃页表。

```rust
pub struct Mapper {
    p4: Unique<Table<Level4>>,
}
pub struct ActivePageTable {
    mapper: Mapper,
}
pub struct InactivePageTable {
    pub p4_frame: Frame,
}
```

页表的切换操作是 `ActivePageTable` 中的 `switch` 方法，会在其中进行 CR3 寄存器的改写操作。

同时要注意在每次修改页表后要刷新 `TLB`，这部分功能由 `ActivePageTable` 的 `flush` 方法和定义好的 `MapperFlush` 类完成：

```rust
impl ActivePageTable {
    //...
    pub fn flush(&mut self, page: Page) {
        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        unsafe { tlb::flush(VirtualAddress(page.start_address())); }
    }

    pub fn flush_all(&mut self) {
        use x86_64::instructions::tlb;
        unsafe { tlb::flush_all(); }
    }
    //...
}

#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlush(Page);

impl MapperFlush {
    /// Create a new page flush promise
    pub fn new(page: Page) -> MapperFlush {
        MapperFlush(page)
    }

    /// Flush this page in the active table
    pub fn flush(self, table: &mut ActivePageTable) {
        table.flush(self.0);
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}
```

在这里我们使用了 `rust` 的 `must_use` 表明 `MapperFlush` 必须被使用，而我们在进行页表操作后都会新建一个 `MapperFlush` 对象，这就保证了我们对页表进行操作后一定会进行 `flush` 操作，否则 `rust` 会报错。这也是 `rust` 提供的一个较好用的保证安全的机制。

### PS2 驱动

实现参考了 `reenix` 的 PS2 驱动部分，对代码进行重构，封装设备，降低不同设备与 i8042 芯片的耦合度。

首先需要实现对 `8042` 芯片的检测与初始化，同时，由于 8042 芯片的两个端口对应键盘和鼠标，而键盘鼠标有多种设备形式，所以我们抽象出两个端口，并对键盘鼠标定义 `PS2Dev` 类进行封装。

```rust
pub struct Port {
	is_second: bool,
	dev: super::PS2Dev,
}
```
其中每个 `Port` 与一个 `PS2Dev` 绑定，而 `PS2Dev` 定义如下，包括一个空状态，未知状态，与外设进行交互正在确认设备信息的状态和最终的键盘、鼠标设备：

```rust
enum PS2Dev
{
	None,
	Unknown,
	Enumerating(EnumWaitState),
	Keyboard(keyboard::Dev),
	Mouse(mouse::Dev),
}
```

对于键盘，鼠标，由于有多种 PS2 设备，因此我们也要相应进行抽象。对于键盘，我们有两种键盘 `MF2, MF2Emul`；

```rust
pub struct Dev
{
	ty: Type,
	state: State,
}

pub enum Type
{
	MF2,
	MF2Emul,
}

enum State
{
	Init(Init),
	Idle(Layer,bool),
}
```

对于鼠标，我们同样有三种：`Std, Scroll, QuintBtn`；

```rust
pub enum Type
{
	Std,
	Scroll,
	QuintBtn,	// 5 buttons
}

enum State
{
	Expect(usize),
	Idle,
	WaitByte2(u8),
	WaitByte3(u8,u8),
}

pub struct Dev
{
	ty: Type,
	state: State,
	btns: u8,
}
```

可以看到这种面向对象的设计方法极大地提高了程序的灵活性，同时 `rust` 提供的 `enum` 可以支持任意类型的混合枚举，简化了编码的工作。

在初始化 8042 时，根据协议使用 `inb/outb` 进行初始化，自检和配置，确认 port 的工作状态；

```rust
impl Ctrl8042 {
    unsafe fn new() -> Result<Ctrlr8042,()> {
        let mut ctrlr = Ctrlr8042::default();
        // 1. Disable the controller during setup
        ctrlr.write_cmd(0xAD);	// Disable primary channel
        ctrlr.write_cmd(0xA7);	// Disable secondary channel (ignored if none)
        // - Flush the input FIFO
        ctrlr.flush();
        
        // Read, Modify, Write the controller's config
        ctrlr.write_cmd(0x20);
        let mut config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config");
        // - Clear 0: Port1 Enable, 1: Port2 Enable, 6: Port1 Translation
        config &= !( (1<<0)|(1<<1)|(1<<6) );
        let can_have_second_port = config & (1<<5) != 0;
        ctrlr.write_cmd(0x60);
        ctrlr.write_data(config);
        
        // Self-test
        ctrlr.write_cmd(0xAA);
        match ctrlr.read_data() {
        Ok(0x55) => {},
        Ok(v) => panic!("PS/2 self-test failed ({:#x} exp 0x55)", v),
        Err(_) => panic!("Timeout waiting for PS/2 self-test"),
        }
        
        let has_second_port = if can_have_second_port {
                ctrlr.write_cmd(0xA8);	// Enable second port
                ctrlr.write_cmd(0x20);
                let config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config (2)");
                ctrlr.write_cmd(0xA7);	// Disable secondary channel (ignored if none)
                // If bit is still set, then the second channel is absent
                config & (1 << 5) == 0
            }
            else {
                false
            };
        
        // - Flush the input FIFO (again)
        //  > Just in case data arrived while twiddling with ports
        ctrlr.flush();
        
        let port1_works = {
            ctrlr.write_cmd(0xAB);
            ctrlr.read_data().unwrap() == 0x00
            };
        let port2_works = if has_second_port {
                ctrlr.write_cmd(0xA9);
                ctrlr.read_data().unwrap() == 0x00
            } else {
                false
            };
        debug!("can_have_second_port={:?}, has_second_port={:?}, port1_works={:?}, port2_works={:?}",
            can_have_second_port, has_second_port, port1_works, port2_works);
        
        if !port1_works && !port2_works {
            // nothing works, give up
            debug!("Handle no ports working");
        }
        
        // Enable working ports.
        // - Enable interrupts first
        ctrlr.write_cmd(0x20);
        let mut config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config (2)");
        if port1_works {
            config |= 1 << 0;	// Enable interrupt
        }
        if port2_works {
            config |= 1 << 1;	// Enable interrupt
        }
        debug!("Controller config = 0b{:08b}", config);
        ctrlr.write_cmd(0x60);
        ctrlr.write_data(config);
        // - Enable ports second
        if port1_works {
            let mut port = Port::new(false);
            debug!("Enabling port 1");
            ctrlr.write_cmd(0xAE);
            ctrlr.write_data(0xFF);
        }
        if port2_works {
            let mut port = Port::new(true);
            debug!("Enabling port 2");
            ctrlr.write_cmd(0xA8);
            ctrlr.write_cmd(0xD4);
            ctrlr.write_data(0xFF);
        }
        
        Ok( ctrlr )
    }
}
```

在收到 PS2 设备发送的信息后，驱动会维护一个状态机判断设备类型并相应设置端口设备，准备好后即可实现响应响应设备的请求；响应设备请求同样根据协议使用状态机进行处理。事实上在对设备进行抽象后，不同设备的响应实现逻辑都比较相似，简化了实现。

比如对键盘来说，我们首先定义设备的不同状态，之后依据设备当前所处的状态进行相应的响应和设置操作：

```rust
enum State
{
	Init(Init),
	Idle(Layer,bool),
}

enum Layer
{
	Base,
	E0,
	E1,
}

enum Init
{
	Disabled,
	ReqScancodeSetAck,
	ReqScancodeSetRsp,
	SetLeds(u8),
}
```

可以看到根据设备的不同类型和当前所处的不同运行状态，我们定义了多种设备状态。在接收到 `i8042` 芯片的信号后，响应函数将根据设备现在的状态和收到的信息进行相应的状态转移。这部分的操作由于 `rust` 提供了方便的 `match` 操作而大大得到简化。

```rust
impl PS2Dev {
    //...
    pub fn recv_byte(&mut self, byte: u8) -> Option<u8> {
		let (rv, new_state): (Option<_>,Option<_>) = match *self
			{
			PS2Dev::None =>
				//...
                ,
			PS2Dev::Unknown => (None, None),
			PS2Dev::Enumerating(state) => match state
				{
				EnumWaitState::DSAck =>
					//...
                    ,
				EnumWaitState::IdentAck =>
					//...
                    ,
				//...
                ,
				},
			PS2Dev::Keyboard(ref mut dev) => {
				(dev.recv_byte(byte), None)
				},
			PS2Dev::Mouse(ref mut dev) => {
				(dev.recv_byte(byte), None)
				},
			};
		
		if let Some(ns) = new_state
		{
			debug!("Byte {:#02x} caused State transition {:?} to {:?}", byte, *self, ns);
			*self = ns;
		}
		rv
	}
    //...
}
```

这是处理时的简单框架，可以看到通过使用嵌套的 `match` 操作进行状态的判断和处理十分方便。

### 实现 IDE 硬盘驱动，能够完成 IO 操作

参考ucore实现了ide硬盘驱动。

能实现将磁盘某位位置连续的n个扇区大小的数据读入到dst数组中，同时能将dst数组写入到磁盘某位位置后连续的地址中。

在实现中rust的x86_64::port提供了如inb、outb等函数，因此相较使用c实现更加简单，可以不需要进行汇编代码的编写。

### 完成一个简单文件系统

参考rust_os及ucore完成了简单的文件系统，rust_os实现的是ramfs，ramfs是一种基于RAM做存储的文件系统，RAM做存储所以会有很高的存储效率。但由于ramfs的实现就相当于把RAM作为最后一层的存储，所以在ramfs中不会使用swap。因此ramfs有一个很大的缺陷就是它会吃光系统所有的内存，同时它也只能被root用户访问。

我基于上面实现的IDE driver，参考ucore及rust_os完成了基于硬盘的简单文件系统，该文件系统能处理以下几种类型的文件，其中Symlink实现的是软链接机制：

* pub enum NodeType<'a>{

  ​    File,  //常规文件类型

  ​    Dir,  //文件夹类型

  ​    Symlink(&'a super::Path),  //链接类型，允许读取链接内容

  }

对三种不同类型文件的基本操作包括

* File

```rust
pub trait File: NodeBase {
​    /// 返回此文件的大小（以字节为单位）
​    fn size(&self) -> u64;
​    /// 更新文件的大小（零填充或截断）
​    fn truncate(&self, newsize: u64) -> Result<u64>;
​    /// 清除文件的指定范围（用零替换）
​    fn clear(&self, ofs: u64, size: u64) -> Result<()>;
​    /// 从文件中读取数据
​    fn read(&self, ofs: u64, buf: &mut [u32]) -> Result<usize>;
​    /// 将数据写入文件
​    fn write(&mut self, id: InodeId, buf: &[u32]) -> Result<usize>;
}
```

* Dir

```rust
pub trait Dir: NodeBase {
​    /// 获取给定名称的节点
​    fn lookup(&self, name: &ByteStr) -> Result<InodeId>;
​    /// 读取条目
​    /// 返回
​    /// - Ok(Next Offset)
​    /// - Err(e) : 错误
​    fn read(&self, start_ofs: usize, callback: &mut ReadDirCallback) -> Result<usize>;
​    /// 在该目录下创建一个新文件，返回新创建的节点编号
​    fn create(&self, name: &ByteStr, nodetype: NodeType) -> Result<InodeId>;
}
```

* Symlink

```rust
pub trait Symlink: NodeBase {
​    /// 将符号链接的内容读入一个字符串
​    fn read(&self) -> ByteString;
}
```

* 内存索引节点结构，描述了文件的inode等信息，用于引用计数、同步互斥等操作。

```rust
pub struct CacheHandle{
​    mountpt: usize,  //挂载点编号
​    inode: InodeId,   //inode编号
​    ptr: *const CachedNode,
} 

struct CachedNode{
​    refcount: AtomicUsize,  //引用计数
​    node: CacheNodeInt,    //inode，用枚举类型表示，有3种不同的inode
} 
```

## 实验过程日志

详见 wiki 页面过程记录 [http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15)

## 实验总结

总结来说，使用 rust 写 os 是一个痛苦而又有收获的过程。首先，由于我们对 rust 本身不够熟悉，同时 rust 本身编译要求十分严格，经常出现死活编译不过的情况；另一方面，对 rust 的不熟悉也使我们对一些 rust 的高级特性不够了解，降低了开发效率的同时也一定程度上损失了 rust 提供的高等级安全保证的特性。

但另一方面，rust 本身对于资源申请和使用所有权的严格要求也大大降低了代码出错的概率；可能的不安全代码段会强制程序员使用 unsafe 进行声明；对于权限和可见性的更加严格的要求，等等语言特性在编译器就通过严格的约束帮助程序员及时发现 bug，降低 debug 成本。

同时，rust 作为比 C 更加高级的语言，在语言层面提供了更多描述能力更强的特性和实现。同时，rust 的包管理器 cargo 可以让我们更方便地指定工程使用的外部库以及版本，从而可以方便地利用各种现成的轮子。如在实现中我们大量使用的 spin::Mutex 等。
在文件系统等代码中我们大量用到了枚举类型enum，该类型相较于c的enum的优点是，rust的enum不同元素可以为不同的类型，而c的enum只能是数字。


## 参考文献与代码

Rust OS

- 一个详尽的使用 Rust 开发 blog_os 的博客： [Write an OS in Rust](https://os.phil-opp.com/)
  - 目标平台为 x86_64
  - 目前有 bootloader，简单的内存管理模块，支持简单的中断机制
- Reenix: [Reenix: A Rust version of the Weenix OS](https://github.com/scialex/reenix)
  - 一个 Brown Univ 的同学的毕业设计，使用 Rust 重写了 weenix 教学 os
  - 不是纯 Rust 的结构，包含大量 C 代码
- Redox：[A Rust Operating System](https://github.com/redox-os/redox/)，一个目前最完善的 Rust based OS
  - 官方主页：[Redox](https://www.redox-os.org/)
- Stanford CS 140e 课程：[Stanford cs140e](https://web.stanford.edu/class/cs140e/)
  - 使用 Rust 在树莓派上实现 os

Rust Driver

- Reenix 中包含有简单的 ATA Driver：[Reenix Drivers](https://github.com/scialex/reenix/tree/vfs/kernel/drivers)
  - 实现上是使用 C 实现的，可以参考
- Rust OS：["Tifflin" Experimental Kernel](https://github.com/thepowersgang/rust_os/)，包含很多驱动支持，可以参考其实现
- Driver Helper Slides：[Brown CS 167 Drivers slides](http://cs.brown.edu/courses/cs167/projects/drivers-help.pdf)

实现中也参考了其他两个 rust 组的实现：

* G11 [https://github.com/wangrunji0408/RustOS](https://github.com/wangrunji0408/RustOS)

* G13 [https://github.com/oscourse-tsinghua/OS2018spring-projects-g13](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13)

