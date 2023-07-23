# vcore

vrisc架构虚拟处理器。

## 多处理器

支持多核，开启时默认启动core#0

## 特性

* 内存访问上，若访问一段连续内存（一般是读取指令或串指令集中使用）：连续内存不越过最小页框，连续读取；
  越过最小页框，页框边界两边的两段内存分别读取。

## 已知的bug

* debugger有概率无法启动。[已修复]
* loop指令相对跳转有问题。[已修复]

## 支持硬件平台

平台|状态
:-:|:-
Linux|正在开发
Windows|等待适配
Mac|等待适配
