/**
 * @file u64toa.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-21
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "u64toa.h"

#include <stdlib.h>
#include <string.h>

static u8
eff_bits(u64 num)
{
  u8 bits = 0;
  while (num)
  {
    bits++;
    num >>= 1;
  }
  return bits;
}

char *u64toa2(u64 num)
{
}

char *u64toa8(u64 num)
{
}

char *u64toa10(u64 num)
{
}

u8 uto16ch[] = {
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
};

char *u64toa16(u64 num)
{
}

char *u64toaddr(u64 num)
{
  char *str = malloc(17);
  memset(str, '0', 17);
  str[16] = '\0';
  for (u32 i = 0; num; i++)
  {
    str[15 - i] = uto16ch[num & 15];
    num >>= 4;
  }
  return str;
}
