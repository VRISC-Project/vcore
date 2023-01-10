/**
 * @file io.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-09
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "../include/io.h"

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

ioctl_shm *link_io()
{
  u32 ioctlf = shm_open(IOCTL_NAMESTR, O_TRUNC | O_RDWR, 0777);
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
  return ctl;
}

void unlink_io(ioctl_shm *ctl)
{
  munmap(ctl, sizeof(ioctl_shm));
  shm_unlink(IOCTL_NAMESTR);
}
