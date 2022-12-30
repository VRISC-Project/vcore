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

#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <dlfcn.h>
#include <pthread.h>
#include <time.h>

#include "base.h"

extern struct options cmd_options;

u8 *core_start_flags;
_core **cores;

u64 (**instructions)(u8 *, _core *);

void *ext_so;

char *exts_name[] = {
    (char *)"",
    (char *)"bae",
    (char *)"ave",
    (char *)"simde"};

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
  instructions[6] = _and;
  instructions[7] = _or;
  instructions[8] = not ;
  instructions[9] = _xor;
  instructions[10] = jc;
  instructions[11] = cc;
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
  instructions[38] = cut;
}

static char *generate_extpath(u64 id)
{
  if (!cmd_options.extinsts)
  {
    printf("fatal: No extending instruction set.\n");
    exit(NO_EXTINSTS);
  }
  char *path = malloc(256);
  char *idstr = malloc(22);
  u64 l = strlen(cmd_options.extinsts);
  memcpy(path, cmd_options.extinsts, l);
  path[l++] = '/';
  memcpy(path + l, "libvriscext", 11);
  l += 11;
  sprintf(idstr, "%d", id);
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

u64 vtaddr(u64 ip, _core *core)
{
  return ip;
}

void *clock_producer(void *args)
{
  u64 cid = *(u64 *)(((void **)args)[1]);
  _core *core = (_core *)(((void **)args)[0]);
  while (core_start_flags[cid])
  {
    // 500 tick/s
    usleep(1800);
    while (core->interrupt.triggered)
      ;
    core->interrupt.triggered = 1;
    core->interrupt.int_id = IR_CLOCK;
  }
}

void *vrisc_core(void *id)
{
  u64 cid = (u64)id;
  _core *core = malloc(sizeof(_core)); // 构造核心
  if (!core)
  {
    printf("Failed to create core#%d.\n", (u64)id);
    return (void *)CORE_FAILED;
  }
  cores[cid] = core; // 注册核心
  memset((void *)core, 0, sizeof(_core));
  printf("Created core#%d.\n", (u64)id);

  // 等待核心被允许开启
  while (!core_start_flags[cid])
  {
    usleep(500);
  }

  pthread_t clock_id;
  void *args[2] = {core, &cid};
  pthread_create(&clock_id, NULL, clock_producer, &args);

  while (core_start_flags[cid])
  {
    if (!*(memory + vtaddr(core->regs.ip, core)))
    { // nop
      while (!core->interrupt.triggered)
        usleep(1000);
      core->regs.ip++;
      continue;
    }
    if (*(memory + vtaddr(core->regs.ip, core)) == 34)
    { // initext
      if (ext_so)
      {
        dlclose(ext_so);
      }
      // 打开扩展指令集
      u64 extid = memory[vtaddr(core->regs.ip, core) + 1];
      ext_so = dlopen(generate_extpath(extid), RTLD_LAZY);
      // 两次认证
      if ((u64)dlsym(ext_so, "vriscext_id") != extid)
      { // id认证
        printf("Verification failed when loading extension.");
        exit(EXT_VERIFY_FAILED);
      }
      if (strcmp(exts_name[extid], (const char *)dlsym(ext_so, "vriscext_name")))
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
    else if (*(memory + vtaddr(core->regs.ip, core)) == 35)
    { // destext
      u64 inst_count = *((u64 *)dlsym(ext_so, "vriscext_inst_count"));
      u64 space_start = *((u64 *)dlsym(ext_so, "vriscext_space_start"));
      memset(instructions + space_start, 0, inst_count * sizeof(u64(*)(u8 *, _core *)));
      dlclose(ext_so);
      core->regs.ip += 1;
      continue;
    }
    if (!(instructions[memory[vtaddr(core->regs.ip, core)]]))
    { // 无效指令
      core->interrupt.triggered = 1;
      core->interrupt.int_id = IR_INSTRUCTION_NOT_RECOGNIZED;
    }
    if ((core->regs.flg & (1 << 6)) && core->interrupt.triggered)
    { // 如果发生中断先进入中断
      core->interrupt.triggered = 0;
      core->regs.x[0] = core->regs.ip;
      core->regs.x[1] = core->regs.flg;
      core->regs.flg &= ~(1 << 6); // 关闭flg^6.ie
      core->regs.flg &= ~(1 << 8); // 进入内核态
      core->regs.ip = *(u64 *)(memory + core->regs.ivt + core->interrupt.int_id * 8);
      continue;
    }
    // 正常执行指令
    core->regs.ip +=
        (*instructions[*(memory + vtaddr(core->regs.ip, core))])(
            memory + vtaddr(core->regs.ip, core), core);
  }
  free(core);
  pthread_join(clock_id, NULL);
}
