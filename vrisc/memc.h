/**
 * @file memc.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2022-12-17
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#ifndef __memc_h__
#define __memc_h__

#include "types.h"

extern u8 *memory;

void create_memory(u64 size);

#endif
