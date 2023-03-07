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
#include <string.h>
#include <pthread.h>
#include <time.h>

#if defined(__linux__)
#include <unistd.h>
#include <dlfcn.h>
#include <sys/time.h>
#elif defined(_WIN32)
#include <windows.h>
#endif

#include "base.h"

/*
指令集的指令执行函数数组。
通过malloc分配数组空间。
 */
u64 (**instructions)(u8 *, _core *);

// 扩展指令集的动态库
void *ext_so;

// 扩展指令集名
char *exts_name[] = {
    (char *)"", // 0号扩展指令集是基本指令集，不需要额外加载
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
  instructions[17] = shl;
  instructions[18] = shr;
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
  instructions[39] = icut;
  instructions[40] = iexp;
}

/* 寻址函数
参数test_or_address为0时表示寻址，不为零表示测试地址有效性。
测试地址有效性时，返回0为无效，非零为有效。
寻址时默认地址有效。
 */
u64 vtaddr(u64 ip, _core *core, u8 test_or_address)
{
  if (!(core->regs.flg & (1 << 7)))
  { // 未开启分页
    if (test_or_address && (core->regs.flg & (1 << 8)))
    { // 处于用户态，不可以使用物理地址
      intctl_addint(core, IR_PERMISSION_DENIED);
      return 0;
    }
    if (ip >= cmd_options.mem_size)
    {
      intctl_addint(core, IR_NOT_EFFECTIVE_ADDRESS);
      return 0;
    }
    if (test_or_address)
    {
      return 1;
    }
    else
    {
      return ip;
    }
  }

  u8 addr_in_usermod = (ip & USERFLAG) ? 1 : 0;
  ip &= ~USERFLAG;

  if (test_or_address && (core->regs.flg & (1 << 8)) && !addr_in_usermod)
  { // 用户态使用内核态地址
    intctl_addint(core, IR_PERMISSION_DENIED);
    return 0;
  }

  /* 四级页表访问 */
  u16 level4_pts = (ip & LEVEL4_PTS_AREA) >> LEVEL4_PTS_OFFSET; // pts指page table selector，页表项选择子

  u64 level3_pt = ((u64 *) // memory是u8 *类型，地址应该是64位，所以将它改成u64 *，就可以直接通过level4_pts访问
                   (memory +
                    ((addr_in_usermod) ? core->regs.upt : core->regs.kpt) // 如果是用户态就使用用户态页表，否则使用内核态
                    ))[level4_pts];
  u64 level4_pt_flgs = level3_pt & PAGE_OFFSET;  // 获取四级页表项的标志位
  u8 level4_exist = level4_pt_flgs & 1;          // 四级页表大页是否在内存中
  u8 level4_bigpage = level4_pt_flgs & (1 << 1); // 四级页表项大页标志
  level3_pt &= ~PAGE_OFFSET;                     // 截取为一个物理地址

  /* 三级页表访问 */
  if (level4_bigpage)
  { // 如果是大页
    if (test_or_address && !level4_exist)
    {
      goto generate_interrupt;
      return 0;
    }
    return level3_pt + (ip & LEVEL4_BIGPAGE_OFFSET);
  }
  u16 level3_pts = (ip & LEVEL3_PTS_AREA) >> LEVEL3_PTS_OFFSET;
  u64 level2_pt = ((u64 *)(memory + level3_pt))[level3_pts];
  u64 level3_pt_flgs = level2_pt & PAGE_OFFSET;
  u8 level3_exist = level3_pt_flgs & 1;
  u8 level3_bigpage = level3_pt_flgs & (1 << 1);
  level2_pt &= ~PAGE_OFFSET;

  /* 二级页表访问 */
  if (level3_bigpage)
  { // 如果是大页
    if (test_or_address && !level3_exist)
    {
      goto generate_interrupt;
      return 0;
    }
    return level2_pt + (ip & LEVEL3_BIGPAGE_OFFSET);
  }
  u16 level2_pts = (ip & LEVEL2_PTS_AREA) >> LEVEL2_PTS_OFFSET;
  u64 level1_pt = ((u64 *)(memory + level2_pt))[level2_pts];
  u64 level2_pt_flgs = level1_pt & PAGE_OFFSET;
  u8 level2_exist = level2_pt_flgs & 1;
  u8 level2_bigpage = level2_pt_flgs & (1 << 1);
  level1_pt &= ~PAGE_OFFSET;

  /* 一级页表访问 */
  if (level2_bigpage)
  { // 如果是大页
    if (test_or_address && !level2_exist)
    {
      goto generate_interrupt;
      return 0;
    }
    return level1_pt + (ip & LEVEL2_BIGPAGE_OFFSET);
  }
  u16 level1_pts = (ip & LEVEL1_PTS_AREA) >> LEVEL1_PTS_OFFSET;
  u64 page_addr = ((u64 *)(memory + level1_pt))[level1_pts];
  u64 level1_pt_flgs = page_addr & PAGE_OFFSET;
  u8 level1_exist = level1_pt_flgs & 1;
  page_addr &= ~PAGE_OFFSET;

  if (test_or_address && !level1_exist)
  {
    goto generate_interrupt;
    return 0;
  }
  ip = page_addr | (ip & PAGE_OFFSET);
  if (ip >= cmd_options.mem_size)
  {
    intctl_addint(core, IR_NOT_EFFECTIVE_ADDRESS);
    return 0;
  }
  return ip;
generate_interrupt:
  intctl_addint(core, IR_NOT_EFFECTIVE_ADDRESS);
  return 0;
}

#if defined(__linux__)
static u64
get_us_time()
{
  static struct timeval time;
  gettimeofday(&time, NULL);
  return (time.tv_sec * 1000 + time.tv_usec) * 1000;
}
#endif

#define CLOCK_TICK 2000
void *
clock_producer(void *args)
{
  _core *core = (_core *)(((_core **)args)[0]);
  u64 cid = (u64)(((u64 *)args)[1]);
  u64 last_time, current_time;
  last_time = get_us_time();
  while (core_start_flags[cid])
  {
    if (core->debug.debugging)
    {
      while (!core->debug.continuing || !core->debug.trap)
      {
#if defined(__linux__)
        nanosleep(&(struct timespec){0, 100000}, NULL);
#elif defined(_WIN32)
        Sleep(1);
#endif
      }
    }
#if defined(__linux__)
    current_time = get_us_time();
    i64 slp_time = CLOCK_TICK - (current_time - last_time);
    if (slp_time < 0)
    {
      slp_time = 0;
    }
    struct timespec slp = {0, slp_time * 1000};
    while (slp.tv_nsec)
    {
      if (!nanosleep(&slp, &slp))
      {
        slp.tv_sec = 0;
        slp.tv_nsec = 0;
      }
    }
#elif defined(_WIN32)
    Sleep(5);
#endif
    intctl_addint(core, IR_CLOCK);
  }
}

static char *
generate_extpath(u64 id)
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
#if defined(__linux__)
  memcpy(path + l, ".so", 3);
#elif defined(_WIN32)
  memcpy(path + l, ".dll", 4);
#endif
  l += 3;
  path[l++] = 0;
  free(idstr);
  return path;
}

static void
inst_initext(_core *core, u64 *ipbuff)
{
#if defined(__linux__)
  if (ext_so)
  {
    dlclose(ext_so);
  }
  // 打开扩展指令集
  u64 extid = memory[vtaddr(core->regs.ip, core, 0) + 1];
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
#elif defined(_WIN32)
// TODO 兼容win
#endif
  core->regs.ip += 2;
  *ipbuff += 2;
}

static void
inst_destext(_core *core, u64 *ipbuff)
{
#if defined(__linux__)
  u64 inst_count = *((u64 *)dlsym(ext_so, "vriscext_inst_count"));
  u64 space_start = *((u64 *)dlsym(ext_so, "vriscext_space_start"));
  memset(instructions + space_start, 0, inst_count * sizeof(u64(*)(u8 *, _core *)));
  dlclose(ext_so);
#elif defined(_WIN32)
// TODO 兼容win
#endif
  core->regs.ip++;
  (*ipbuff)++;
}

/* 添加一个中断 */
void intctl_addint(_core *core, u8 intid)
{
  while (
      core->interrupt.controller.head ==
      core->interrupt.controller.tail + 1)
    nanosleep(&(struct timespec){0, 50000}, NULL); // 如果队列满了要等待
  u8_lock_lock(core->interrupt.controller.lock);
  core->interrupt.controller.interrupt_queue[core->interrupt.controller.tail] = intid;
  core->interrupt.controller.tail++;
  if (core->interrupt.controller.tail == LOCAL_INTQUEUE_BUFFER_SIZE)
  {
    core->interrupt.controller.tail = 0;
  }
  u8_lock_unlock(core->interrupt.controller.lock);
}

/* 本地中断控制函数 */
static void
local_interrupt_controlling(_core *core)
{
  core->interrupt.triggered = 1;
  core->interrupt.int_id = core->interrupt.controller.interrupt_queue[core->interrupt.controller.head];
  u8_lock_lock(core->interrupt.controller.lock);
  core->interrupt.controller.head++;
  if (core->interrupt.controller.head == LOCAL_INTQUEUE_BUFFER_SIZE)
  {
    core->interrupt.controller.head = 0;
  }
  u8_lock_unlock(core->interrupt.controller.lock);
}

static void
inst_nop(_core *core, u64 *ipbuff)
{
  while (!core->interrupt.triggered)
  {
    local_interrupt_controlling(core);
#if defined(__linux__)
    nanosleep(&(struct timespec){0, 1000000}, NULL);
#elif defined(_WIN32)
    Sleep(1);
#endif
  }
  core->regs.ip++;
  (*ipbuff)++;
}

static void
flash_ipbuff(_core *core, u64 *ipbuff)
{
  // 先在AM中寻找已经转换的地址
  for (u8 i = core->addressing_manager.end; i != core->addressing_manager.begin; i--)
  {
    if (i == AM_AD_SIZE)
    {
      i = 0;
    }
    struct vp_pair pair = core->addressing_manager.addressed_addresses[i];
    if (pair.vt == core->regs.ip)
    {
      *ipbuff = pair.ph;
      core->ipbuff_need_flash = 0;
      // 将这个地址推到末尾，防止过早地被刷新掉
      for (u8 j = i; j != core->addressing_manager.end - 1; j++)
      {
        core->addressing_manager.addressed_addresses[j].vt =
            core->addressing_manager.addressed_addresses[j + 1].vt;
        core->addressing_manager.addressed_addresses[j].ph =
            core->addressing_manager.addressed_addresses[j + 1].ph;
      }
      core->addressing_manager.addressed_addresses
          [core->addressing_manager.end - 1]
              .vt = pair.vt;
      core->addressing_manager.addressed_addresses
          [core->addressing_manager.end - 1]
              .ph = pair.ph;
      break;
    }
  }
  if (core->ipbuff_need_flash)
  {
    *ipbuff = vtaddr(core->regs.ip, core, 0);
    core->ipbuff_need_flash = 0;
    // 新寻址后的地址存入AM中
    if (core->addressing_manager.begin - 1 ==
        core->addressing_manager.end)
    {
      core->addressing_manager.begin += 64;
    }
    core->addressing_manager.addressed_addresses
        [core->addressing_manager.end++] =
        (struct vp_pair){
            .vt = core->regs.ip,
            .ph = *ipbuff};
  }
  // 如果需要刷新寻址管理器
  if (core->am_need_flash)
  {
    core->addressing_manager.begin = 0;
    core->addressing_manager.end = 0;
    core->am_need_flash = 0;
  }
}

static void
debugging(_core *core)
{
  for (u32 i = 0; i < core->debug.bpcount; i++)
  {
    if (core->debug.breakpoints[i] == core->regs.ip)
    {
      core->debug.continuing = 0;
      core->debug.contflg = 0;
      core->debug.trapflg = 0;
      break;
    }
  }
  if (core->debug.trapflg)
  {
    if (core->debug.trap)
    {
      core->debug.trap--;
      return;
    }
    core->debug.debugging = 1;
    while (!core->debug.continuing && !core->debug.trap)
    {
#if defined(__linux__)
      nanosleep(&(struct timespec){0, 1000000}, NULL);
#elif defined(_WIN32)
      Sleep(1);
#endif
    }
    core->debug.debugging = 0;
    core->debug.trapflg = 0;
    return;
  }
  if (core->debug.contflg)
  {
    if (core->debug.continuing)
    {
      return;
    }
    core->debug.debugging = 1;
    while (!core->debug.continuing && !core->debug.trap)
    {
#if defined(__linux__)
      nanosleep(&(struct timespec){0, 1000000}, NULL);
#elif defined(_WIN32)
      Sleep(1);
#endif
    }
    core->debug.debugging = 0;
    core->debug.contflg = 0;
  }
}

void *
vrisc_core(void *id)
{
  u64 cid = (u64)id;
  // _core coreobj;
  // _core *core = &coreobj;
  _core *core = malloc(sizeof(_core)); // 构造核心
  if (!core)
  {
    printf("Failed to create core#%d.\n", (u64)id);
    return (void *)CORE_FAILED;
  }

  cores[cid] = core; // 注册核心
  memset((void *)core, 0, sizeof(_core));
  core->id = cid;

  core->ipbuff_need_flash = 1;

  // 等待核心被允许开启
  while (!core_start_flags[cid])
  {
#if defined(__linux__)
    nanosleep(&(struct timespec){0, 500000}, NULL);
#elif defined(_WIN32)
    Sleep(1);
#endif
  }

  pthread_t clock_id;
  void *args[2] = {core, (void *)cid};
  if (!cmd_options.shield_internal_clock)
  { // 开启内部时钟
    pthread_create(&clock_id, NULL, clock_producer, args);
  }

  u64 ipbuff; // 此变量说明见 _core::ipbuff_need_flush

  core->debug.continuing = 1;

  // u64 tot = 0;
  // u64 ts = get_us_time();

  while (core_start_flags[cid])
  {
    // tot++;

    if (core->ipbuff_need_flash)
    { // 刷新ipbuff
      flash_ipbuff(core, &ipbuff);
    }

    if ((
            core->interrupt.controller.head !=
            core->interrupt.controller.tail) &&
        !core->interrupt.triggered)
    { // 可以处理下一个中断
      local_interrupt_controlling(core);
    }

    // debug
    if (cmd_options.debug)
    {
      debugging(core);
    }

    if (!*(memory + ipbuff))
    { // nop
      inst_nop(core, &ipbuff);
      continue;
    }
    if (*(memory + ipbuff) == 34)
    { // initext
      inst_initext(core, &ipbuff);
      continue;
    }
    else if (*(memory + ipbuff) == 35)
    { // destext
      inst_destext(core, &ipbuff);
      continue;
    }
    if (!(instructions[memory[ipbuff]]))
    { // 无效指令
      intctl_addint(core, IR_INSTRUCTION_NOT_RECOGNIZED);
    }
    if ((core->regs.flg & (1 << 6)) && core->interrupt.triggered)
    { // 如果发生中断先进入中断
      core->interrupt.triggered = 0;
      // 保存必要寄存器状态
      core->regs.x[0] = core->regs.ip;
      core->regs.x[1] = core->regs.flg;
      core->regs.flg &= ~(1 << 6); // 关闭flg^6.ie
      core->regs.flg &= ~(1 << 8); // 进入内核态
      // 让ip寄存器跳到中断处理程序
      core->regs.ip = *(u64 *)(memory + core->regs.ivt + core->interrupt.int_id * 8);
      core->ipbuff_need_flash = 1;
      continue;
    }
    // 执行指令
    core->incr =
        (*instructions[*(memory + ipbuff)])(
            memory + ipbuff, core);
    core->regs.ip += core->incr;
    ipbuff += core->incr;
  }

  if (!cmd_options.shield_internal_clock)
  {
    pthread_join(clock_id, NULL);
  }

  // u64 te = get_us_time();

  // double hz = tot / (((double)te - ts) / 1000000);

  // printf("\n%.2fHz\n", hz);

  free(core);
}
