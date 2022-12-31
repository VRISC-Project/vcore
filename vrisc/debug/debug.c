/**
 * @file debug.cc
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2022-12-29
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "debug.h"
#include "../types.h"

char *commands[] = {
    (char *)"bp", // 设置断点
};

char *
debug(const char *command)
{
    return (char *)command;
}
