/**
 * @file io.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2023-01-09
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#ifndef __io_h__
#define __io_h__

#include "../vrisc/ioctl.h"

#define IOCTL_NAMESTR "vrisc-ioctl"

ioctl_shm *link_io();
void unlink_io(ioctl_shm *ctl);

#endif
