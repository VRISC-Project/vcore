/**
 * @file core.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-17
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __vrisc_h__
#define __vrisc_h__

#include "../types.h"
#include "pubstruc.h"

extern char *exts_name[];

#ifdef __vrisc_main__

extern u8 *core_start_flags;

extern u64 (*instructions)(u8 *, _core *);

#endif

extern _core **cores;
extern u8 *core_start_flags;

extern struct options cmd_options;

// 初始化vrisc核心
void init_core();

// 寻址
#define LEVEL4_PTS_AREA (0x003ff00000000000)
#define LEVEL3_PTS_AREA (0x00000ffc00000000)
#define LEVEL2_PTS_AREA (0x00000003ff000000)
#define LEVEL1_PTS_AREA (0x0000000000ffc000)
#define PAGE_OFFSET (0x0000000000003fff)
#define LEVEL4_BIGPAGE_OFFSET (LEVEL3_PTS_AREA | LEVEL2_PTS_AREA | LEVEL1_PTS_AREA | PAGE_OFFSET)
#define LEVEL3_BIGPAGE_OFFSET (LEVEL2_PTS_AREA | LEVEL1_PTS_AREA | PAGE_OFFSET)
#define LEVEL2_BIGPAGE_OFFSET (LEVEL1_PTS_AREA | PAGE_OFFSET)
#define LEVEL4_PTS_OFFSET (44)
#define LEVEL3_PTS_OFFSET (34)
#define LEVEL2_PTS_OFFSET (24)
#define LEVEL1_PTS_OFFSET (14)
#define USERFLAG (0x8000000000000000)
u64 vtaddr(u64 ip, _core *core, u8 test_or_address);

// vrisc核心线程
void *vrisc_core(void *id);

void intctl_addint(_core *core, u8 intid);

#endif
