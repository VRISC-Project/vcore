/**
 * @file assembler.hh
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-12
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __assembler_hh__
#define __assembler_hh__

#include <iostream>
#include <map>
#include <functional>
#include <vector>
#include <fstream>

extern std::function<
    void(
        std::vector<std::string> &,
        std::map<std::string, uint64_t> &st,
        uint64_t spesymcnt,
        std::fstream &)>
    instructions[256];

extern uint64_t glb_ip;

#endif
