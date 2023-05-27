# vcore

vrisc架构虚拟处理器。

## 使用

### 运行

vcore命令用法：

```bash
-m    指定内存大小
-c    指定vrisc核心数量
-b    指定boot程序文件
-t    使用外部时钟；不使用此选项则使用内部的默认时钟，频率约为?Hz
-d    开启debugger
```

## 多处理器

支持多核，开启时默认启动core#0

## 支持硬件平台

平台|状态
:-:|:-
Linux|正在开发
Windows|等待适配
Mac|等待适配
