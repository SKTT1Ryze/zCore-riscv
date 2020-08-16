# Add RISCV Support for zCore  
by @SKTT1Ryze  
run zCore on qemu of riscv64  

## 进度
- [x] 搭建框架
- [x] 简易版内核对象框架
- [x] 内核对象
- [x] 内核对象测试模块
- [ ] ......

## 分析
+ kernel-hal-bare 模块是唯一与硬件相关的代码
+ zCore 的模块化十分清晰，它将各个层次模块作为一个个 crate 进行调用。比如 kernel-hal 和 kernel-hal-bare。
+ kernel-hal 模块不依赖于其他模块，并为其他模块提供支持
+ 各个模块最终作为一个 crate 在 zCore 中被调用来构建一个 OS
+ 在硬件抽象层之下的 kernel-hal-bare 和 之上的各个层次都有架构相关的代码
+ 在这个项目中，原本在 zCore 中的模块在这里将不再作为一个个 crate，而是一个个 mod
+ 目前的大致方向是在我搭建的这个框架上一点点地构建 os，最终目标是使其能在 riscv 下的 qemu 上执行 zCore 的大部分功能。
