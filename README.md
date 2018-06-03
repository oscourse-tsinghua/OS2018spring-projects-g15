# OS2018spring-projects-g15 Rucore
http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g15

Tsinghua OS course 2018 Project: Rucore

计54 乔一凡 杨国炜

## Introduction

Rucore 是一个使用 rust 实现的 x86_64 下的简单操作系统，并提供了相关的 IDE 硬盘驱动，PS/2 键盘鼠标驱动，能够在 qemu 虚拟环境中运行。

## Dependency

运行 `Rucore` 需要如下依赖：`nasm`, `grub-mkrescue`, `xorriso`, `qemu`, `rustc`, `rustup`, `cargo`, `xargo`, `x86_64-elf-gcc`, `x86_64-elf-binutils`。

由于 `Rust` 是一门相对年轻的语言，编译器更新很快，语言特性还在不断修改中，新版的 `rustc` 编译器可能无法顺利编译我们的代码。所以我们固定 `rustc` 版本为 `rustc 1.26.0-nightly (9c9424de5 2018-03-27)`

Rust 和 cargo 的安装：

```bash
curl https://sh.rustup.rs -sSf | sh
```

可以使用 `rustup` 进行 `rustc` 版本管理：

```bash
rustup default nightly-2018-03-27
rustup component add rust-src
```

Xargo 的安装：

```bash
cargo install xargo
```

交叉编译器安装：

在 Mac OS 下我们提供了 HomeBrew formula 安装 `x86_64-elf-gcc` 与 `x86_64-elf-binutils`:

```bash
brew tap ivanium/gcc_cross_compilers
brew install x86_64-elf-gcc
brwe install x86_64-elf-binutils
```

## Compile & Run

```bash
make
make run
```

## Documents

* 项目调研 PPT：[Rucore](https://github.com/oscourse-tsinghua/OS2018spring-projects-g15/blob/9ee1144f147f8732173b0f18f65748c31f6c93c3/docs/Middle_pre.pptx)
* 最终报告 PPT：[Final](https://github.com/oscourse-tsinghua/OS2018spring-projects-g15/blob/9ee1144f147f8732173b0f18f65748c31f6c93c3/docs/final.pptx)
* 报告：[report](https://github.com/oscourse-tsinghua/OS2018spring-projects-g15/blob/9ee1144f147f8732173b0f18f65748c31f6c93c3/docs/report.md)

## References

Rust OS

* 一个详尽的使用 Rust 开发 blog_os 的博客： [Write an OS in Rust](https://os.phil-opp.com/)
  * 目标平台为 x86_64
  * 目前有 bootloader，简单的内存管理模块，支持简单的中断机制
* Reenix: [Reenix: A Rust version of the Weenix OS](https://github.com/scialex/reenix)
  * 一个 Brown Univ 的同学的毕业设计，使用 Rust 重写了 weenix 教学 os
  * 不是纯 Rust 的结构，包含大量 C 代码
* Redox：[A Rust Operating System](https://github.com/redox-os/redox/)，一个目前最完善的 Rust based OS
  * 官方主页：[Redox](https://www.redox-os.org/)
* Stanford CS 140e 课程：[Stanford cs140e](https://web.stanford.edu/class/cs140e/)
  * 使用 Rust 在树莓派上实现 os

Rust Driver

* Reenix 中包含有简单的 ATA Driver：[Reenix Drivers](https://github.com/scialex/reenix/tree/vfs/kernel/drivers)
  * 实现上是使用 C 实现的，可以参考
* Rust OS：["Tifflin" Experimental Kernel](https://github.com/thepowersgang/rust_os/)，包含很多驱动支持，可以参考其实现
* Driver Helper Slides：[Brown CS 167 Drivers slides](http://cs.brown.edu/courses/cs167/projects/drivers-help.pdf)

实现中也参考了其他两个 rust 组的实现：

* G11 [https://github.com/wangrunji0408/RustOS](https://github.com/wangrunji0408/RustOS)

* G13 [https://github.com/oscourse-tsinghua/OS2018spring-projects-g13](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13)

