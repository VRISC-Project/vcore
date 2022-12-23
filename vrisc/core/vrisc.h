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
    ^9-
     */
    u64 flg;
    u64 ip;
    u64 usb, ust; // 用户态栈帧
    u64 ksb, kst; // 内核态栈帧
    u64 kpt, upt; // 内核态和用户态页表指针
    u64 ivt;      // 中断向量表
  } regs;

  struct interrupt
  {
    u8 triggered;

    u8 int_id;
  } interrupt;

} _core;

extern u8 *core_start_flags;

extern u64 (*instructions)(u8 *inst);

void init_core();

void *vrisc_core(void *id);

#endif
