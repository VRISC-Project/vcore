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
`core?`         |To ask number of cores.(See running information with additional parameter `a`.)
`core <id>`     |Set the core id you want to debug.
`bp <addr>`     |Set physical address breakpoint.(Every hit to this `addr` will stop vcore.)
`rbp <addr>`    |Remove breakpoint when `addr` is a breakpoint.
`lbp`           |List all breakpoints.
`stp <steps>`   |Run next `steps` instructions, and run one without `steps` parameter.
`cont`          |Continue until hit a breakpoint.
`start`         |start current cpu.

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
