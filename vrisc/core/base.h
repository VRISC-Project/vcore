/**
 * @file base.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-23
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __base_h__
#define __base_h__

#include "../types.h"
#include "../ioctl.h"
#include "vrisc.h"

extern u8 *memory;

u64 add(u8 *inst, _core *core);
u64 sub(u8 *inst, _core *core);
u64 inc(u8 *inst, _core *core);
u64 dec(u8 *inst, _core *core);
u64 cmp(u8 *inst, _core *core);
u64 _and (u8 * inst, _core *core);
u64 _or (u8 * inst, _core *core);
u64 not(u8 * inst, _core *core);
u64 _xor (u8 * inst, _core *core);
u64 jc(u8 *inst, _core *core);
u64 cc(u8 *inst, _core *core);
u64 r(u8 *inst, _core *core);
u64 ir(u8 *inst, _core *core);
u64 sysc(u8 *inst, _core *core);
u64 sysr(u8 *inst, _core *core);
u64 loop(u8 *inst, _core *core);
u64 chl(u8 *inst, _core *core);
u64 chr(u8 *inst, _core *core);
u64 rol(u8 *inst, _core *core);
u64 ror(u8 *inst, _core *core);
u64 ldi(u8 *inst, _core *core);
u64 ldm(u8 *inst, _core *core);
u64 stm(u8 *inst, _core *core);
u64 ei(u8 *inst, _core *core);
u64 di(u8 *inst, _core *core);
u64 ep(u8 *inst, _core *core);
u64 dp(u8 *inst, _core *core);
u64 mv(u8 *inst, _core *core);
u64 livt(u8 *inst, _core *core);
u64 lkpt(u8 *inst, _core *core);
u64 lupt(u8 *inst, _core *core);
u64 lsrg(u8 *inst, _core *core);
u64 ssrg(u8 *inst, _core *core);
u64 in(u8 *inst, _core *core);
u64 out(u8 *inst, _core *core);
u64 cut(u8 *inst, _core *core);
u64 icut(u8 *inst, _core *core);
u64 iexp(u8 *inst, _core *core);

#endif
