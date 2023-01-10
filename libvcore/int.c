/**
 * @file int.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-10
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "../include/int.h"

#include "../include/types.h"
#include "../vrisc/err.h"

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

intctl_shm *link_int()
{
  // 打开共享内存fd
  u32 intctlf = shm_open(INTCTL_NAMESTR, O_TRUNC | O_RDWR, 0777);
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
}

void unlink_int(intctl_shm *ctl)
{
  munmap(ctl, sizeof(intctl_shm));
  shm_unlink(INTCTL_NAMESTR);
}
