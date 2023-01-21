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
  u8 effb = eff_bits(num);
  char *str = malloc(effb + 1);
  str[effb] = '\0';
  while (num)
  {
    for (u32 i = 0; i < effb; i++)
    {
      str[i] = str[i + 1];
    }
    str[effb - 1] = num & 1 ? '1' : '0';
    num >>= 1;
  }
  return str;
}

char *u64toa8(u64 num)
{
  u8 effb = eff_bits(num);
  effb = effb / 3 + ((effb % 3) ? 1 : 0);
  char *str = malloc(effb + 1);
  str[effb] = '\0';
  while (num)
  {
    for (u32 i = 0; i < effb; i++)
    {
      str[i] = str[i + 1];
    }
    str[effb - 1] = (num & 7) + '0';
    num >>= 3;
  }
  return str;
}

char *u64toa10(u64 num)
{
  char *str = malloc(21);
  memset(str, 0, 21);
  while (num)
  {
    for (u32 i = 0; i < 20; i++)
    {
      str[i] = str[i + 1];
    }
    str[19] = num % 10 + '0';
    num /= 10;
  }
  while (!str[0])
  {
    for (u32 i = 0; i < 20; i++)
    {
      str[i] = str[i + 1];
    }
  }
  return str;
}

char *u64toa16(u64 num)
{
  u8 effb = eff_bits(num);
  effb = effb / 4 + ((effb % 4) ? 1 : 0);
  char *str = malloc(effb + 1);
  str[effb] = '\0';
  while (num)
  {
    for (u32 i = 0; i < effb; i++)
    {
      str[i] = str[i + 1];
    }
    str[effb - 1] = (num & 15) + '0';
    num >>= 4;
  }
  return str;
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

char *u64toaddr(u64 num)
{
  char *str = malloc(17);
  memset(str, '0', 17);
  str[16] = '\0';
  while (num)
  {
    for (u32 i = 0; i < 16; i++)
    {
      str[i] = str[i + 1];
    }
    str[15] = uto16ch[num % 16];
    num >>= 4;
  }
  return str;
}
