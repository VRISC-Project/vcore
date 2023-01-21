/**
 * @file device_control.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-01
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "device_control.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

_core **cores;

u8 *core_start_flags;

void make_vrisc_device()
{
  FILE *vrisc_device = fopen("/dev/vrisc", "r");
  if (vrisc_device)
  {
    printf("A vrisc vCPU is running, waiting...\n");
    while (vrisc_device = fopen("/dev/vrisc", "r"))
    { // 等待直到没有此文件存在
      // 休眠0.1s
      usleep(100000);
    }
  }
  vrisc_device = fopen("/dev/vrisc", "w");
  pid_t pid = getpid();
  fwrite(&pid, 1, sizeof(pid_t), vrisc_device);
  fclose(vrisc_device);
}

void remove_vrisc_device()
{
  remove("/dev/vrisc");
}
