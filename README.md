# vcore

vrisc架构虚拟处理器。

## 开始使用

* 使用`vrom.img`作为启动镜像启动一个单核虚拟机:

```bash
cargo run --bin vcore --release --features debugger -- -m 1048576 -v vrom.img -d
```

* 查看`vcore`命令使用方法:

```bash
cargo run --bin vcore --release --features debugger -- -h
```

## 说明

`vrom.img`是vcore可运行的启动代码二进制镜像，由VRISC-Project中的vas汇编器从`vroms/test1.vas`生成。

## 多处理器

支持多核，开启时默认启动core#0。

## 开发计划

* [0.2]
  * 重构vcore debugger，使debugger交互性更强，命令输入更舒适快捷。[v]
  * 实现总线输入输出指令`in`和`out`。
  * 加入对多核的完整支持，以前的版本只能在debugger中启动其它核心，新版本中可以通过机器指令访问总线打开核心。

## 特性

* 内存访问上，若访问一段连续内存（一般是读取指令或串指令集中使用）：连续内存不越过最小页框，连续读取；
  越过最小页框，页框边界两边的两段内存分别读取。
* `initext`和`destext`指令暂时无效。
* 在开启vcore debugger的版本中，在step模式下，执行nop指令时查看寄存器，`ip`寄存器会指向下一个指令。

## 支持操作系统平台

平台|状态
:-:|:-
Linux|0.2.0已发布
Windows|正在开发
Mac|等待适配
