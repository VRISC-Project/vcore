/**
 * @file intctl.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-02
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "intctl.h"

#include "device_control.h"
#include "err.h"
#include "core/pubstruc.h"
#include "core/vrisc.h"

#include <stdio.h>
#include <stdlib.h>

#if defined(__linux__)
#include <unistd.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <fcntl.h>
#elif defined(_WIN32)
#include <windows.h>
#endif

void *
interrupt_global_controller(void *args)
{
  // 打开共享内存fd
  u32 intctlf = shm_open(INTCTL_NAMESTR, O_CREAT | O_TRUNC | O_RDWR, 0777);
  if (intctlf == -1)
  {
    printf("Shared memory allocation failed.\n");
    exit(SHM_FAILED);
  }
  // 改变大小
  if (ftruncate(intctlf, sizeof(intctl_shm)) == -1)
  {
    printf("Shared memory truncation failed.\n");
    exit(TRUNCATE_FAILED);
  }
  // 映射共享内存
  intctl_shm *ctl = mmap(
      NULL, sizeof(intctl_shm),
      PROT_READ | PROT_WRITE, MAP_SHARED, // 标志位
      intctlf, 0                          // 文件描述符
  );
  close(intctlf);

  // 初始化intctl数据
  ctl->lock = 0;
  ctl->head = 0;
  ctl->tail = 0;

  // 控制循环
  while (core_start_flags[0])
  { // 只要判断core#0是否在运行即可，因为core#0一定第一个开启，最后关闭
#if defined(__linux__)
    usleep(700);
#elif defined(_WIN32)
    Sleep(1);
#endif

    if (ctl->head == ctl->tail)
    { // 空队列
      continue;
    }

    // 加入本地中断控制器
    u8_lock_lock(ctl->lock);
    u8 int_id = ctl->interrupt_queue[ctl->head].int_id;
    u8 mode = ctl->interrupt_queue[ctl->head].mode;
    ctl->head++;
    if (ctl->head == GLB_IR_QUEUE_LEN)
    {
      ctl->head = 0;
    }
    u8_lock_unlock(ctl->lock);
    if (mode == GLB_IR_MODE_SINGLE)
    { // 单处理器中断
      // 选择一个本地中断队列最短的加入
      u32 core_id;
      u32 shortest = 0xffffffff;
      for (u32 i = 0; i < cmd_options.core; i++)
      {
        u32 len;
        LOCAL_INTQUEUE_LEN(cores[i]->interrupt.controller, len);
        if (core_start_flags[i] && len < shortest)
        {
          core_id = i;
        }
      }
      intctl_addint(cores[core_id], int_id);
    }
    else if (mode == GLB_IR_MODE_GLOBAL)
    { // 全局中断
      for (u32 i = 0; i < cmd_options.core; i++)
      {
        if (core_start_flags[i])
        {
          intctl_addint(cores[i], int_id);
        }
      }
    }
  }

  // 取消映射，删除共享内存
  munmap(ctl, sizeof(intctl_shm));
  shm_unlink(INTCTL_NAMESTR);
}
