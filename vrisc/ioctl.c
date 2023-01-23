/**
 * @file ioctl.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-05
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "ioctl.h"

#include "device_control.h"
#include "err.h"
#include "core/pubstruc.h"

#include <stdio.h>
#include <stdlib.h>

#if defined(__linux__)
#include <unistd.h>
#include <time.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <fcntl.h>
#elif defined(_WIN32)
#include <windows.h>
#endif

ioctl_shm *io;

void *
io_global_controller(void *args)
{
  u32 ioctlf = shm_open(IOCTL_NAMESTR, O_CREAT | O_TRUNC | O_RDWR, 0777);
  if (ioctlf == -1)
  {
    printf("Shared memory allocation failed.\n");
    exit(SHM_FAILED);
  }
  if (ftruncate(ioctlf, sizeof(ioctl_shm)) == -1)
  {
    printf("Shared memory truncation failed.\n");
    exit(TRUNCATE_FAILED);
  }
  ioctl_shm *ctl = mmap(
      NULL, sizeof(ioctl_shm),
      PROT_READ | PROT_WRITE, MAP_SHARED,
      ioctlf, 0);
  close(ioctlf);

  while (core_start_flags[0])
  {
#if defined(__linux__)
    nanosleep(&(struct timespec){0, 1000000}, NULL);
#elif defined(_WIN32)
    Sleep(10);
#endif
    // 1端口是否写入
    if (ctl->output_heads[1] == ctl->output_tails[1])
    {
      continue;
    }
    u8 open_core = ctl->output[1][ctl->output_heads[1]];
    if (open_core >= cmd_options.core)
    {
      continue;
    }
    u8_lock_lock(ctl->output_locks[1]);
    ctl->output_heads[1]++;
    u8_lock_unlock(ctl->output_locks[1]);
    core_start_flags[open_core] = 1;
  }

  munmap(ctl, sizeof(ioctl_shm));
  shm_unlink(IOCTL_NAMESTR);
}
