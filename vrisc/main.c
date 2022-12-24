/**
 * @file main.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-21
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <getopt.h>
#include <string.h>

#include <pthread.h>

#include "memc.h"
#include "core/vrisc.h"

#include "err.h"

struct options
{
  u64 mem_size;
  u8 core;
  char *bootloader; // 启动代码文件
  char *extinsts;   // 扩展指令集路径
} cmd_options;

pthread_t *cores;

void deal_with_cmdline(int argc, char **argv);

void load_bootloader();

void create_cores();

void *console(void *thr);

void join_cores();

int main(int argc, char **argv)
{
  deal_with_cmdline(argc, argv);

  create_memory(cmd_options.mem_size);

  load_bootloader();

  create_cores();

  pthread_join((pthread_t)console(0), NULL);

  join_cores();
}

void join_cores()
{
  for (u64 u = 0; u < cmd_options.core; u++)
  {
    pthread_join(cores[u], NULL);
  }
}

void *console(void *thr)
{
  // 通过一个传参使自己创建一个自己的线程，比较优雅
  if (!thr)
  {
    u64 id;
    if (pthread_create(&id, NULL, console, (void *)1))
    {
      printf("Failed to create console thread.");
      return CONSOLE_FAILED;
    }
    return (void *)id;
  }
  // 控制台
  while (1)
  {
    u8 cmd[128];
    u8 cnt = 0;
    u8 ch;
    memset(cmd, 0, 128);
    printf("vrisc >> ");
    while ((ch = getchar()) != '\n' && cmd[cnt - 1] != '\\')
    {
      if (cnt == 128)
        break;
      cmd[cnt++] = ch;
    }
    // TODO
    printf(cmd);
  }
}

void load_bootloader()
{
  // 打开文件
  FILE *bl = fopen(cmd_options.bootloader, "r");
  if (!bl)
  {
    printf("The bootloader is unreachable.");
    exit(BOOTLOADER_BAD);
  }
  // 获取文件大小
  fseek(bl, 0, SEEK_END);
  u64 size = ftell(bl);
  if (size > cmd_options.mem_size)
  {
    printf("Memory is too small.");
    exit(MEM_TOO_SMALL);
  }
  // 读取至内存
  fseek(bl, 0, SEEK_SET);
  fread(memory, 1, size, bl);
  fclose(bl);
}

void create_cores()
{
  core_start_flags = malloc(cmd_options.core * sizeof(u8));
  core_start_flags[0] = 1; // 首先开启核心0

  cores = malloc(cmd_options.core * sizeof(pthread_t *));
  for (u64 u = 0; u < cmd_options.core; u++)
  {
    if (pthread_create(cores + u, NULL, vrisc_core, (void *)u))
    {
      printf("Failed to create core#%d.\n", u);
      exit(CORE_FAILED);
    }
  }
}

void deal_with_cmdline(int argc, char **argv)
{
  int opt;
  while ((opt = getopt(argc, argv, "m:c:b:e")) != -1)
  {
    switch (opt)
    {
    case 'm': // 内存
      cmd_options.mem_size = atoi(optarg);
      break;

    case 'c': // 核心数
      cmd_options.core = atoi(optarg);
      break;

    case 'b': // 引导程序
      cmd_options.bootloader = optarg;
      break;

    case 'e': // 扩展指令集路径
      cmd_options.extinsts = optarg;
      break;

    default:
      break;
    }
  }
}
