# Add RISCV Support for zCore  
by @SKTT1Ryze  
run zCore on qemu of riscv64  

## 进度
- [x] 搭建框架
- [x] 简易版内核对象框架
- [x] 内核对象
- [x] 内核对象测试模块
- [x] zircon 系统调用
- [x] zircon-loader
- [x] Memory (maybe)
- [x] Frame Allocator
- [ ] ......

## 分析
+ kernel-hal-bare 模块是唯一与硬件相关的代码
+ zCore 的模块化十分清晰，它将各个层次模块作为一个个 crate 进行调用。比如 kernel-hal 和 kernel-hal-bare。
+ kernel-hal 模块不依赖于其他模块，并为其他模块提供支持
+ 各个模块最终作为一个 crate 在 zCore 中被调用来构建一个 OS
+ 在硬件抽象层之下的 kernel-hal-bare 和 之上的各个层次都有架构相关的代码
+ 在这个项目中，原本在 zCore 中的模块在这里将不再作为一个个 crate，而是一个个 mod
+ 目前的大致方向是在我搭建的这个框架上一点点地构建 os，最终目标是使其能在 riscv 下的 qemu 上执行 zCore 的大部分功能。

## 目前遇到的障碍
+ 原版 zCore 需要用到zircon镜像 prebuilt/zircon/x64/userboot.so，这些镜像文件依赖于 Fuchsia 官方镜像，目前 Fuchsia 官方不支持 riscv，因此我目前无法获得适用于 zCore 的 riscv 上的 Fuchsia 镜像。
+ 我的理解是 zCore 现在可以在裸机或 QEMU 上跑 Fuchsia 原生用户程序，而 Fuchsia 官方目前只支持 x86 和 Arm 两种架构，再有由于 Fuchsia 是商业项目，因此他们可能不打算支持 riscv，这样的话想在 zCore 上跑 Fuchsia 用户程序的话道阻且长。


## 目前想到的解决办法
+ 写一个简单 Fuchsia 小程序替代 userboot.so，先看看运行效果
+ 利用 loader 层底下的实现暂时先重写一个简陋的 zircon-loader，先让整个框架能跑在 loader 层以上，能在上层环境输出，以达到验证底层代码正确性的效果
+ 参照 rCore-Tutorial，整个重写 zircon-loader，不执着于跑 Fuchsia 用户程序的思路，而是跑自己用 Rust 写的用户程序

## 正在进行：
再经过研究 zCore 源码和各种考虑之后，目前决定先简单重写一下 zircon-loader 层，具体来讲就是让重写的 loader 加载一个用 rustc 编译的面向 riscv 平台的 elf 文件，然后让进程去运行它，运行接口保留原来 zCore 的接口。实现方式将沿用 rCore-Tutorial 里面加载用户程序的方式。  
同时这样做也是暂时放弃了和 Fuchsia 的对接，后续会如何发展目前还得不出结论。  


