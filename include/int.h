/**
 * @file int.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2023-01-10
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#ifndef __int_h__
#define __int_h__

#include "../vrisc/intctl.h"

#define INTCTL_NAMESTR "vrisc-intctl"

intctl_shm *link_int();
void unlink_int(intctl_shm *ctl);

#endif
