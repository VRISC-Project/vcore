/**
 * @file assembler.cc
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief
 * @version 0.1
 * @date 2023-01-12
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include "assembler.hh"

#include "len.hh"

#define ASSEM_FUNC\
std ::vector<std::string> &v,            \
    std::map<std::string, uint64_t> &st, \
    uint64_t spesymcnt,                  \
    std::fstream &of

void i_nop(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_add(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    // 源操作数
    uint8_t src;
    std::string op = v[2];
    op.erase(0);
    src = atou64(op);
    src <<= 4;
    op = v[1];
    op.erase(0);
    src |= 0xf & atou64(op);
    of.write((char *)&src, 1);
    // 目标操作数
    uint8_t tar;
    op = v[3];
    op.erase(0);
    tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_sub(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    // 源操作数
    uint8_t src;
    std::string op = v[2];
    op.erase(0);
    src = atou64(op);
    src <<= 4;
    op = v[1];
    op.erase(0);
    src |= 0xf & atou64(op);
    of.write((char *)&src, 1);
    // 目标操作数
    uint8_t tar;
    op = v[3];
    op.erase(0);
    tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_inc(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_dec(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_cmp(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_and(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    // 源操作数
    uint8_t src;
    std::string op = v[2];
    op.erase(0);
    src = atou64(op);
    src <<= 4;
    op = v[1];
    op.erase(0);
    src |= 0xf & atou64(op);
    of.write((char *)&src, 1);
    // 目标操作数
    uint8_t tar;
    op = v[3];
    op.erase(0);
    tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_or(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    // 源操作数
    uint8_t src;
    std::string op = v[2];
    op.erase(0);
    src = atou64(op);
    src <<= 4;
    op = v[1];
    op.erase(0);
    src |= 0xf & atou64(op);
    of.write((char *)&src, 1);
    // 目标操作数
    uint8_t tar;
    op = v[3];
    op.erase(0);
    tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_not(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_xor(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    // 源操作数
    uint8_t src;
    std::string op = v[2];
    op.erase(0);
    src = atou64(op);
    src <<= 4;
    op = v[1];
    op.erase(0);
    src |= 0xf & atou64(op);
    of.write((char *)&src, 1);
    // 目标操作数
    uint8_t tar;
    op = v[3];
    op.erase(0);
    tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

static uint8_t
condcode(std::string condinst)
{
    if (condinst == "")
    {
        return 0;
    }
    if (condinst == "e")
    {
        return 1;
    }
    if (condinst == "b")
    {
        return 2;
    }
    if (condinst == "s")
    {
        return 3;
    }
    if (condinst == "ne")
    {
        return 4;
    }
    if (condinst == "nb")
    {
        return 5;
    }
    if (condinst == "ns")
    {
        return 6;
    }
    if (condinst == "h")
    {
        return 7;
    }
    if (condinst == "l")
    {
        return 8;
    }
    if (condinst == "nh")
    {
        return 9;
    }
    if (condinst == "nl")
    {
        return 10;
    }
    if (condinst == "o")
    {
        return 11;
    }
    if (condinst == "z")
    {
        return 12;
    }
    return 0xff;
}

static uint8_t
eff_bits(uint64_t num)
{
    uint8_t bits = 0;
    while (num)
    {
        bits++;
        num >>= 1;
    }
    return bits;
}

void i_jc(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string instname = v[0];
    instname.erase(0);
    uint8_t byte1 = condcode(instname);
    byte1 <<= 4;
    uint64_t num;
    std::string op = v[1];
    if (st.find(op) != st.end())
    {
        num = st[op];
    }
    else
    {
        num = atou64(op);
    }
    uint8_t bits = eff_bits(num);
    if (bits <= 16)
    {
        byte1 |= 0;
        bits = 0;
    }
    else if (bits <= 32)
    {
        byte1 |= 1;
        bits = 1;
    }
    else
    {
        byte1 |= 2;
        bits = 2;
    }
    of.write((char *)&byte1, 1);
    if (bits == 0)
    {
        of.write((char *)&num, 2);
    }
    else if (bits == 1)
    {
        of.write((char *)&num, 4);
    }
    else
    {
        of.write((char *)&num, 8);
    }
}

void i_cc(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string instname = v[0];
    instname.erase(0);
    uint8_t byte1 = condcode(instname);
    byte1 <<= 4;
    uint64_t num;
    std::string op = v[1];
    if (st.find(op) != st.end())
    {
        num = st[op];
    }
    else
    {
        num = atou64(op);
    }
    uint8_t bits = eff_bits(num);
    if (bits <= 16)
    {
        byte1 |= 0;
        bits = 0;
    }
    else if (bits <= 32)
    {
        byte1 |= 1;
        bits = 1;
    }
    else
    {
        byte1 |= 2;
        bits = 2;
    }
    of.write((char *)&byte1, 1);
    if (bits == 0)
    {
        of.write((char *)&num, 2);
    }
    else if (bits == 1)
    {
        of.write((char *)&num, 4);
    }
    else
    {
        of.write((char *)&num, 8);
    }
}

void i_r(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_ir(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t op;
    if (v[1] == "0")
    {
        op = 0;
    }
    else if (v[1] == "1")
    {
        op = 1;
    }
    else if (v[1] == "2")
    {
        op = 2;
    }
    of.write((char *)&op, 1);
}

void i_sysc(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_sysr(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_loop(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t src = 0xf & atou64(op);
    of.write((char *)&src, 1);
    int32_t num;
    op = v[2];
    if (st.find(op) != st.end())
    {
        num = st[op] - glb_ip;
    }
    else
    {
        if (op[0] == '-')
        {
            op.erase(0);
            num = -atou64(op);
        }
        else
        {
            num = atou64(op);
        }
    }
    of.write((char *)&num, 4);
}

void i_chl(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_chr(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_rol(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_ror(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_ldi(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[2];
    op.erase(0);
    uint8_t byte1 = 0xf & atou64(op);
    byte1 <<= 4;
    op = v[1];
    uint64_t num = atou64(op);
    uint8_t bits = eff_bits(num);
    if (bits <= 8)
    {
        bits = 1;
        byte1 |= 0;
    }
    else if (bits <= 16)
    {
        bits = 2;
        byte1 |= 1;
    }
    else if (bits <= 32)
    {
        bits = 4;
        byte1 |= 2;
    }
    else
    {
        bits = 8;
        byte1 |= 3;
    }
    of.write((char *)&byte1, 1);
    of.write((char *)&num, bits);
}

void i_ldm(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_stm(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_ei(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_di(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_ep(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_dp(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_mv(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t byte2;
    uint8_t flg = 0;
    std::string op = v[2];
    if (op[0] == '*')
    {
        flg |= 2;
        op.erase(0);
    }
    op.erase(0);
    byte2 |= 0xf & atou64(op);
    byte2 <<= 4;
    op = v[1];
    if (op[0] == '*')
    {
        flg |= 1;
        op.erase(0);
    }
    op.erase(0);
    byte2 |= 0xf & atou64(op);
    of.write((char *)&flg, 1);
    of.write((char *)&byte2, 1);
}

void i_livt(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_lkpt(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_lupt(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_lsrg(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_ssrg(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    uint8_t ops;
    std::string op = v[2];
    op.erase(0);
    ops = 0xf & atou64(op);
    ops <<= 4;
    op = v[1];
    op.erase(0);
    ops |= 0xf & atou64(op);
    of.write((char *)&ops, 1);
}

void i_initext(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    op.erase(0);
    uint8_t tar = 0xf & atou64(op);
    of.write((char *)&tar, 1);
}

void i_destext(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

void i_in(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[1];
    uint8_t flg = 0;
    uint8_t src;
    if (is_number(op))
    {
        flg = 2;
        src = atou64(op);
    }
    else
    {
        op.erase(0);
        src = 0xf & atou64(op);
    }
    flg <<= 4;
    op = v[2];
    op.erase(0);
    flg |= 0xf & atou64(op);
    of.write((char *)&flg, 1);
    of.write((char *)&src, 1);
}

void i_out(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[2];
    uint8_t flg = 0;
    uint8_t src;
    if (is_number(op))
    {
        flg = 1;
        src = atou64(op);
    }
    else
    {
        op.erase(0);
        src = 0xf & atou64(op);
    }
    flg <<= 4;
    op = v[1];
    op.erase(0);
    flg |= 0xf & atou64(op);
    of.write((char *)&flg, 1);
    of.write((char *)&src, 1);
}

void i_cut(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[2];
    op.erase(0);
    uint8_t byte = atou64(op);
    byte <<= 4;
    op = v[1];
    op.erase(0);
    byte |= 0xf & atou64(op);
    of.write((char *)&byte, 1);
}

void i_icut(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[2];
    op.erase(0);
    uint8_t byte = atou64(op);
    byte <<= 4;
    op = v[1];
    op.erase(0);
    byte |= 0xf & atou64(op);
    of.write((char *)&byte, 1);
}

void i_iexp(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
    std::string op = v[2];
    op.erase(0);
    uint8_t byte = atou64(op);
    byte <<= 4;
    op = v[1];
    op.erase(0);
    byte |= 0xf & atou64(op);
    of.write((char *)&byte, 1);
}

void i_cpuid(
    std ::vector<std::string> &v,
    std::map<std::string, uint64_t> &st,
    uint64_t spesymcnt,
    std::fstream &of)
{
    uint8_t icode = instcode[v[0]];
    of.write((char *)&icode, 1);
}

std::function<
    void(std::vector<std::string> &, std::map<std::string, uint64_t> &, uint64_t, std::fstream &)>
    instructions[256] = {
        i_nop,
        i_add,
        i_sub,
        i_inc,
        i_dec,
        i_cmp,
        i_and,
        i_or,
        i_not,
        i_xor,
        i_jc,
        i_cc,
        i_r,
        i_ir,
        i_sysc,
        i_sysr,
        i_loop,
        i_chl,
        i_chr,
        i_rol,
        i_ror,
        i_ldi,
        i_ldm,
        i_stm,
        i_ei,
        i_di,
        i_ep,
        i_dp,
        i_mv,
        i_livt,
        i_lkpt,
        i_lupt,
        i_lsrg,
        i_ssrg,
        i_initext,
        i_destext,
        i_in,
        i_out,
        i_cut,
        i_icut,
        i_iexp,
        i_cpuid,
};
