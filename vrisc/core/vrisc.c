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

#include "base.h"

u8 *core_start_flags;

u64 (**instructions)(u8 *, _core *);

void init_core()
{
  instructions = malloc(256 * sizeof(u64(*)(u8 *, _core *)));

  memset(instructions, 0, 256 * sizeof(u64(*)(u8 *, _core *)));
  instructions[0] = (u64(*)(u8 *))NULL;
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
  instructions[27] = ep;
  instructions[28] = dp;
  instructions[29] = mv;
  instructions[30] = livt;
  instructions[31] = lkpt;
  instructions[32] = lupt;
  instructions[33] = lsrg;
  instructions[34] = ssrg;
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
    }
    else if (*(memory + core->regs.ip) == 35)
    { // destext
    }
  }
}
