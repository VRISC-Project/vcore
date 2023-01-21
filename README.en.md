# vcore

An achivement of vrisc architecture.

## Usage

### build

```bash
mkdir build
cd build
cmake ../CMakeLists.txt
make vcore
cd ..
```

### run

```bash
sudo ./bin/vcore -m 1048576 -c 1 -b ./boot/boot.bin
```

vcore命令用法：

```bash
-m    Set memory size.
-c    Set number of cores.
-b    Set boot file.
-t    Use external clock; without this option, vcore
        uses default internal clock, frequences
        500Hz on linux and 200Hz on windows. 
-d    enable internal debugger.
```

## Internal debugger

Debugger's command:

command|explanation
:-:|:-
`core?`         |查询核心数量
`core <id>`     |设置debug的核心号
`bp <addr>`     |设置断点，虚拟机运行到物理地址`addr`处暂停
`rbp <addr>`    |移除断点，若`addr`为断点，则移除，不是则不变
`lbp`           |列出断点
`stp <steps>`   |继续运行`steps`个指令，没有参数默认steps=1
`cont`          |继续运行直到遇到断点
`start`         |使当前调试的cpu开启

## Internal clock

On `linux`, ticks 2ms.

On `windows`, ticks 5ms.

## Multi-processing

We supports multi-procession.

Defaultly start core#0.

## Device abstraction

We supports device `/dev/vrisc`， which can be
accessed by protocol library `libvcore-ioint.so`.

## Development Items

* debugger

## English edition explanation

This text might not be always the newest, the newest
edition is wrote in Chinese
(See [Chinese edition](README.md)).
