/**
 * @file u64toa.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-21
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __u64toa_h__
#define __u64toa_h__

#include "../types.h"

/*
此系列函数的字符串都通过malloc申请，
返回的字符串使用后一定要free
 */

char *u64toa2(u64 num);
char *u64toa8(u64 num);
char *u64toa10(u64 num);
char *u64toa16(u64 num);
char *u64toaddr(u64 num);

#endif
