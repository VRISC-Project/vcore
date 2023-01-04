# vRisc设备

## 设备抽象

### Linux

提供`/dev/vrisc`设备文件以及`libvrisc.so`接口库访问虚拟CPU。

由于系统中可能运行多个vrisc，`/dev/vrisc`文件中只有一个32位无符号整数表示正在运行的vrisc数量。

真正提供访问接口的设备文件是`/dev/vrisc-x`，其中`x`表示第x个vrisc虚拟CPU。

此文件的数据结构见[`vrisc/core/pubstruc.h::vrisc_dev`](../vrisc/core/pubstruc.h)。
