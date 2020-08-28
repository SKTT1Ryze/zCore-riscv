# zCore 实习阶段性报告讲稿

# zCore on RISCV
+ 早上刚回到寝室
+ 下面开始我的报告

## 前言
+ 考虑到邀请到的老师可能不太了解 zCore
+ 王润基学长的链接
+ 问题来了，zCore 目前只支持 x86 架构

## 目标
+ 那么我这次实习选择的研究方向，就是 zCore 到 riscv 上的移植，具体来讲就是希望在 riscv 架构下的虚拟机 qemu 中跑 zCore

## 相关工作介绍
+ 在 zCore 的开发过程中，开发者已经考虑到将来可能要支持 riscv，架构相关的代码都做了标注，因此移植的工作很大一部分就是对这部分架构相关的代码添加 riscv 支持
+ 在 kernel-hal-bare 模块中，有一部分代码在 riscv 上已经实现了，这部分代码在 kernel-hal-bare/arch/riscv.rs 文件中
+ 最大的对移植工作的帮助来自于 rCore，rCore是王润基学长用 Rust 写的第一个 OS，它支持 riscv 架构，它里面的各种实现都会给 zCore 到 riscv 的移植工作有很大帮助
+ 刘丰源学长曾将 zCore 移植到了 mips 架构上，虽然不清楚进展到了何种程度，但相信可以给移植工作带来助力

## 实现方案
+ 移植工作的首先就是重新搭建一个 riscv 运行环境。为什么要重新搭建一个而不是使用 zCore 原有的运行环境呢？原因就在于 bootloader 上面。目前 zCore 和 rCore 在 x86 架构下的 bootloader 都是用的王润基学长写的 Rust 库 rboot，而 rboot 是不支持 riscv 的，而 rCore 在 riscv 架构下的 bootloader 是 OpenSBI。换句话说，x86 下的 bootloader 是面向 x86 写的 UEFI bootloader rboot，而在 riscv 下的 bootloader 应该用的是 OpenSBI。回到运行环境的问题上，目前 zCore 只支持 x86 ，它在裸机上跑的时候用的 bootloader 是 rboot，没有 OpenSBI 的支持，简单来说就是 zCore 的Makefile 没有写如何用 OpenSBI 作为 bootloader 去运行 riscv 下的 qemu。因此，为了让 zCore 在 riscv 的 qemu 上先简单运行起来，我们重新搭建一个新的基于 OpenSBI 的运行环境是比较适合的想法，而且我们在 rCore-Tutorial 中已经学到了如何搭建这个运行环境。
+ 有了 OpenSBI 这个 bootloader 和一些基础设施，我们就可以一步步将原 zCore 中的模块移植到 riscv 的运行环境中，回到前一点的做法，我们可以看到这样一来不仅省去了不少学习成本，而且在一步步搭建这个运行环境的过程中可以逐渐熟悉 zCore 的层次结构。
+ 我们在一步步搭建运行环境的过程中，我们需要知道我们的代码写得对不对
+ 先将一些架构相关的空给补了，用的 unimplemnted!宏来补
+ 当我们移植完 loader 层的代码的时候，我们希望调用 loader 层的函数去运行一些什么东西

## 具体实现方法
+ 这个简单来讲就是.....但会有些地方不一样，后面会讲到
+ 这么做没有什么特别的原因，硬要说的话就是这样做我能更容易地搭建起运行环境，节约时间成本
+ fake_test 就是假的单元测试，这里为了实现一些测试的函数，在原来的代码实现中添加了一些公有的成员函数
+ 为架构相关的代码添加 riscv 标注
+ 这个目前遇到了障碍还没解决

## 当前遇到的障碍和解决思路
原版 zCore 需要用到 zircon 镜像 prebuilt/zircon/x64/userboot.so，这些镜像文件依赖于 Fuchsia 官方镜像，目前 Fuchsia 官方不支持 riscv，因此我目前无法获得适用于 zCore 的 riscv 上的 Fuchsia 镜像。  
我的理解是 zCore 现在可以在裸机或 QEMU 上跑 Fuchsia 原生用户程序，而 Fuchsia 官方目前只支持 x86 和 Arm 两种架构，再有由于 Fuchsia 是商业项目，因此他们可能不打算支持 riscv，这样的话想在 zCore 上跑 Fuchsia 用户程序的话道阻且长。  
因为当前的运行环境是参照 rCore-Tutorial 搭建的一个简易的运行环境，还很简陋，虽然在这个月的工作中已经得到了一些改善，这些改善也会在后面讲到，但目前来看这个运行环境还需要不断的改进。  

写一个简单 Fuchsia 小程序替代 userboot.so，先看看运行效果  
利用 loader 层底下的实现暂时先重写一个简陋的 zircon-loader，先让整个框架能跑 loader 层以上，能在上层环境输出，以达到验证底层代码正确性的效果  
参照 rCore-Tutorial，整个重写 zircon-loader，不执着于跑 Fuchsia 用户程序的思路而是跑自己用 Rust 写的用户程序    
往 Linux 分支发展，现在已经把 linux 分支的模块移植到运行环境中了  
目前还在阅读 Makefile 的过程中。  

## 遇到的一点小插曲
在之前的运行环境中，如果代码使用了 kcounter 相关的功能话，会报链接错误，现在修改了 linker.ld 文件使得代码可以正确链接  
之前的运行环境中 memory 模块还是沿用 rCore-Tutorial 的 memory 模块，现在重写 memory 模块，实现了页帧分配器，并实现了 hal_frame_alloc，hal_frame_alloc_contiguous，hal_frame_dealloc 这三个在 kernel-hal 中定义但没有实现的函数，目前页帧分配已经可以正确使用。al_pt_map_kernel 函数还是 unimplement!  

## 后续工作的方向思路
+ 一项很系统的工作
+ RustSBI？
+ 毕竟代码都是用 Rust 写的，少不了阅读和写 Rust 代码，同时也是一种锻炼
+ 方便后人接手这个项目，主要是日志，后面也会整理一些阶段性概述文档

# 谢谢听讲！
谢谢老师，同学们听讲，我的报告到此结束，谢谢大家！