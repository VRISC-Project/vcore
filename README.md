# vrisc

极简的、可扩展的虚拟处理器架构。

* vcore是vrisc的一个实现

## vrisc架构详情见[文档](docs/index.md)

## 使用

### 构建

* vcore虚拟机

```bash
mkdir build
cd build
cmake ../CMakeLists.txt
make vcore
cd ..
```

* vas汇编器

```bash
make vas
```

### 运行

* vcore虚拟机

```bash
sudo ./bin/vcore -m 1048576 -c 1 -b ./boot/boot.bin
```

vcore命令用法：

```bash
-m    指定内存大小
-c    指定vrisc核心数量
-b    指定boot程序文件
-t    使用外部时钟；不使用此选项则使用内部的默认时钟，频率约为500Hz
```

* vas汇编器

```bash
./bin/vas -i 源文件 -o 目标文件 -m map文件（可选） 
```

> map文件：导出源文件中的符号在目标文件中的地址。

## debugger

调试器命令：

命令|说明
:-:|:-
`core?`         |查询核心数量
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

提供了设备`/dev/vriscx`，使用协议库访问设备。

## 开发事项

* 汇编器还需要重新开发，这个只是临时使用。
* debugger
