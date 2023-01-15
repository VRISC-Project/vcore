/**
 * @file len.cc
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-09
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "len.hh"

#define LEN_FUNC [](                      \
    std::vector<std::string> & v,         \
    std::map<std::string, uint64_t> & st, \
    uint64_t spesymcnt)

uint64_t jc(std::vector<std::string> &v, std::map<std::string, uint64_t> &st, uint64_t spesymcnt)
{
    std::string numstr = v[1];
    if (!is_number(numstr))
    {
        if (numstr[0] == '@')
        {
            if (numstr[1] == 'p')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt);
            }
            else if (numstr[1] == 'n')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt + 1);
            }
        }
        numstr = st[numstr];
    }
    uint64_t num = atou64(numstr);
    int bits = 0;
    while (num)
    {
        bits++;
        num >>= 1;
    }
    if (bits <= 16)
    {
        return 4;
    }
    else if (bits <= 32)
    {
        return 6;
    }
    else
    {
        return 10;
    }
}

uint64_t cc(std::vector<std::string> &v, std::map<std::string, uint64_t> &st, uint64_t spesymcnt)
{
    std::string numstr = v[1];
    if (!is_number(numstr))
    {
        if (numstr[0] == '@')
        {
            if (numstr[1] == 'p')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt);
            }
            else if (numstr[1] == 'n')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt + 1);
            }
        }
        numstr = st[numstr];
    }
    uint64_t num = atou64(numstr);
    int bits = 0;
    while (num)
    {
        bits++;
        num >>= 1;
    }
    if (bits <= 16)
    {
        return 4;
    }
    else if (bits <= 32)
    {
        return 6;
    }
    else
    {
        return 10;
    }
}

uint64_t ldi(std::vector<std::string> &v, std::map<std::string, uint64_t> &st, uint64_t spesymcnt)
{
    std::string numstr = v[1];
    if (!is_number(numstr))
    {
        if (numstr[0] == '@')
        {
            if (numstr[1] == 'p')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt);
            }
            else if (numstr[1] == 'n')
            {
                numstr.erase(1);
                numstr += std::to_string(spesymcnt + 1);
            }
        }
        numstr = st[numstr];
    }
    uint64_t num = atou64(numstr);
    int bits = 0;
    while (num)
    {
        bits++;
        num >>= 1;
    }
    if (bits <= 8)
    {
        return 3;
    }
    else if (bits <= 16)
    {
        return 4;
    }
    else if (bits <= 32)
    {
        return 6;
    }
    else
    {
        return 10;
    }
}

std::function<
    uint64_t(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t)>
    len[256] = {
        LEN_FUNC
        { return 1; }, // nop
        LEN_FUNC
        { return 3; }, // add
        LEN_FUNC
        { return 3; }, // sub
        LEN_FUNC
        { return 2; }, // inc
        LEN_FUNC
        { return 2; }, // dec
        LEN_FUNC
        { return 2; }, // cmp
        LEN_FUNC
        { return 3; }, // and
        LEN_FUNC
        { return 3; }, // or
        LEN_FUNC
        { return 2; }, // not
        LEN_FUNC
        { return 3; }, // xor
        jc,
        cc,
        LEN_FUNC
        { return 1; }, // r
        LEN_FUNC
        { return 2; }, // ir
        LEN_FUNC
        { return 1; }, // sysc
        LEN_FUNC
        { return 1; }, // sysr
        LEN_FUNC
        { return 6; }, // loop
        LEN_FUNC
        { return 2; }, // chl
        LEN_FUNC
        { return 2; }, // chr
        LEN_FUNC
        { return 2; }, // rol
        LEN_FUNC
        { return 2; }, // ror
        ldi,
        LEN_FUNC
        { return 2; }, // ldm
        LEN_FUNC
        { return 2; }, // stm
        LEN_FUNC
        { return 1; }, // ei
        LEN_FUNC
        { return 1; }, // di
        LEN_FUNC
        { return 1; }, // ep
        LEN_FUNC
        { return 1; }, // dp
        LEN_FUNC
        { return 3; }, // mv
        LEN_FUNC
        { return 2; }, // livt
        LEN_FUNC
        { return 2; }, // lkpt
        LEN_FUNC
        { return 2; }, // lupt
        LEN_FUNC
        { return 2; }, // lsrg
        LEN_FUNC
        { return 2; }, // ssrg
        LEN_FUNC
        { return 2; }, // initext
        LEN_FUNC
        { return 1; }, // destext
        LEN_FUNC
        { return 3; }, // in
        LEN_FUNC
        { return 3; }, // out
        LEN_FUNC
        { return 2; }, // cut
        LEN_FUNC
        { return 2; }, // icut
        LEN_FUNC
        { return 2; }, // iexp
        LEN_FUNC
        { return 1; }, // cpuid
};

std::map<std::string, uint8_t> instcode = {
    std::make_pair("nop", 0),
    std::make_pair("add", 1),
    std::make_pair("sub", 2),
    std::make_pair("inc", 3),
    std::make_pair("dec", 4),
    std::make_pair("cmp", 5),
    std::make_pair("and", 6),
    std::make_pair("or", 7),
    std::make_pair("not", 8),
    std::make_pair("xor", 9),
    std::make_pair("j", 10),
    std::make_pair("je", 10),
    std::make_pair("jb", 10),
    std::make_pair("js", 10),
    std::make_pair("jne", 10),
    std::make_pair("jnb", 10),
    std::make_pair("jns", 10),
    std::make_pair("jh", 10),
    std::make_pair("jl", 10),
    std::make_pair("jnh", 10),
    std::make_pair("jnl", 10),
    std::make_pair("jo", 10),
    std::make_pair("jz", 10),
    std::make_pair("c", 11),
    std::make_pair("ce", 11),
    std::make_pair("cb", 11),
    std::make_pair("cs", 11),
    std::make_pair("cne", 11),
    std::make_pair("cnb", 11),
    std::make_pair("cns", 11),
    std::make_pair("ch", 11),
    std::make_pair("cl", 11),
    std::make_pair("cnh", 11),
    std::make_pair("cnl", 11),
    std::make_pair("co", 11),
    std::make_pair("cz", 11),
    std::make_pair("r", 12),
    std::make_pair("ir", 13),
    std::make_pair("sysc", 14),
    std::make_pair("sysr", 15),
    std::make_pair("loop", 16),
    std::make_pair("chl", 17),
    std::make_pair("chr", 18),
    std::make_pair("rol", 19),
    std::make_pair("ror", 20),
    std::make_pair("ldi", 21),
    std::make_pair("ldm", 22),
    std::make_pair("stm", 23),
    std::make_pair("ei", 24),
    std::make_pair("di", 25),
    std::make_pair("ep", 26),
    std::make_pair("dp", 27),
    std::make_pair("mv", 28),
    std::make_pair("livt", 29),
    std::make_pair("lkpt", 30),
    std::make_pair("lupt", 31),
    std::make_pair("lsrg", 32),
    std::make_pair("ssrg", 33),
    std::make_pair("initext", 34),
    std::make_pair("destext", 35),
    std::make_pair("in", 36),
    std::make_pair("out", 37),
    std::make_pair("cut", 38),
    std::make_pair("icut", 39),
    std::make_pair("iexp", 40),
    std::make_pair("cpuid", 41),
};
