# Add RISCV Support for zCore  
by @SKTT1Ryze  
run zCore on qemu of riscv64  

## 进度
- [x] 运行环境
- [x] kernel-hal
- [x] kernel-hal-bare
- [x] zircon-object
- [x] zircon-syscall
- [x] zircon-loader
- [x] Memory (maybe)
- [x] Frame Allocator
- [x] linux-object
- [x] linux-syscall
- [x] linux-loader
- [x] fake_test
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
+ 往 Linux 分支发展  


## 做了哪些：
+ 整个运行环境的搭建
+ 用 kernel-hal 中 unimplement 的函数中加上 kernel-hal-bare 中实现的函数链接，让代码转到 kernel-bare 中执行
+ 重写 memory 模块，实现了页帧分配器，并实现了 hal_frame_alloc，hal_frame_alloc_contiguous，hal_frame_dealloc 这三个在 kernel-hal 中定义但没有实现的函数，目前页帧分配已经可以正确使用。al_pt_map_kernel 函数还是 unimplement!
+ 将 print! 和 println! 宏从 console.rs 转到了 logging.rs，更好地对接原 zCore 的实现
+ 之前如果代码使用了 kcounter 相关的功能话，会报链接错误，现在修改了 linker.ld 文件使得代码可以正确链接
+ 在 thread.start 函数中原本没有在 riscv 平台下对 context 的处理，我这里为其加上了一些简单的实现，正确与否还得在后续的开发中观察和修改
+ 因为 no_std 环境里面实现单元测试十分麻烦，因此我写了一个 fake_test mod 用于代码测试
+ 为某些实现增加了一些共用的成员函数，以便我可以写测试代码


## 正在进行：
~~再经过研究 zCore 源码和各种考虑之后，目前决定先简单重写一下 zircon-loader 层，具体来讲就是让重写的 loader 加载一个用 rustc 编译的面向 riscv 平台的 elf 文件，然后让进程去运行它，运行接口保留原来 zCore 的接口。实现方式将沿用 rCore-Tutorial 里面加载用户程序的方式.~~  
~~但是上述功能的实现需要我理解 zCore 里面文件系统是怎么运作的，目前我对于着一块还是比较陌生。~~  
~~同时这样做也是暂时放弃了和 Fuchsia 的对接，后续会如何发展目前还得不出结论。  
往后要复习考试，可能进度会停滞一些了，在复习期间会抽时间给 zCore 加一点单元测试，同时也能使我更全面地理解 zCore。~~  

~~zCore 有两个分支，一个是对接 Fuchsia 的 zircon 分支，另外一个是在上面跑 linux 程序的 linux 分支。~~  
~~经过在 22 号的会议上的讨论，向勇老师提出既然对接 Fuchsia 行不通，那么就可以考虑在inux 分支上移植到 riscv，因为在 rCore 中已经有功移植到 riscv 的成功先例。因此当前正在进行的是搭建 linux 分支的运行环境，然后希望在月底之前可以在 rust_main 函数中调用 linux_loader 中的函数。  ~~  

kernel-hal-bare 和 kernel-hal 层的 riscv 实现。  




