/**
 * @file len.hh
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-09
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#ifndef __len_hh__
#define __len_hh__

#include <iostream>
#include <functional>
#include <map>

int is_number(std::string str);
uint64_t atou64(std::string &str);

uint64_t jc(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t);
uint64_t cc(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t);
uint64_t ldi(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t);

extern std::function<uint64_t(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t)> len[256];
extern std::map<std::string, uint8_t> instcode;

#endif
