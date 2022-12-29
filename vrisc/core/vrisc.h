/**
 * @file core.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-17
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __vrisc_h__
#define __vrisc_h__

#include "../types.h"

struct options
{
  u64 mem_size;
  u8 core;
  char *bootloader; // 启动代码文件
  char *extinsts;   // 扩展指令集路径
};

extern char *exts_name[];

typedef struct core
{
  struct regs
  {
    /*
    64位：x
    32位：d
    16位：w
    8位：b
    */
    u64 x[16];
    /* flg
    ^0-equal
    ^1-bigger
    ^2-smaller
    ^3-zero
    ^4-signal
    ^5-overflow
    ^6-interrupt enable
    ^7-paging enable
    ^8-privilege level(effective when paging enabled)
      0-kernel
      1-user
    ^9-higher
    ^10-lower
     */
    u64 flg;
    u64 ip;
    u64 usb, ust; // 用户态栈帧
    u64 ksb, kst; // 内核态栈帧
    u64 kpt, upt; // 内核态和用户态页表指针
    u64 ivt;      // 中断向量表
    u64 scp;      // 系统调用指针
  } regs;

  struct interrupt
  {
    u8 triggered;

    u8 int_id;
#define IR_DIV_BY_ZERO 0
#define IR_NOT_EFFECTIVE_ADDRESS 1
#define IR_DEVICES 2
#define IR_INSTRUCTION_NOT_RECOGNIZED 3
#define IR_CLOCK 4
  } interrupt;

} _core;

#ifdef __vrisc_main__

extern u8 *core_start_flags;
extern _core **cores;

extern u64 (*instructions)(u8 *, _core *);

#endif

u64 vtaddr(u64 ip, _core *core);

void init_core();

void *vrisc_core(void *id);

#endif
