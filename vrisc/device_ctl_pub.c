/**
 * @file device_ctl_pub.c
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 
 * @version 0.1
 * @date 2023-01-06
 * 
 * @copyright Copyright (c) 2022 Random World Studio
 * 
 */

#include "device_control.h"

#include <stdio.h>
#include <stdlib.h>

char *generate_intctl_namestr()
{
  char *intctl;
  intctl = malloc(32);
  sprintf(intctl, "vrisc-intctl");
  return intctl;
}

char *generate_ioctl_namestr()
{
  char *ioctl;
  ioctl = malloc(32);
  sprintf(ioctl, "vrisc-ioctl");
  return ioctl;
}
