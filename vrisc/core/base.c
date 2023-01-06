/**
 * @file base.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-23
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "base.h"

extern struct options cmd_options;

#define USER_MODE_CHECK(core) (core->regs.flg & (1 << 8))

#define RPL_MODE_CHECK(core)                     \
  {                                              \
    if (USER_MODE_CHECK(core))                   \
    {                                            \
      intctl_addint(core, IR_PERMISSION_DENIED); \
    }                                            \
  }

u64 add(u8 *inst, _core *core)
{
  u8 srcs = inst[1];
  u8 tar = inst[2];
  core->regs.x[tar] = core->regs.x[srcs % 16];
  srcs >>= 4;
  core->regs.x[tar] += core->regs.x[srcs];
  if (core->regs.x[tar])
  {
    core->regs.flg &= ~(1 << 3);
  }
  else
  {
    core->regs.flg |= 1 << 3;
  }
  return 3;
}

u64 sub(u8 *inst, _core *core)
{
  u8 srcs = inst[1];
  u8 tar = inst[2];
  core->regs.x[tar] = core->regs.x[srcs % 16];
  srcs >>= 4;
  core->regs.x[tar] -= core->regs.x[srcs];
  if (core->regs.x[tar])
  {
    core->regs.flg &= ~(1 << 3);
  }
  else
  {
    core->regs.flg |= 1 << 3;
  }
  return 3;
}

u64 inc(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  core->regs.x[tar]++;
  if (!core->regs.x[tar])
  {
    core->regs.flg |= 1 << 5;
  }
  else
  {
    core->regs.flg &= ~(1 << 5);
  }
  return 2;
}

u64 dec(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  core->regs.x[tar]--;
  if (core->regs.x[tar])
  {
    core->regs.flg &= ~(1 << 3);
  }
  else
  {
    core->regs.flg |= 1 << 3;
  }
  return 2;
}

u64 cmp(u8 *inst, _core *core)
{
  u8 srcs = inst[1];
  u64 r1 = core->regs.x[srcs % 16];
  srcs >>= 4;
  u64 r2 = core->regs.x[srcs];
  if (r1 == r1)
  {
    core->regs.flg |= 1;
  }
  else
  {
    core->regs.flg &= ~1;
  }
  if (r1 > r2)
  {
    core->regs.flg |= 1 << 1;
  }
  else
  {
    core->regs.flg &= ~(1 << 1);
  }
  if (r1 < r2)
  {
    core->regs.flg |= 1 << 2;
  }
  else
  {
    core->regs.flg &= ~(1 << 2);
  }
  if ((i64)r1 > (i64)r2)
  {
    core->regs.flg |= 1 << 9;
  }
  else
  {
    core->regs.flg &= ~(1 << 9);
  }
  if ((i64)r1 > (i64)r2)
  {
    core->regs.flg |= 1 << 10;
  }
  else
  {
    core->regs.flg &= ~(1 << 10);
  }
  return 2;
}

u64 _and(u8 *inst, _core *core)
{
  u8 src = inst[1];
  u8 tar = inst[2];
  core->regs.x[tar] = core->regs.x[src % 16];
  src >>= 4;
  core->regs.x[tar] &= core->regs.x[src];
  if (!core->regs.x[tar])
  {
    core->regs.flg |= 1 << 3;
  }
  else
  {
    core->regs.flg &= ~(1 << 3);
  }
  return 3;
}

u64 _or(u8 *inst, _core *core)
{
  u8 src = inst[1];
  u8 tar = inst[2];
  core->regs.x[tar] = core->regs.x[src % 16];
  src >>= 4;
  core->regs.x[tar] |= core->regs.x[src];
  if (!core->regs.x[tar])
  {
    core->regs.flg |= 1 << 3;
  }
  else
  {
    core->regs.flg &= ~(1 << 3);
  }
  return 3;
}

u64 not(u8 * inst, _core *core)
{
  u8 src = inst[1];
  u8 tar = src >> 4;
  src >>= 4;
  core->regs.x[tar] = ~core->regs.x[src];
  if (!core->regs.x[tar])
  {
    core->regs.flg |= 1 << 3;
  }
  else
  {
    core->regs.flg &= ~(1 << 3);
  }
  return 2;
}

u64 _xor(u8 *inst, _core *core)
{
  u8 src = inst[1];
  u8 tar = inst[2];
  core->regs.x[tar] = core->regs.x[src % 16];
  src >>= 4;
  core->regs.x[tar] ^= core->regs.x[src];
  if (!core->regs.x[tar])
  {
    core->regs.flg |= 1 << 3;
  }
  else
  {
    core->regs.flg &= ~(1 << 3);
  }
  return 3;
}

static u8 condition(u8 cond, u64 flg)
{
  if (!cond)
  {
    return 1;
  }
  else if (cond == 1)
  { // equal
    if (!(flg & 1))
    {
      return 1;
    }
  }
  else if (cond == 2)
  { // bigger
    if (!(flg & (1 << 1)))
    {
      return 1;
    }
  }
  else if (cond == 3)
  { // smaller
    if (!(flg & (1 << 2)))
    {
      return 1;
    }
  }
  else if (cond == 4)
  { // n-equal
    if (flg & 1)
    {
      return 1;
    }
  }
  else if (cond == 5)
  { // n-bigger
    if (flg & (1 << 1))
    {
      return 1;
    }
  }
  else if (cond == 6)
  { // n-smaller
    if (flg & (1 << 2))
    {
      return 1;
    }
  }
  else if (cond == 7)
  { // higher
    if (!(flg & (1 << 9)))
    {
      return 1;
    }
  }
  else if (cond == 8)
  { // lower
    if (!(flg & (1 << 10)))
    {
      return 1;
    }
  }
  else if (cond == 9)
  { // n-higher
    if (flg & (1 << 9))
    {
      return 1;
    }
  }
  else if (cond == 10)
  { // n-lower
    if (flg & (1 << 10))
    {
      return 1;
    }
  }
  else if (cond == 11)
  { // overflow
    if (flg & (1 << 5))
    {
      return 1;
    }
  }
  else if (cond == 12)
  { // zero
    if (flg & (1 << 3))
    {
      return 1;
    }
  }
  return 0;
}

u64 jc(u8 *inst, _core *core)
{
  u8 opl = inst[1];
  u8 cond = opl >> 4;
  opl %= 16;
  if (opl == 0)
    opl = 2;
  else if (opl == 1)
    opl = 4;
  else if (opl == 2)
    opl = 8;
  u64 tar = *((u64 *)(inst + 2));
  if (opl == 2)
  {
    tar = (u16)tar;
  }
  else if (opl == 4)
  {
    tar = (u32)tar;
  }
  if (condition(cond, core->regs.flg))
  {
    core->regs.ip = tar;
    core->ipbuff_need_flush = 1;
    return 0;
  }
  return opl + 2;
}

u64 cc(u8 *inst, _core *core)
{
  u8 opl = inst[1];
  u8 cond = opl >> 4;
  opl %= 16;
  if (opl == 0)
    opl = 2;
  else if (opl == 1)
    opl = 4;
  else if (opl == 2)
    opl = 8;
  u64 tar = *((u64 *)(inst + 2));
  if (opl == 2)
  {
    tar = (u16)tar;
  }
  else if (opl == 4)
  {
    tar = (u32)tar;
  }
  if (condition(cond, core->regs.flg))
  {
    core->regs.x[0] = core->regs.ip + opl + 2;
    core->regs.ip = tar;
    core->ipbuff_need_flush = 1;
  }
  return opl + 2;
}

u64 r(u8 *inst, _core *core)
{
  core->regs.ip = core->regs.x[0];
  core->ipbuff_need_flush = 1;
  return 0;
}

volatile u64 ir(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 mod = inst[1];
  if (!mod)
  {
    core->regs.ip = 0;
    core->regs.flg = 0;
    return 0;
  }
  core->regs.flg = core->regs.x[1];
  core->regs.ip = core->regs.x[0];
  core->ipbuff_need_flush = 1;
  if (mod == 1)
  { // retry模式
    return -core->incr;
  }
  else
  { // skip模式
    return 0;
  }
}

u64 sysc(u8 *inst, _core *core)
{
  core->regs.x[0] = core->regs.ip;
  core->regs.flg |= 1 << 8;
  core->regs.ip = core->regs.scp;
  core->ipbuff_need_flush = 1;
  return 0;
}

u64 sysr(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  core->regs.flg &= ~(1 << 8);
  core->regs.ip = core->regs.x[0];
  core->ipbuff_need_flush = 1;
  return 1;
}

u64 loop(u8 *inst, _core *core)
{
  u8 cond = inst[1];
  if (core->regs.x[cond])
  {
    core->regs.x[cond]--;
    i32 imm = *(i32 *)(inst + 2);
    i64 imm64 = imm;
    return *((u64 *)(&imm64));
  }
  else
  {
    return 6;
  }
}

u64 chl(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u64 bits = core->regs.x[src];
  core->regs.x[tar] <<= bits;
  return 2;
}

u64 chr(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u64 bits = core->regs.x[src];
  core->regs.x[tar] >>= bits;
  return 2;
}

u64 rol(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u64 bits = core->regs.x[src];
  u64 r = core->regs.x[tar];
  u64 re = r << bits;
  re |= r >> (63 - bits);
  core->regs.x[tar] = re;
  return 2;
}

u64 ror(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u64 bits = core->regs.x[src];
  u64 r = core->regs.x[tar];
  u64 re = r >> bits;
  re |= r << (63 - bits);
  core->regs.x[tar] = re;
  return 2;
}

u64 ldi(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 opl = tar % 16;
  tar >>= 4;
  u64 imm;
  if (opl == 0)
  {
    opl = 1;
    imm = *(u8 *)(inst + 2);
  }
  else if (opl == 1)
  {
    opl = 2;
    imm = *(u16 *)(inst + 2);
  }
  else if (opl == 2)
  {
    opl = 4;
    imm = *(u32 *)(inst + 2);
  }
  else if (opl == 3)
  {
    opl = 8;
    imm = *(u64 *)(inst + 2);
  }
  core->regs.x[tar] = imm;
  return opl + 2;
}

u64 ldm(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u64 addr = core->regs.x[src];
  if (!vtaddr(addr, core, 1))
  {
    return 0;
  }
  core->regs.x[tar] = *(u64 *)(memory + vtaddr(addr, core, 0));
  return 2;
}

u64 stm(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  u8 addr = core->regs.x[tar];
  if (!vtaddr(addr, core, 1))
  {
    return 0;
  }
  *(u64 *)(memory + vtaddr(addr, core, 0)) = core->regs.x[src];
  return 2;
}

u64 ei(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  core->regs.flg |= 1 << 6;
  return 1;
}

u64 di(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  core->regs.flg &= ~(1 << 6);
  return 1;
}

u64 ep(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  core->regs.flg |= 1 << 7;
  return 1;
}

u64 dp(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  core->regs.flg &= ~(1 << 7);
  return 1;
}

u64 mv(u8 *inst, _core *core)
{
  u8 tar = inst[2];
  u8 src = tar % 16;
  tar >>= 4;
  u8 flg = inst[1];
  u64 s, t;
  if (flg & 2)
  {
    if (!vtaddr(core->regs.x[src], core, 1))
    {
      return 0;
    }
    s = *(u64 *)(memory + vtaddr(core->regs.x[src], core, 0));
  }
  else
  {
    s = core->regs.x[src];
  }

  if (flg & 1)
  {
    if (!vtaddr(core->regs.x[tar], core, 1))
    {
      return 0;
    }
    *(u64 *)(memory + vtaddr(core->regs.x[tar], core, 0)) = s;
  }
  else
  {
    core->regs.x[tar] = s;
  }
  return 3;
}

u64 livt(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 src = inst[1];
  core->regs.ivt = core->regs.x[src];
  return 2;
}

u64 lkpt(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 src = inst[1];
  core->regs.kpt = core->regs.x[src];
  return 2;
}

u64 lupt(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 src = inst[1];
  core->regs.upt = core->regs.x[src];
  return 2;
}

u64 lsrg(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  core->regs.x[tar] = (&core->regs.usb)[src];
  return 2;
}

u64 ssrg(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  (&core->regs.usb)[src] = core->regs.x[tar];
  return 2;
}

u64 in(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  // TODO in
  return 3;
}

u64 out(u8 *inst, _core *core)
{
  RPL_MODE_CHECK(core);
  // TODO out
  return 3;
}

u64 cut(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  if (tar == 1)
  {
    core->regs.x[src] = (u8)core->regs.x[src];
  }
  else if (tar == 2)
  {
    core->regs.x[src] = (u16)core->regs.x[src];
  }
  else if (tar == 4)
  {
    core->regs.x[src] = (u32)core->regs.x[src];
  }
  // 不需要处理64位的
  return 2;
}

u64 icut(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  if (tar == 1)
  {
    core->regs.x[src] = (i8)core->regs.x[src];
  }
  else if (tar == 2)
  {
    core->regs.x[src] = (i16)core->regs.x[src];
  }
  else if (tar == 4)
  {
    core->regs.x[src] = (i32)core->regs.x[src];
  }
  // 不需要处理64位的
  return 2;
}

u64 iexp(u8 *inst, _core *core)
{
  u8 tar = inst[1];
  u8 src = tar % 16;
  tar >>= 4;
  if (tar == 1)
  {
    i8 temp = core->regs.x[src];
    u8 sig = temp < 0 ? 1 : 0;
    temp = -temp;
    i64 temp64 = temp;
    if (sig)
    {
      temp64 = -temp64;
    }
  }
  else if (tar == 2)
  {
    i16 temp = core->regs.x[src];
    u8 sig = temp < 0 ? 1 : 0;
    temp = -temp;
    i64 temp64 = temp;
    if (sig)
    {
      temp64 = -temp64;
    }
  }
  else if (tar == 4)
  {
    i32 temp = core->regs.x[src];
    u8 sig = temp < 0 ? 1 : 0;
    temp = -temp;
    i64 temp64 = temp;
    if (sig)
    {
      temp64 = -temp64;
    }
  }
  // 不需要处理64位的
  return 2;
}
