/**
 * @file debug.cc
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-29
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "debug.h"
#include "../types.h"

#include "../device_control.h"
#include "../tools/atou64.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// 命令名
char *commands[] = {
    "core?", // 输出核心数量
    "core",  // 指定调试核心
    "bp",    // 设置断点
    "rbp",   // 移除断点
    "lbp",   // 列出断点
    "stp",   // 步进执行
    "cont",  // 继续运行
    "start", // 开启当前CPU
};

// 最多分8个
u32 split_str(char *__str, char **tar, char spc);

/* 命令处理函数 */
char *db_core_help(char **arg);
char *db_core(char **arg);
char *db_bp(char **arg);
char *db_rbp(char **arg);
char *db_lbp(char **arg);
char *db_stp(char **arg);
char *db_cont(char **arg);
char *db_start(char **arg);

// 命令处理函数表
char *(*cmdhand[])(char **) = {
    db_core_help,
    db_core,
    db_bp,
    db_rbp,
    db_lbp,
    db_stp,
    db_cont,
    db_start,
};

i64 debugging_core = -1;

char *
debug(const char *command)
{
  char *cmd[8];
  memset(cmd, 0, 8 * sizeof(char *));
  u32 count = split_str((char *)command, cmd, ' ');
  for (u32 i = 0; i < sizeof(commands) / sizeof(char *); i++)
  {
    if (!strcmp(cmd[0], commands[i]))
    {
      return cmdhand[i](cmd);
    }
  }
}

char *db_core_help(char **arg)
{
  char *res = malloc(256);
  sprintf(res,
          "Totally %u cores.\n"
          "Core#%d is on debugging.\n"
          "(\'Core#-1\' means no core is on debugging.)\n",
          cmd_options.core, debugging_core);
  return res;
}

char *db_core(char **arg)
{
  debugging_core = atoi(arg[1]);
  if (debugging_core >= cmd_options.core)
  {
    char *res = malloc(26);
    sprintf(res, "Not an invalid core id.\n");
    return res;
  }
  char *res = malloc(1);
  res[0] = '\0';
  return res;
}

static u8
debugging_core_have_set()
{
  if (debugging_core == -1)
  {
    return 0;
  }
  else
  {
    return 1;
  }
}

#define TEST_IF_HAVE_DEBUGGING_CORE()  \
  if (!debugging_core_have_set())      \
  {                                    \
    char *res = malloc(27);            \
    sprintf(                           \
        res,                           \
        "No core is on debugging.\n"); \
    return res;                        \
  }

char *db_bp(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
  if (!arg[1])
  {
    char *res = malloc(22);
    sprintf(res, "An argument needed.\n");
    return res;
  }
  u64 bp = atou64(arg[1]);
  if (cores[debugging_core]->debug.bpcount == MAX_BP_COUNT)
  {
    char *res = malloc(30);
    sprintf(res, "Cannot add more breakpoint.\n");
    return res;
  }
  for (u32 i = 0; i < cores[debugging_core]->debug.bpcount; i++)
  {
    if (bp == cores[debugging_core]->debug.breakpoints[i])
    {
      char *res = malloc(28 + strlen(arg[1]));
      sprintf(res, "%s is already a breakpoint.\n", arg[1]);
      return res;
    }
  }
  cores[debugging_core]->debug.breakpoints
      [cores[debugging_core]->debug.bpcount++] = bp;
  char *res = malloc(1);
  res[0] = '\0';
  return res;
}

char *db_rbp(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
  if (!arg[1])
  {
    char *res = malloc(22);
    sprintf(res, "An argument needed.\n");
    return res;
  }
  u64 bp = atou64(arg[1]);
  for (u32 i = 0; i < cores[debugging_core]->debug.bpcount; i++)
  {
    if (cores[debugging_core]->debug.breakpoints[i] == bp)
    {
      memcpy(
          cores[debugging_core]->debug.breakpoints + i,
          cores[debugging_core]->debug.breakpoints + i + 1,
          sizeof(u64) * cores[debugging_core]->debug.bpcount - i - 1);
      cores[debugging_core]->debug.bpcount--;
      break;
    }
  }
  char *res = malloc(1);
  res[0] = '\0';
  return res;
}

char *db_lbp(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
  for (u32 i = 0; i < cores[debugging_core]->debug.bpcount; i++)
  {
    printf("%16x\n", (void *)(cores[debugging_core]->debug.breakpoints[i]));
  } // TODO 地址显示有些问题
  char *res = malloc(1);
  res[0] = '\0';
  return res;
}

char *db_stp(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
}

char *db_cont(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
}

char *db_start(char **arg)
{
  TEST_IF_HAVE_DEBUGGING_CORE();
}

u32 split_str(char *__str, char **tar, char spc)
{
  u32 count = 0;
  u8 flg = 1;
  u32 len = strlen(__str);
  for (u32 i = 0; i < len; i++)
  {
    if (flg)
    {
      tar[count++] = __str + i;
      flg = 0;
      if (count >= 8)
      {
        return count;
      }
    }
    if (__str[i] == spc)
    {
      __str[i] = '\0';
      flg = 1;
    }
  }
}
