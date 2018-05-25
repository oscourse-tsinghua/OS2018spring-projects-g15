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
   * 没有实现同步互斥机制，使用开关中断的方法和rust自带的Mutex、RwLock实现互斥访问
2. 实现 IDE 硬盘驱动，能够完成 IO 操作
3. 完成一个简单文件系统

## 已有相关工作介绍

我们在工程初期主要参考了[Write an OS in Rust](https://os.phil-opp.com/) 中的 blog_os，建立起了基本的框架。这个简单 os 包括了简单内存管理框架，中断处理框架，并有完善的指导；

我们也参考了其他很多基于 rust 实现的 os，详情可以参考我们的 [wiki](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15) 页面调研情况。其中 Redox 是一个目前完成度较高的 kernel，同样基于 x86_64 且其内存管理框架与 blog_os 很接近，我们在实习过程中也参考了一些 Redox 的实现。

## 小组成员分工

杨国炜主要实现了 IDE 硬盘驱动程序和文件系统，以及进程与调度部分；

乔一凡主要实现了初期工程的框架建立，底层基本驱动，内存管理和中断处理等部分。

## 实现方案

### 完成了 ucore 基本框架的移植

- 对基本硬件的启动和初始化
- 物理内存管理，物理页分配
- 虚拟内存管理，x86_64 建立四级页表，有较为简单的虚存管理框架
- 内核线程和用户进程，可以实现进程切换；但是目前我们的 fork 仍然不能工作
- 调度器：简单的 Round Robin 算法
- 没有实现同步互斥机制，使用开关中断的方法和rust自带的Mutex、RwLock实现互斥访问

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

* Dir

  pub trait Dir: NodeBase {

  ​    /// 获取给定名称的节点

  ​    fn lookup(&self, name: &ByteStr) -> Result<InodeId>;

  ​    /// 读取条目

  ​    /// 返回：

  ​    /// - Ok(Next Offset)

  ​    /// - Err(e) : 错误

  ​    fn read(&self, start_ofs: usize, callback: &mut ReadDirCallback) -> Result<usize>;

  ​    /// 在该目录下创建一个新文件，返回新创建的节点编号

  ​    fn create(&self, name: &ByteStr, nodetype: NodeType) -> Result<InodeId>;

  }

* Symlink

  pub trait Symlink: NodeBase {

  ​    /// 将符号链接的内容读入一个字符串

  ​    fn read(&self) -> ByteString;

  }

  内存索引节点结构，描述了文件的inode等信息，用于引用计数、同步互斥等操作。

* pub struct CacheHandle{

  ​    mountpt: usize,  //挂载点编号

  ​    inode: InodeId,   //inode编号

  ​    ptr: *const CachedNode,

  } 

  struct CachedNode{

  ​    refcount: AtomicUsize,  //引用计数

  ​    node: CacheNodeInt,    //inode，用枚举类型表示，有3种不同的inode

  } 

## 实验过程日志

详见 wiki 页面过程记录 [http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15](http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15)

## 实验总结

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