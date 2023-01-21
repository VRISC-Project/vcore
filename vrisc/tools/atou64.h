/**
 * @file atou64.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2023-01-21
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#ifndef __atou64_h__
#define __atou64_h__

#include "../types.h"

/**
 * @brief 字符串转64位无符号整数
 * 
 * 支持2、8、10、16进制
 * 
 * @param numstr 
 * @return u64 
 */
u64 atou64(char *numstr);

#endif
