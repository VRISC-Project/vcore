/**
 * @file atou64.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-21
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "atou64.h"

#include <stdlib.h>
#include <string.h>

static u8
is_a_num(char c)
{
  return (c >= '0' && c <= '9');
}

/*
判断是否是数字
是数字返回进制（支持2、8、10、16进制）
 */
static u32
is_number(char * str)
{
  int carrysys = 10;
  if (!is_a_num(str[0]))
  {
    return 0;
  }

  if (strlen(str) == 1)
  {
    return carrysys;
  }

  if (!is_a_num(str[1]))
  {
    if (str[1] == 'x')
    {
      carrysys = 16;
    }
    else if (str[1] == 'b')
    {
      carrysys = 2;
    }
    else
    {
      return 0;
    }
  }
  else if (str[0] == '0')
  {
    carrysys = 8;
  }

  if (strlen(str) == 2)
  {
    return carrysys;
  }

  for (int i = 2; i < strlen(str); i++)
  {
    if (!is_a_num(str[i]))
    {
      if (carrysys == 16 && str[i] >= 'a' && str[i] <= 'f')
      {
        continue;
      }
      else
      {
        return 0;
      }
    }
    else if (carrysys == 8)
    {
      if (str[i] == '8' || str[i] == '9')
      {
        return 0;
      }
    }
    else if (carrysys == 2)
    {
      if (str[i] != '0' && str[i] != '1')
      {
        return 0;
      }
    }
  }
  return carrysys;
}

static u32
cs16_numchar_to_int(char ch)
{
  if (ch >= '0' && ch <= '9')
  {
    return ch - '0';
  }
  else if (ch >= 'a' && ch <= 'f')
  {
    return ch - 'a' + 10;
  }
}

u64 atou64(char *numstr)
{
  int carrysys = is_number(numstr);
  if (!carrysys)
  {
    return 0;
  }
  u64 num = 0;
  if (carrysys == 2)
  {
    for (int i = 2; i < strlen(numstr); i++)
    {
      num |= numstr[i] - '0';
      num <<= 1;
    }
    num >>= 1;
  }
  else if (carrysys == 8)
  {
    for (int i = 1; i < strlen(numstr); i++)
    {
      num |= numstr[i] - '0';
      num <<= 3;
    }
    num >>= 3;
  }
  else if (carrysys == 10)
  {
    for (int i = 0; i < strlen(numstr); i++)
    {
      num += numstr[i] - '0';
      num *= 10;
    }
    num /= 10;
  }
  else if (carrysys == 16)
  {
    for (int i = 2; i < strlen(numstr); i++)
    {
      num |= cs16_numchar_to_int(numstr[i]);
      num <<= 4;
    }
    num >>= 4;
  }
  return num;
}
