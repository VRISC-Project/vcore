# vcore

vrisc虚拟处理器架构的实现。

## 使用

### 构建

```bash
mkdir build
cd build
cmake ../CMakeLists.txt
make vcore
cd ..
```

### 运行

```bash
sudo ./bin/vcore -m 1048576 -c 1 -b ./boot/boot.bin
```

vcore命令用法：

```bash
-m    指定内存大小
-c    指定vrisc核心数量
-b    指定boot程序文件
-t    使用外部时钟；不使用此选项则使用内部的默认时钟，频率约为500Hz
-d    开启debugger
```

## debugger

调试器命令：

命令|说明
:-:|:-
`core?`         |查询核心数量，加参数`a`可以查询每个核心的运行信息
`core <id>`     |设置debug的核心号
`bp <addr>`     |设置断点，虚拟机运行到物理地址`addr`处暂停
`rbp <addr>`    |移除断点，若`addr`为断点，则移除，不是则不变
`lbp`           |列出断点
`stp <steps>`   |继续运行`steps`个指令，没有参数默认steps=1
`cont`          |继续运行直到遇到断点
`start`         |使当前调试的cpu开启

## 内部计时器

在`linux`下，计时器周期为2ms。

在`windows`下，计时器周期为5ms。

## 多处理器

支持多核，开启时默认启动core#0

## 设备抽象

提供了设备`/dev/vrisc`，使用协议库`libvcore-ioint.so`访问设备。

## 开发事项

* debugger

## English edition

See [README.en.md](README.en.md).

This edition might not be always the newest, the newest
edition is this text.
