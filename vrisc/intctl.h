/**
 * @file intctl.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-02
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __intctl_h__
#define __intctl_h__

#include "types.h"
#include "core/pubstruc.h"

typedef struct intctl_shm
{
  u8 lock; // 进程锁，读取时不用解锁，写入需要锁
#define GLB_IR_QUEUE_LEN 4096
  struct interrupt_signal
  {
    u8 int_id;
#define GLB_IR_MODE_GLOBAL 0
#define GLB_IR_MODE_SINGLE 1
    u8 mode;
  } interrupt_queue[GLB_IR_QUEUE_LEN];
  u16 head, tail;
} intctl_shm;

/* 中断控制线程 */
void *interrupt_global_controller(void *);

#endif
