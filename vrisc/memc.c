/**
 * @file memc.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2022-12-17
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#include "memc.h"
#include "err.h"

#include <stdlib.h>
#include <stdio.h>

u8 *memory;

void
create_memory(u64 size)
{
  memory = malloc(size);
  if (!memory)
  {
    printf("Failed to allocate memory, exit.\n");
    exit(MEM_FAILED);
  }
  printf("Installed memory %d byte.\n", size);
}
