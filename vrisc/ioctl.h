/**
 * @file ioctl.h
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-05
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __ioctl_h__
#define __ioctl_h__

#include "types.h"
#include "core/pubstruc.h"

typedef struct ioctl_shm
{
  /*
  io共有64个端口，
  使用串行方式传输（在软件实现上其实就是一个端口整一个队列）；
   */
#define IO_INTERFACE_AMOUNT 64
#define IO_BUFFER_SIZE 65536

  u8 input_locks[IO_INTERFACE_AMOUNT];
  u8 input[IO_INTERFACE_AMOUNT][IO_BUFFER_SIZE];
  u16 input_heads[IO_INTERFACE_AMOUNT];
  u16 input_tails[IO_INTERFACE_AMOUNT];

  u8 output_locks[IO_INTERFACE_AMOUNT];
  u8 output[IO_INTERFACE_AMOUNT][IO_BUFFER_SIZE];
  u16 output_heads[IO_INTERFACE_AMOUNT];
  u16 output_tails[IO_INTERFACE_AMOUNT];
} ioctl_shm;

#ifdef __base_h__
extern ioctl_shm *io;
#endif

void *io_global_controller(void *);

#endif
