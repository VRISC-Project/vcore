/**
 * @file core.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-17
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "vrisc.h"

#include "../types.h"
#include "../err.h"
#include "../memc.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <dlfcn.h>

#include "base.h"

struct options
{
  u64 mem_size;
  u8 core;
  char *bootloader; // 启动代码文件
  char *extinsts;   // 扩展指令集路径
};

extern struct options cmd_options;

u8 *core_start_flags;

u64 (**instructions)(u8 *, _core *);

void *ext_so;

void init_core()
{
  instructions = malloc(256 * sizeof(u64(*)(u8 *, _core *)));

  memset(instructions, 0, 256 * sizeof(u64(*)(u8 *, _core *)));
  instructions[0] = (u64(*)(u8 *, _core *))NULL;
  instructions[1] = add;
  instructions[2] = sub;
  instructions[3] = inc;
  instructions[4] = dec;
  instructions[5] = cmp;
  instructions[6] = and;
  instructions[7] = or ;
  instructions[8] = not ;
  instructions[9] = xor;
  instructions[10] = jc;
  instructions[11] = c;
  instructions[12] = r;
  instructions[13] = ir;
  instructions[14] = sysc;
  instructions[15] = sysr;
  instructions[16] = loop;
  instructions[17] = chl;
  instructions[18] = chr;
  instructions[19] = rol;
  instructions[20] = ror;
  instructions[21] = ldi;
  instructions[22] = ldm;
  instructions[23] = stm;
  instructions[24] = ei;
  instructions[25] = di;
  instructions[26] = ep;
  instructions[27] = dp;
  instructions[28] = mv;
  instructions[29] = livt;
  instructions[30] = lkpt;
  instructions[31] = lupt;
  instructions[32] = lsrg;
  instructions[33] = ssrg;
  instructions[36] = in;
  instructions[37] = out;
}

static char *generate_extpath(u64 id)
{
  char *path = malloc(256);
  char *idstr = malloc(22);
  u64 l = strlen(cmd_options.extinsts);
  memcpy(path, cmd_options.extinsts, l);
  path[l++] = '/';
  memcpy(path + l, "libvriscext", 11);
  l += 11;
  itoa(id, idstr, 10);
  memcpy(path + l, idstr, strlen(idstr));
  l += strlen(idstr);
  path[l++] = '.';
  memcpy(path + l, exts_name[id], strlen(exts_name[id]));
  l += strlen(exts_name[id]);
  memcpy(path + l, ".so", 3);
  l += 3;
  path[l++] = 0;
  free(idstr);
  return path;
}

void *vrisc_core(void *id)
{
  u64 cid = (u64)id;
  _core *core = malloc(sizeof(_core));
  if (!core)
  {
    printf("Failed to create core#%d.\n", (u64)id);
    return CORE_FAILED;
  }
  memset((void *)core, 0, sizeof(_core));
  printf("Created core#%d.\n", (u64)id);

  // 等待核心被允许开启
  while (!core_start_flags[cid])
  {
    usleep(500);
  }

  while (core_start_flags[cid])
  {
    if (!*(memory + core->regs.ip))
    { // nop
      while (!core->interrupt.triggered)
        usleep(500);
      core->regs.ip++;
      continue;
    }
    if (*(memory + core->regs.ip) == 34)
    { // initext
      if (ext_so)
      {
        dlclose(ext_so);
      }
      // 打开扩展指令集
      u64 extid = memory[core->regs.ip + 1];
      ext_so = dlopen(generate_extpath(extid), RTLD_LAZY);
      // 两次认证
      if ((u64)dlsym(ext_so, "vriscext_id") != extid)
      { // id认证
        printf("Verification failed when loading extension.");
        exit(EXT_VERIFY_FAILED);
      }
      if (strcmp(exts_name, (const char *)dlsym(ext_so, "vriscext_name")))
      { // 名字认证
        printf("Verification failed when loading extension.");
        exit(EXT_VERIFY_FAILED);
      }
      u64 inst_count = *((u64 *)dlsym(ext_so, "vriscext_inst_count"));
      u64 space_start = *((u64 *)dlsym(ext_so, "vriscext_space_start"));
      u64 (**ext_insts)(u8 *, _core *);
      ext_insts = (u64(**)(u8 *, _core *))dlsym(ext_so, "vriscext_instructions");
      memcpy(instructions + space_start, ext_insts, inst_count * sizeof(u64(*)(u8 *, _core *)));
      core->regs.ip += 2;
      continue;
    }
    else if (*(memory + core->regs.ip) == 35)
    { // destext
      u64 inst_count = *((u64 *)dlsym(ext_so, "vriscext_inst_count"));
      u64 space_start = *((u64 *)dlsym(ext_so, "vriscext_space_start"));
      memset(instructions + space_start, 0, inst_count * sizeof(u64(*)(u8 *, _core *)));
      dlclose(ext_so);
      core->regs.ip += 1;
      continue;
    }
    if (!(instructions[memory[core->regs.ip]]))
    { // 无效指令
      core->interrupt.triggered = 1;
      core->interrupt.int_id = IR_INSTRUCTION_NOT_RECOGNIZED;
    }
    if (core->interrupt.triggered)
    { // 如果发生中断先进入中断
      core->regs.x[0] = core->regs.ip;
      core->regs.x[1] = core->regs.flg;
      core->regs.flg &= ~(1 << 6); // 关闭flg^6.ie
      core->regs.flg &= ~(1 << 8); // 进入内核态
      core->regs.ip = memory[core->regs.ivt + core->interrupt.int_id * 8];
      continue;
    }
    // 正常执行指令
    core->regs.ip += (*instructions[core->regs.ip])(core->regs.ip, core);
  }
}
