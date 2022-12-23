#include <iostream>
#include <fstream>
#include <list>

std::string source, target;
std::fstream srcf, tarf;

void assembly();

int main(int argc, char **argv)
{
    source = argv[1];
    target = argv[2];
    srcf.open(source, std::ios::in);
    source = "";
    while (!srcf.eof())
    {
        std::string tmp;
        srcf >> tmp;
        source += tmp;
        source += '\n';
    }
    srcf.close();
    assembly();
}

struct inst
{
    uint64_t size;
    uint64_t inst;
    std::string addr1, addr2;
};

std::string format;
uint64_t start;
uint64_t align;

std::list<inst> insts;

void assembly()
{
    
}
