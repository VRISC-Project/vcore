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
#include <getopt.h>
#include <string.h>
#include <pthread.h>

#if defined(__linux__)
#include <unistd.h>
#include <signal.h>
#elif defined(_WIN32)
#include <windows.h>
#endif

#define __vrisc_main__

#include "memc.h"
#include "device_control.h"
#include "intctl.h"
#include "ioctl.h"
#include "core/vrisc.h"
#include "debug/debug.h"

#include "err.h"

struct options cmd_options;

pthread_t *core_threads;

void deal_with_cmdline(int argc, char **argv);

void load_bootloader();

void create_cores();

void *console(void *thr);

void join_cores();

void terminal_sigint(i32);

int main(int argc, char **argv)
{
  deal_with_cmdline(argc, argv);

  //设置终端发送的signal处理函数
  signal(SIGINT, terminal_sigint);

  create_memory(cmd_options.mem_size);

  load_bootloader();

  init_core();

  create_cores();

  make_vrisc_device();

  pthread_t cons = (pthread_t)console(0);

  pthread_t intctl, ioctl;
  pthread_create(&intctl, NULL, interrupt_global_controller, NULL);
  pthread_create(&ioctl, NULL, io_global_controller, NULL);

  join_cores();
  pthread_join(intctl, NULL);

  pthread_join(cons, NULL);

  remove_vrisc_device();
}

void terminal_sigint(i32 i)
{
  remove_vrisc_device();
  printf("\b\bvrisc terminated.\n");
  exit(0);
}

void join_cores()
{
  for (u64 u = 0; u < cmd_options.core; u++)
  {
    pthread_join(core_threads[u], NULL);
  }
}

void *
console(void *thr)
{
  // 通过一个传参使自己创建一个自己的线程，比较优雅
  if (!thr)
  {
    pthread_t id;
    if (pthread_create(&id, NULL, console, (void *)1))
    {
      printf("Failed to create console thread.\n");
      return (void *)CONSOLE_FAILED;
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
      {
        break;
      }
      cmd[cnt++] = ch;
    }
    printf(debug(cmd));
  }
}

void load_bootloader()
{
  if (!cmd_options.bootloader)
  {
    printf("fatal: No bootloader.\n");
    exit(NO_BOOTLOADER);
  }
  // 打开文件
  FILE *bl = fopen(cmd_options.bootloader, "r");
  if (!bl)
  {
    printf("The bootloader is unreachable.\n");
    exit(BOOTLOADER_BAD);
  }
  // 获取文件大小
  fseek(bl, 0, SEEK_END);
  u64 size = ftell(bl);
  if (size > cmd_options.mem_size)
  {
    printf("Memory is too small.\n");
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

  cores = malloc(cmd_options.core * sizeof(_core *));

  core_threads = malloc(cmd_options.core * sizeof(pthread_t *));
  for (u64 u = 0; u < cmd_options.core; u++)
  {
    if (pthread_create((pthread_t *)(core_threads + u), NULL, vrisc_core, (void *)u))
    {
      printf("Failed to create core#%d.\n", u);
      exit(CORE_FAILED);
    }
  }
}

void deal_with_cmdline(int argc, char **argv)
{
  cmd_options.bootloader = NULL;
  cmd_options.core = 0;
  cmd_options.extinsts = NULL;
  cmd_options.mem_size = 0;
  int opt;
  while ((opt = getopt(argc, argv, "m:c:b:e:t")) != -1)
  {
    switch (opt)
    {
    case 'm': // 内存
      cmd_options.mem_size = atoi(optarg);
      break;

    case 'c': // 核心数
      cmd_options.core = atoi(optarg);
      if (cmd_options.core == 0)
      {
        printf("fatal: The number of core is zero.\n");
        exit(CORE_NUM_IS_ZERO);
      }
      break;

    case 'b': // 引导程序
      cmd_options.bootloader = optarg;
      break;

    case 'e': // 扩展指令集路径
      cmd_options.extinsts = optarg;
      break;

    case 't': // 屏蔽内部时钟
      cmd_options.shield_internal_clock = 1;
      break;

    default:
      break;
    }
  }
}
