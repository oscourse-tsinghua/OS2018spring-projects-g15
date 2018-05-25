# 2018 OS：Rucore 课程设计报告

G15 小组：乔一凡 杨国炜

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
   * 没有实现同步互斥机制，使用开关中断的方法实现互斥访问
2. 实现 IDE 硬盘驱动，能够完成 IO 操作
3. 完成一个简单文件系统

## 已有相关工作介绍

我们在工程初期主要参考了[Write an OS in Rust](https://os.phil-opp.com/) 中的 blog_os，建立起了基本的框架。这个简单 os 包括了简单内存管理框架，中断处理框架，并有完善的指导；

我们也参考了其他很多基于 rust 实现的 os，详情可以参考我们的 [wiki](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15) 页面调研情况。其中 Redox 是一个目前完成度较高的 kernel，同样基于 x86_64 且其内存管理框架与 blog_os 很接近，我们在实习过程中也参考了一些 Redox 的实现。

## 小组成员分工

杨国炜主要实现了 IDE 硬盘驱动程序和文件系统，以及进程与调度部分；

乔一凡主要实现了初期工程的框架建立，底层基本驱动，内存管理和中断处理等部分。

## 实现方案

### 底层驱动支持

在最开始我们参考 blog_os 建立起的框架中已经有了 VGA 字符输出的实现，但是没有诸如 pit 等其他的底层驱动。为了实现 lab1 时钟中断 tick 的操作，首先需要补充底层驱动的支持。

其中，我们直接复制使用了 Redox 有关 pit，pic 和串口的驱动，完成对 pit，pic 和串口的初始化工作；

有关 APIC 部分的代码，我们参考了 13 组同学的实现，进行了 Local APIC 和 IO APIC 的初始化；

我们的 ACPI 驱动参考了 Redox 的 ACPI 驱动，针对我们的框架做了修改移植，同时删掉了一些较为复杂的我们不需要的部分（如 AML）的代码，简化了初始化的流程，但同时仍然能够检测 RSDT，FADT 等结构，为 PS2 的 8042 芯片初始化提供了可能。

### 内存管理

在最开始基于 blog_os 的框架中，物理内存的分配算法是 FFMA，但是每次仅能分配一页且没有实现释放页帧的操作，比较不完善。我们基于这个框架改善我们内存分配的方法。

将每次分配一页扩展为分配多页的方法比较简单，只需要在遍历所有空闲连续空间时判断当前空间是否具有足够多页帧即可。在释放页帧并回收的实现上，我们参考了 Redox 的物理内存实现，采用了一个 recycler 进行释放页帧的收集。采用装饰器模式，使用 recycler 包裹之前简单的 allocator，在分配页帧时先查看 recycler 中是否具有足够释放的帧，如果有可以直接分配，否则再调用内部 allocator 进行帧的分配。这样的实现比较简单，同时提高了内存使用效率。

在虚拟内存管理中，我们首先完成段机制的设置。这部分主要设置 gdt 表和 idt 表。

在设置 gdt 表时，我最初犯了一个错误，将 gdt 表的第一项也设置为 GNULL 段，结果所有段描述符都向后移了一位，导致段选择子都选偏了一项。

idt 表的设置也是一个大坑。在最初的框架中，我们使用了一个外部 crate 实现 idt 结构以及中断处理例程的设置。虽然这个 crate 将常用的操作封装得很完善，但是由于有的地方封装过于完善，设置起来十分不灵活。对于这个问题我们一直没有好的解决办法，最后我们参考了 G13 组的王润基实现的类似 ucore 中断处理分发机制，解决了这个问题。

### PS2 驱动

实现参考了 reenix 的 PS2 驱动部分，对代码进行重构，封装设备，降低不同设备与 i8042 芯片的耦合度。

首先需要实现对 8042 芯片的检测与初始化，同时，由于 8042 芯片的两个端口对应键盘和鼠标，而键盘鼠标有多种设备形式，所以我们抽象出两个端口，并对键盘鼠标定义 PS2Dev 类进行封装。

在初始化 8042 时，根据协议使用 inb/outb 进行初始化，自检和配置，确认 port 的工作状态；

对于键盘，我们有两种键盘 MF2, MF2Emul；

对于鼠标，我们同样有三种：Std, Scroll, QuintBtn；

在收到 PS2 设备发送的信息后，驱动会维护一个状态机判断设备类型并相应设置端口设备，准备好后即可实现响应响应设备的请求；响应设备请求同样根据协议使用状态机进行处理。事实上在对设备进行抽象后，不同设备的响应实现逻辑都比较相似，简化了实现。



## 实验过程日志

详见 wiki 页面过程记录 [http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15)

## 实验总结

总结来说，使用 rust 写 os 是一个痛苦而又有收获的过程。首先，由于我们对 rust 本身不够熟悉，同时 rust 本身编译要求十分严格，经常出现死活编译不过的情况；另一方面，对 rust 的不熟悉也使我们对一些 rust 的高级特性不够了解，降低了开发效率的同时也一定程度上损失了 rust 提供的高等级安全保证的特性。

但另一方面，rust 本身对于资源申请和使用所有权的严格要求也大大降低了代码出错的概率；可能的不安全代码段会强制程序员使用 unsafe 进行声明；对于权限和可见性的更加严格的要求，等等语言特性在编译器就通过严格的约束帮助程序员及时发现 bug，降低 debug 成本。

同时，rust 作为比 C 更加高级的语言，在语言层面提供了更多描述能力更强的特性和实现。同时，rust 的包管理器 cargo 可以让我们更方便地指定工程使用的外部库以及版本，从而可以方便地利用各种现成的轮子。如在实现中我们大量使用的 spin::Mutex 等。

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

  