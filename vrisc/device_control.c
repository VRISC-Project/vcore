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

u32 vrisc_device_id;
u8 *core_start_flags;

static char *generate_vrisc_dev_namestr(u32 vrisc_id)
{
  char *name;
  name = malloc(32);
  sprintf(name, "/dev/vrisc%d", vrisc_id);
  return name;
}

char *generate_intctl_namestr(u32 vrisc_id)
{
  char *intctl;
  intctl = malloc(32);
  sprintf(intctl, "vrisc-%d-intctl", vrisc_id);
  return intctl;
}

char *generate_ioctl_namestr(u32 vrisc_id)
{
  char *ioctl;
  ioctl = malloc(32);
  sprintf(ioctl, "vrisc-%d-ioctl", vrisc_id);
  return ioctl;
}

static vrisc_dev *generate_devstruc(u32 vrisc_id)
{
  // 产生shm的id字符串
  char *intctl, *ioctl;
  intctl = generate_intctl_namestr(vrisc_id);
  ioctl = generate_ioctl_namestr(vrisc_id);
  // 设备结构
  vrisc_dev *vdstruc = malloc(sizeof(vrisc_dev));
  vdstruc->intctl_shm_namestart = 0;
  vdstruc->ioctl_shm_namestart = strlen(intctl) + 1;
  u32 totlen = vdstruc->ioctl_shm_namestart + strlen(ioctl) + 1;
  vdstruc = realloc(vdstruc, sizeof(vrisc_dev) + totlen);
  vdstruc->totlen = totlen;
  strcpy(vdstruc->strs, intctl);
  strcpy(vdstruc->strs + vdstruc->ioctl_shm_namestart, ioctl);
  free(intctl);
  free(ioctl);
  return vdstruc;
}

void make_vrisc_device()
{
  FILE *vrisc_count_file = fopen("/dev/vrisc", "r");
  u32 vrisc_id;
  if (!vrisc_count_file)
  { // 系统中还没有运行的vrisc
    vrisc_count_file = fopen("/dev/vrisc", "w");
    u32 count = 1;
    fwrite(&count, 1, sizeof(u32), vrisc_count_file);
    fclose(vrisc_count_file);
    vrisc_id = 0;
  }
  else
  { // 有运行的vrisc
    fclose(vrisc_count_file);
    vrisc_count_file = fopen("/dev/vrisc", "r+");
    u32 count;
    fread(&count, 1, sizeof(u32), vrisc_count_file);
    vrisc_id = count;
    count++;
    fseek(vrisc_count_file, 0, SEEK_SET);
    fwrite(&count, 1, sizeof(u32), vrisc_count_file);
    fclose(vrisc_count_file);
  }
  // 创建"/dev/vrisc-{vrisc_id}"文件并写入相关信息
  char *dev_name_str = generate_vrisc_dev_namestr(vrisc_id);
  FILE *vrisc_devf = fopen(dev_name_str, "w");
  free(dev_name_str);
  vrisc_dev *vdstruc = generate_devstruc(vrisc_id);
  fwrite(vdstruc, 1, sizeof(vrisc_dev) + vdstruc->totlen, vrisc_devf);
  free(vdstruc);
  fclose(vrisc_devf);
  vrisc_device_id = vrisc_id;
}

void remove_vrisc_device()
{
  //移除/dev/vrisc-{device_id}文件
  char *dev_name_str = generate_vrisc_dev_namestr(vrisc_device_id);
  remove(dev_name_str);
  free(dev_name_str);
  //修改/dev/vrisc文件
  FILE *vrisc_count_file = fopen("/dev/vrisc", "r+");
  u32 count;
  fread(&count, 1, sizeof(u32), vrisc_count_file);
  count--;
  if (!count)
  {
    fclose(vrisc_count_file);
    remove("/dev/vrisc");
  }else{
    fseek(vrisc_count_file, 0, SEEK_SET);
    fwrite(&count, 1, sizeof(u32), vrisc_count_file);
    fclose(vrisc_count_file);
  }
}
