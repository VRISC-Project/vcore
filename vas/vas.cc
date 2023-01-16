/**
 * @file vas.cc
 * @author pointer-to-bios (pointer-to-bios@outlook.com)
 * @brief 单文件编译
 * @version 0.1
 * @date 2023-01-09
 *
 * @copyright Copyright (c) 2022 Random World Studio
 *
 */

#include <iostream>
#include <fstream>
#include <map>
#include <vector>
#include <cstring>

extern "C"
{
#include <getopt.h>
};

#include "len.hh"
#include "assembler.hh"

enum err_code
{
    no_inputfile = -1,
    cannot_open_ofile = -2,
    undefined_command = -3,
    undefined_instruction = -4,
    invalid_symblic = -5,
    sym_or_inst_needed = -6,
    cannot_open_ifile = -7,
    invalid_align_val = -8,
    invalid_start_val = -9,
    undefined_symbolic = -10,
    symbolic_redefined = -11,
};

struct cmd_options
{
    std::string outfile;
    std::string infile;
    std::string format;
    std::string map;
} cmd_opt;

std::fstream infile;
std::fstream outfile;

std::map<std::string, uint64_t> symbolic_table;

std::string nullstring = "";

void deal_with_cmdline(int argc, char **argv);

void build_symblic_table();

void openfile();

void generate();

void output_mapfile();

int main(int argc, char **argv)
{
    cmd_opt.outfile = "";
    cmd_opt.infile = "";
    cmd_opt.format = "";
    cmd_opt.map = "";

    deal_with_cmdline(argc, argv);

    openfile();

    build_symblic_table();

    output_mapfile();

    generate();
}

void output_mapfile()
{
    if (cmd_opt.map == "")
    {
        return;
    }
    std::fstream f(cmd_opt.map, std::ios::out);
    for (std::pair<std::string, uint64_t> p : symbolic_table)
    {
        f << std::hex << p.second << "\t\t" << p.first << std::endl;
    }
    f.close();
}

static std::vector<std::string>
split(const std::string &str, const std::string &delim)
{
    std::vector<std::string> res;
    if ("" == str)
        return res;
    char *strs = new char[str.length() + 1];
    strcpy(strs, str.c_str());
    char *d = new char[delim.length() + 1];
    strcpy(d, delim.c_str());
    char *p = strtok(strs, d);
    while (p)
    {
        std::string s = p;
        res.push_back(s);
        p = strtok(NULL, d);
    }
    delete strs;
    delete d;

    return res;
}

static std::vector<std::string> &
get_line_from(std::fstream &file)
{
    std::string line = "";
    std::vector<std::string> *v = new std::vector<std::string>;
    char c;
    // 读行
    while (!file.eof())
    {
        file.read(&c, 1);
        if (c == '\n')
            break;
        line += c;
    }
    if (line == "")
    {
        return *v;
    }
    // 截掉注释
    int x;
    for (x = 0; x < line.length(); x++)
    {
        if (line[x] == '/')
            break;
    }
    if (x < line.length())
    {
        line = line.substr(0, x);
    }
    // 分割
    *v = split(line, " ,:");
    for (int i = 0; i < (*v).size();)
    {
        if ((*v)[i] == "")
        {
            (*v).erase((*v).begin() + i);
        }
        else
        {
            i++;
        }
    }
    return *v;
}

static bool
is_a_num(char c)
{
    return (c >= '0' && c <= '9');
}

/*
判断是否是数字
是数字返回进制（支持2、8、10、16进制）
 */
int is_number(std::string str)
{
    int carrysys = 10;
    if (!is_a_num(str[0]))
    {
        return false;
    }

    if (str.length() == 1)
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
            return false;
        }
    }
    else if (str[0] == '0')
    {
        carrysys = 8;
    }

    if (str.length() == 2)
    {
        return carrysys;
    }

    for (int i = 2; i < str.length(); i++)
    {
        if (!is_a_num(str[i]))
        {
            if (carrysys == 16 && str[i] >= 'a' && str[i] <= 'f')
            {
                continue;
            }
            else
            {
                return false;
            }
        }
        else if (carrysys == 8)
        {
            if (str[i] == '8' || str[i] == '9')
            {
                return false;
            }
        }
        else if (carrysys == 2)
        {
            if (str[i] != '0' && str[i] != '1')
            {
                return false;
            }
        }
    }
    return carrysys;
}

uint64_t
atou64(std::string &str)
{
    int carrysys = is_number(str);
    if (!carrysys)
    {
        return 0;
    }
    uint64_t num = 0;
    if (carrysys == 2)
    {
        for (int i = 2; i < str.length(); i++)
        {
            num |= str[i] - '0';
            num <<= 1;
        }
        num >>= 1;
    }
    else if (carrysys == 8)
    {
        for (int i = 1; i < str.length(); i++)
        {
            num |= str[i] - '0';
            num <<= 3;
        }
        num >>= 3;
    }
    else if (carrysys == 10)
    {
        for (int i = 0; i < str.length(); i++)
        {
            num += str[i] - '0';
            num *= 10;
        }
        num /= 10;
    }
    else if (carrysys == 16)
    {
        std::map<char, int> cs16_numchar_to_int =
            {
                std::pair('0', 0),
                std::pair('1', 1),
                std::pair('2', 2),
                std::pair('3', 3),
                std::pair('4', 4),
                std::pair('5', 5),
                std::pair('6', 6),
                std::pair('7', 7),
                std::pair('8', 8),
                std::pair('9', 9),
                std::pair('a', 10),
                std::pair('b', 11),
                std::pair('c', 12),
                std::pair('d', 13),
                std::pair('e', 14),
                std::pair('f', 15),
            };
        for (int i = 2; i < str.length(); i++)
        {
            num |= cs16_numchar_to_int[str[i]];
            num <<= 4;
        }
        num >>= 4;
    }
    return num;
}

static void
open_outfile()
{
    if (cmd_opt.outfile == "")
    {
        cmd_opt.outfile = cmd_opt.infile + ".bin";
    }
    outfile.open(cmd_opt.outfile, std::ios::out);
    if (outfile.bad())
    {
        std::cerr << "fatal: Can't open output file." << std::endl;
        exit(err_code::cannot_open_ofile);
    }
}

uint64_t glb_ip;

void generate()
{
    open_outfile();
    infile.close();
    openfile();

    std::vector<std::string> v;
    glb_ip = 0;
    uint16_t align = 1;
    uint64_t special_symcnt = 0;
    uint64_t linenum = 0;
    while (!infile.eof())
    {
        v = get_line_from(infile);
        linenum++;
        if (v.size() == 0)
        {
            continue;
        }
        if (instcode.find(v[0]) == instcode.end())
        {
            if (glb_ip % align != 0 && v[0][0] != '@')
            {
                uint64_t previp = glb_ip;
                glb_ip = (glb_ip / align + 1) * align;
                for (int i = 0; i < glb_ip - previp; i++)
                {
                    uint8_t num = 0;
                    outfile.write((char *)(&num), 1);
                }
            }
            if (v[0][0] == '.')
            {
                if (v[0] == ".start")
                {
                    glb_ip = atou64(v[1]);
                }
                else if (v[0] == ".align")
                {
                    align = atou64(v[1]);
                }
                else if (v[0] == ".b")
                {
                    for (int i = 1; i < v.size(); i++)
                    {
                        if (is_number(v[i]))
                        {
                            uint8_t num = atou64(v[i]);
                            outfile.write((char *)(&num), 1);
                        }
                        else
                        {
                            if (symbolic_table.find(v[i]) == symbolic_table.end())
                            {
                                std::cerr
                                    << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                    << "Undefined symbolic: " << v[i] << std::endl;
                                exit(err_code::undefined_symbolic);
                            }
                            uint64_t num = symbolic_table[v[i]];
                            outfile.write((char *)&num, 1);
                        }
                    }
                    glb_ip += v.size() - 1;
                }
                else if (v[0] == ".w")
                {
                    for (int i = 1; i < v.size(); i++)
                    {
                        if (is_number(v[i]))
                        {
                            uint16_t num = atou64(v[i]);
                            outfile.write((char *)(&num), 2);
                        }
                        else
                        {
                            if (symbolic_table.find(v[i]) == symbolic_table.end())
                            {
                                std::cerr
                                    << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                    << "Undefined symbolic: " << v[i] << std::endl;
                                exit(err_code::undefined_symbolic);
                            }
                            uint64_t num = symbolic_table[v[i]];
                            outfile.write((char *)&num, 2);
                        }
                    }
                    glb_ip += 2 * (v.size() - 1);
                }
                else if (v[0] == ".d")
                {
                    for (int i = 1; i < v.size(); i++)
                    {
                        if (is_number(v[i]))
                        {
                            uint32_t num = atou64(v[i]);
                            outfile.write((char *)(&num), 4);
                        }
                        else
                        {
                            if (symbolic_table.find(v[i]) == symbolic_table.end())
                            {
                                std::cerr
                                    << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                    << "Undefined symbolic: " << v[i] << std::endl;
                                exit(err_code::undefined_symbolic);
                            }
                            uint64_t num = symbolic_table[v[i]];
                            outfile.write((char *)&num, 4);
                        }
                    }
                    glb_ip += 4 * (v.size() - 1);
                }
                else if (v[0] == ".q")
                {
                    for (int i = 1; i < v.size(); i++)
                    {
                        if (is_number(v[i]))
                        {
                            uint64_t num = atou64(v[i]);
                            outfile.write((char *)(&num), 8);
                        }
                        else
                        {
                            if (symbolic_table.find(v[i]) == symbolic_table.end())
                            {
                                std::cerr
                                    << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                    << "Undefined symbolic: " << v[i] << std::endl;
                                exit(err_code::undefined_symbolic);
                            }
                            uint64_t num = symbolic_table[v[i]];
                            outfile.write((char *)&num, 8);
                        }
                    }
                    glb_ip += 8 * (v.size() - 1);
                }
            }
            else if (is_number(v[0]))
            {
                uint64_t time = atou64(v[0]);
                if (instcode.find(v[1]) == instcode.end())
                {
                    if (glb_ip % align != 0)
                    {
                        uint64_t previp = glb_ip;
                        glb_ip = (glb_ip / align + 1) * align;
                        for (int i = 0; i < glb_ip - previp; i++)
                        {
                            uint8_t num = 0;
                            outfile.write((char *)(&num), 1);
                        }
                    }
                    if (v[1][0] == '.')
                    {
                        if (v[1] == ".b")
                        {
                            for (int tmp = 0; tmp < time; tmp++)
                            {
                                for (int i = 2; i < v.size(); i++)
                                {
                                    if (is_number(v[i]))
                                    {
                                        uint8_t num = atou64(v[i]);
                                        outfile.write((char *)(&num), 1);
                                    }
                                    else
                                    {
                                        if (symbolic_table.find(v[i]) == symbolic_table.end())
                                        {
                                            std::cerr
                                                << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                                << "Undefined symbolic: " << v[i] << std::endl;
                                            exit(err_code::undefined_symbolic);
                                        }
                                        uint64_t num = symbolic_table[v[i]];
                                        outfile.write((char *)&num, 1);
                                    }
                                }
                            }
                            glb_ip += time * (v.size() - 2);
                        }
                        else if (v[1] == ".w")
                        {
                            for (int tmp = 0; tmp < time; tmp++)
                            {
                                for (int i = 2; i < v.size(); i++)
                                {
                                    if (is_number(v[i]))
                                    {
                                        uint16_t num = atou64(v[i]);
                                        outfile.write((char *)(&num), 2);
                                    }
                                    else
                                    {
                                        if (symbolic_table.find(v[i]) == symbolic_table.end())
                                        {
                                            std::cerr
                                                << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                                << "Undefined symbolic: " << v[i] << std::endl;
                                            exit(err_code::undefined_symbolic);
                                        }
                                        uint64_t num = symbolic_table[v[i]];
                                        outfile.write((char *)&num, 2);
                                    }
                                }
                            }
                            glb_ip += 2 * time * (v.size() - 2);
                        }
                        else if (v[1] == ".d")
                        {
                            for (int tmp = 0; tmp < time; tmp++)
                            {
                                for (int i = 2; i < v.size(); i++)
                                {
                                    if (is_number(v[i]))
                                    {
                                        uint32_t num = atou64(v[i]);
                                        outfile.write((char *)(&num), 4);
                                    }
                                    else
                                    {
                                        if (symbolic_table.find(v[i]) == symbolic_table.end())
                                        {
                                            std::cerr
                                                << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                                << "Undefined symbolic: " << v[i] << std::endl;
                                            exit(err_code::undefined_symbolic);
                                        }
                                        uint64_t num = symbolic_table[v[i]];
                                        outfile.write((char *)&num, 4);
                                    }
                                }
                            }
                            glb_ip += 4 * time * (v.size() - 2);
                        }
                        else if (v[1] == ".q")
                        {
                            for (int tmp = 0; tmp < time; tmp++)
                            {
                                for (int i = 2; i < v.size(); i++)
                                {
                                    if (is_number(v[i]))
                                    {
                                        uint64_t num = atou64(v[i]);
                                        outfile.write((char *)(&num), 8);
                                    }
                                    else
                                    {
                                        if (symbolic_table.find(v[i]) == symbolic_table.end())
                                        {
                                            std::cerr
                                                << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                                << "Undefined symbolic: " << v[i] << std::endl;
                                            exit(err_code::undefined_symbolic);
                                        }
                                        uint64_t num = symbolic_table[v[i]];
                                        outfile.write((char *)&num, 8);
                                    }
                                }
                            }
                            glb_ip += 8 * time * (v.size() - 2);
                        }
                    }
                }
                else
                {
                    v.erase(v.begin());
                    instructions[instcode[v[1]]](v, symbolic_table, special_symcnt, outfile);
                    glb_ip += time * len[instcode[v[1]]](v, symbolic_table, special_symcnt);
                }
            }
            else if (v[0][0] == '@')
            {
                special_symcnt++;
            }
        }
        else
        {
            instructions[instcode[v[0]]](v, symbolic_table, special_symcnt, outfile);
            glb_ip += len[instcode[v[0]]](v, symbolic_table, special_symcnt);
        }
    }
}

void build_symblic_table()
{
    infile.seekg(0);

    std::vector<std::string> v;
    uint64_t ip = 0;
    uint16_t align = 1;
    uint64_t special_symcnt = 0;
    uint64_t linenum = 0;
    while (!infile.eof())
    {
        v = get_line_from(infile);
        linenum++;
        if (v.size() == 0)
        {
            continue;
        }
        if (instcode.find(v[0]) == instcode.end())
        {
            if (ip % align != 0 && v[0][0] != '@')
            {
                ip = (ip / align + 1) * align;
            }
            if (v[0][0] == '.')
            {
                if (v[0] == ".start")
                {
                    if (!is_number(v[1]))
                    {
                        std::cerr
                            << "error: " << cmd_opt.infile << ": " << linenum << ": "
                            << "Invalid start value: " << v[1] << std::endl;
                        exit(err_code::invalid_start_val);
                    }
                    ip = atou64(v[1]);
                }
                else if (v[0] == ".align")
                {
                    if (!is_number(v[1]))
                    {
                        std::cerr
                            << "error: " << cmd_opt.infile << ": " << linenum << ": "
                            << "Invalid align value: " << v[1] << std::endl;
                        exit(err_code::invalid_align_val);
                    }
                    align = atou64(v[1]);
                }
                else if (v[0] == ".b")
                {
                    ip += v.size() - 1;
                }
                else if (v[0] == ".w")
                {
                    ip += 2 * (v.size() - 1);
                }
                else if (v[0] == ".d")
                {
                    ip += 4 * (v.size() - 1);
                }
                else if (v[0] == ".q")
                {
                    ip += 8 * (v.size() - 1);
                }
                else
                {
                    std::cerr
                        << "error: " << cmd_opt.infile << ": " << linenum << ": "
                        << "Undefined assembler command: " << v[0] << std::endl;
                    exit(err_code::undefined_command);
                }
            }
            else if (is_number(v[0]))
            {
                uint64_t time = atou64(v[0]);
                if (instcode.find(v[1]) == instcode.end())
                {
                    if (ip % align != 0)
                    {
                        ip = (ip / align + 1) * align;
                    }
                    if (v[1][0] == '.')
                    {
                        if (v[1] == ".b")
                        {
                            ip += time * (v.size() - 2);
                        }
                        else if (v[1] == ".w")
                        {
                            ip += 2 * time * (v.size() - 2);
                        }
                        else if (v[1] == ".d")
                        {
                            ip += 4 * time * (v.size() - 2);
                        }
                        else if (v[1] == ".q")
                        {
                            ip += 8 * time * (v.size() - 2);
                        }
                        else
                        {
                            std::cerr
                                << "error: " << cmd_opt.infile << ": " << linenum << ": "
                                << "Undefined assembler command: " << v[1] << std::endl;
                            exit(err_code::undefined_command);
                        }
                    }
                    else
                    {
                        std::cerr
                            << "error: " << cmd_opt.infile << ": " << linenum << ": "
                            << "Symblic or Instruction needed after repeating command: "
                            << v[1] << std::endl;
                        exit(err_code::sym_or_inst_needed);
                    }
                }
                else
                {
                    ip += time * len[instcode[v[1]]](v, symbolic_table, special_symcnt);
                }
            }
            else if (is_number(nullstring + v[0][0]))
            {
                std::cerr
                    << "error: " << cmd_opt.infile << ": " << linenum << ": "
                    << "An invalid symblic: " << v[0] << std::endl;
                exit(err_code::invalid_symblic);
            }
            else
            {
                if (v[0][0] != '@')
                {
                    // if (symbolic_table.find(v[0]) != symbolic_table.end())
                    // {
                    //     std::cerr
                    //         << "error: " << cmd_opt.infile << ": " << linenum << ": "
                    //         << "Symbolic redefined: " << v[0] << std::endl;
                    //     exit(err_code::symbolic_redefined);
                    // }
                    symbolic_table[v[0]] = ip;
                }
                else
                {
                    symbolic_table[std::string("@") + std::to_string(special_symcnt)] = ip;
                    special_symcnt++;
                }
            }
        }
        else
        {
            ip += len[instcode[v[0]]](v, symbolic_table, special_symcnt);
        }
    }
}

void openfile()
{
    if (cmd_opt.infile == "")
    {
        std::cerr << "fatal: No input file." << std::endl;
        exit(err_code::no_inputfile);
    }
    infile.open(cmd_opt.infile, std::ios::in);
    if (infile.bad())
    {
        std::cerr << "fatal: Can't open input file." << std::endl;
        exit(err_code::cannot_open_ifile);
    }
}

void deal_with_cmdline(int argc, char **argv)
{
    int opt;
    while ((opt = getopt(argc, argv, "o:i:f:m:")) != -1)
    {
        switch (opt)
        {
        case 'o':
            cmd_opt.outfile = optarg;
            break;

        case 'i':
            cmd_opt.infile = optarg;
            break;

        case 'f':
            cmd_opt.format = optarg;
            break;

        case 'm':
            cmd_opt.map = optarg;
            break;

        default:
            break;
        }
    }
}
