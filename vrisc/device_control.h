/**
 * @file device_control.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-01
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __device_control_h__
#define __device_control_h__

#include "types.h"
#include "core/pubstruc.h"

extern _core **cores;
extern u8 *core_start_flags;
extern struct options cmd_options;
extern u8 *memory;

void make_vrisc_device();
void remove_vrisc_device();

#define INTCTL_NAMESTR "vrisc-intctl"
#define IOCTL_NAMESTR "vrisc-ioctl"

#endif
