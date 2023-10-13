use crate::utils::memory::ReadWrite;

use super::vcore::{ BitOptions, ConditionCode, FlagRegFlag, InterruptId, Vcore, VcoreInstruction };

/// ## 基本指令集
pub const BASE: [Option<VcoreInstruction>; 64] = [
    Some((i_nop, 1)),
    Some((i_add, 3)),
    Some((i_sub, 3)),
    Some((i_inc, 2)),
    Some((i_dec, 2)),
    Some((i_shl, 3)),
    Some((i_shr, 3)),
    Some((i_rol, 3)),
    Some((i_ror, 3)),
    Some((i_cmp, 2)),
    Some((i_and, 3)),
    Some((i_or, 3)),
    Some((i_not, 2)),
    Some((i_xor, 3)),
    None,
    None,
    Some((i_jc, 10)),
    Some((i_cc, 10)),
    Some((i_r, 1)),
    Some((i_loop, 6)),
    Some((i_ir, 2)),
    Some((i_sysc, 1)),
    Some((i_sysr, 1)),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some((i_ldi, 10)),
    Some((i_ldm, 3)),
    Some((i_stm, 3)),
    Some((i_in, 3)),
    Some((i_out, 3)),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some((i_ei, 1)),
    Some((i_di, 1)),
    Some((i_ep, 1)),
    Some((i_dp, 1)),
    Some((i_livt, 2)),
    Some((i_lkpt, 2)),
    Some((i_lupt, 2)),
    Some((i_lscp, 2)),
    Some((i_lipdump, 2)),
    Some((i_lflagdump, 2)),
    Some((i_sipdump, 2)),
    Some((i_sflagdump, 2)),
    Some((i_cpuid, 1)),
    None,
    None,
    None,
];

trait Oprand {
    fn higher(&self) -> u8;
    fn lower(&self) -> u8;
}

impl Oprand for u8 {
    fn higher(&self) -> u8 {
        *self >> 4
    }

    fn lower(&self) -> u8 {
        *self & 0x0f
    }
}

///## 基本指令集的实现
/// 参数`inst`提供一段内存，这段内存是一个vrisc指令
/// 返回一个`u64`类型，表示`ip`寄存器需要增加的数，有的指令
/// 会自行修改ip寄存器，实际返回的值可能与指令长度不符，所以需要
/// 指令返回ip寄存器需要增加的长度
// TODO
pub fn i_nop(_inst: &[u8], core: &mut Vcore) -> u64 {
    core.nopflag = true;
    1
}

pub fn i_add(inst: &[u8], core: &mut Vcore) -> u64 {
    match inst[2].higher() {
        0 => {
            core.regs.x[inst[2].lower() as usize] = ((core.regs.x[inst[1].lower() as usize] as u8) +
                (core.regs.x[inst[1].higher() as usize] as u8)) as u64;
        }
        1 => {
            core.regs.x[inst[2].lower() as usize] = ((
                core.regs.x[inst[1].lower() as usize] as u16
            ) + (core.regs.x[inst[1].higher() as usize] as u16)) as u64;
        }
        2 => {
            core.regs.x[inst[2].lower() as usize] = ((
                core.regs.x[inst[1].lower() as usize] as u32
            ) + (core.regs.x[inst[1].higher() as usize] as u32)) as u64;
        }
        3 => {
            core.regs.x[inst[2].lower() as usize] =
                core.regs.x[inst[1].lower() as usize] + core.regs.x[inst[1].higher() as usize];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(
        core.regs.x[inst[1].lower() as usize].max(core.regs.x[inst[1].higher() as usize]),
        core.regs.x[inst[2].lower() as usize]
    );
    3
}

pub fn i_sub(inst: &[u8], core: &mut Vcore) -> u64 {
    match inst[2].higher() {
        0 => {
            core.regs.x[inst[2].lower() as usize] = ((core.regs.x[inst[1].lower() as usize] as u8) -
                (core.regs.x[inst[1].higher() as usize] as u8)) as u64;
        }
        1 => {
            core.regs.x[inst[2].lower() as usize] = ((
                core.regs.x[inst[1].lower() as usize] as u16
            ) - (core.regs.x[inst[1].higher() as usize] as u16)) as u64;
        }
        2 => {
            core.regs.x[inst[2].lower() as usize] = ((
                core.regs.x[inst[1].lower() as usize] as u32
            ) - (core.regs.x[inst[1].higher() as usize] as u32)) as u64;
        }
        3 => {
            core.regs.x[inst[2].lower() as usize] =
                core.regs.x[inst[1].lower() as usize] - core.regs.x[inst[1].higher() as usize];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(
        core.regs.x[inst[1].lower() as usize].max(core.regs.x[inst[1].higher() as usize]),
        core.regs.x[inst[2].lower() as usize]
    );
    if core.regs.flag.bit_get(FlagRegFlag::Overflow) {
        core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    } else {
        core.regs.flag.bit_set(FlagRegFlag::Overflow);
    }
    3
}

pub fn i_inc(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].lower() as usize];
    match inst[1].higher() {
        0 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xff) as u8;
            opr += 1;
            core.regs.x[inst[1].lower() as usize] %= 0x100;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        1 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xffff) as u16;
            opr += 1;
            core.regs.x[inst[1].lower() as usize] %= 0x1_0000;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        2 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xff) as u32;
            opr += 1;
            core.regs.x[inst[1].lower() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        3 => {
            core.regs.x[inst[1].lower() as usize] += 1;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].lower() as usize]);
    2
}

pub fn i_dec(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].lower() as usize];
    match inst[1].higher() {
        0 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xff) as u8;
            opr -= 1;
            core.regs.x[inst[1].lower() as usize] %= 0x100;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        1 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xffff) as u16;
            opr -= 1;
            core.regs.x[inst[1].lower() as usize] %= 0x1_0000;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        2 => {
            let mut opr = (core.regs.x[inst[1].lower() as usize] & 0xff) as u32;
            opr -= 1;
            core.regs.x[inst[1].lower() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].lower() as usize] |= opr as u64;
        }
        3 => {
            core.regs.x[inst[1].lower() as usize] -= 1;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].lower() as usize]);
    if core.regs.flag.bit_get(FlagRegFlag::Overflow) {
        core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    } else {
        core.regs.flag.bit_set(FlagRegFlag::Overflow);
    }
    2
}

pub fn i_shl(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].higher() as usize];
    let shbits = core.regs.x[inst[1].lower() as usize];
    match inst[2].lower() {
        0 => {
            let mut r2 = reg_before as u8;
            r2 <<= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x100;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        1 => {
            let mut r2 = reg_before as u16;
            r2 <<= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        2 => {
            let mut r2 = reg_before as u32;
            r2 <<= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        3 => {
            core.regs.x[inst[1].higher() as usize] <<= shbits;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].higher() as usize]);
    3
}

pub fn i_shr(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].higher() as usize];
    let shbits = core.regs.x[inst[1].lower() as usize];
    match inst[2].lower() {
        0 => {
            let mut r2 = reg_before as u8;
            r2 >>= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x100;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        1 => {
            let mut r2 = reg_before as u16;
            r2 >>= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        2 => {
            let mut r2 = reg_before as u32;
            r2 >>= shbits;
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].higher() as usize] |= r2 as u64;
        }
        3 => {
            core.regs.x[inst[1].higher() as usize] >>= shbits;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].higher() as usize]);
    3
}

pub fn i_rol(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].higher() as usize];
    let shbits = core.regs.x[inst[1].lower() as usize];
    match inst[2].lower() {
        0 => {
            let mut r2 = 0u8;
            r2 |= (reg_before as u8) << shbits;
            r2 |= (reg_before as u8) >> (8 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x100;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        1 => {
            let mut r2 = 0u16;
            r2 |= (reg_before as u16) << shbits;
            r2 |= (reg_before as u16) >> (16 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        2 => {
            let mut r2 = 0u32;
            r2 |= (reg_before as u32) << shbits;
            r2 |= (reg_before as u32) >> (32 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        3 => {
            let mut r2 = 0u64;
            r2 |= reg_before << shbits;
            r2 |= reg_before >> (64 - shbits);
            core.regs.x[inst[1].higher() as usize] = r2;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].higher() as usize]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    3
}

pub fn i_ror(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].higher() as usize];
    let shbits = core.regs.x[inst[1].lower() as usize];
    match inst[2].lower() {
        0 => {
            let mut r2 = 0u8;
            r2 |= (reg_before as u8) >> shbits;
            r2 |= (reg_before as u8) << (8 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x100;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        1 => {
            let mut r2 = 0u16;
            r2 |= (reg_before as u16) >> shbits;
            r2 |= (reg_before as u16) << (16 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        2 => {
            let mut r2 = 0u32;
            r2 |= (reg_before as u32) >> shbits;
            r2 |= (reg_before as u32) << (32 - shbits);
            core.regs.x[inst[1].higher() as usize] %= 0x1_0000_0000;
            core.regs.x[inst[1].higher() as usize] = r2 as u64;
        }
        3 => {
            let mut r2 = 0u64;
            r2 |= reg_before >> shbits;
            r2 |= reg_before << (64 - shbits);
            core.regs.x[inst[1].higher() as usize] = r2;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].higher() as usize]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    3
}

pub fn i_cmp(inst: &[u8], core: &mut Vcore) -> u64 {
    let r1 = inst[1].lower() as usize;
    let r2 = inst[1].higher() as usize;
    //相等标志
    if core.regs.x[r1] == core.regs.x[r2] {
        core.regs.flag.bit_set(FlagRegFlag::Equal);
        core.regs.flag.bit_reset(FlagRegFlag::Higher);
        core.regs.flag.bit_reset(FlagRegFlag::Lower);
        core.regs.flag.bit_reset(FlagRegFlag::Bigger);
        core.regs.flag.bit_reset(FlagRegFlag::Smaller);
    } else {
        core.regs.flag.bit_reset(FlagRegFlag::Equal);
        //无符号大于小于
        if core.regs.x[r1] > core.regs.x[r2] {
            core.regs.flag.bit_set(FlagRegFlag::Higher);
            core.regs.flag.bit_reset(FlagRegFlag::Lower);
        } else {
            core.regs.flag.bit_set(FlagRegFlag::Lower);
            core.regs.flag.bit_reset(FlagRegFlag::Higher);
        }
        //有符号大于小于
        if (core.regs.x[r1] as i64) > (core.regs.x[r2] as i64) {
            core.regs.flag.bit_set(FlagRegFlag::Higher);
            core.regs.flag.bit_reset(FlagRegFlag::Lower);
        } else {
            core.regs.flag.bit_set(FlagRegFlag::Lower);
            core.regs.flag.bit_reset(FlagRegFlag::Higher);
        }
    }
    2
}

pub fn i_and(inst: &[u8], core: &mut Vcore) -> u64 {
    let r1 = inst[1].lower() as usize;
    let r2 = inst[1].higher() as usize;
    let r3 = inst[2].lower() as usize;
    let reg_before = core.regs.x[r1].max(core.regs.x[r2]);
    match inst[2].higher() {
        0 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u8) & (core.regs.x[r2] as u8)) as u64;
        }
        1 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u16) & (core.regs.x[r2] as u16)) as u64;
        }
        2 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u32) & (core.regs.x[r2] as u32)) as u64;
        }
        3 => {
            core.regs.x[r3] = core.regs.x[r1] & core.regs.x[r2];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[r3]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    3
}

pub fn i_or(inst: &[u8], core: &mut Vcore) -> u64 {
    let r1 = inst[1].lower() as usize;
    let r2 = inst[1].higher() as usize;
    let r3 = inst[2].lower() as usize;
    let reg_before = core.regs.x[r1].max(core.regs.x[r2]);
    match inst[2].higher() {
        0 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u8) | (core.regs.x[r2] as u8)) as u64;
        }
        1 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u16) | (core.regs.x[r2] as u16)) as u64;
        }
        2 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u32) | (core.regs.x[r2] as u32)) as u64;
        }
        3 => {
            core.regs.x[r3] = core.regs.x[r1] | core.regs.x[r2];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[r3]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    3
}

pub fn i_not(inst: &[u8], core: &mut Vcore) -> u64 {
    let r1 = inst[1].lower() as usize;
    let r2 = inst[1].higher() as usize;
    match inst[2].lower() {
        0 => {
            core.regs.x[r2] = !(core.regs.x[r1] as u8) as u64;
        }
        1 => {
            core.regs.x[r2] = !(core.regs.x[r1] as u16) as u64;
        }
        2 => {
            core.regs.x[r2] = !(core.regs.x[r1] as u32) as u64;
        }
        3 => {
            core.regs.x[r2] = !core.regs.x[r2];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(core.regs.x[r1], core.regs.x[r2]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    2
}

pub fn i_xor(inst: &[u8], core: &mut Vcore) -> u64 {
    let r1 = inst[1].lower() as usize;
    let r2 = inst[1].higher() as usize;
    let r3 = inst[2].lower() as usize;
    let reg_before = core.regs.x[r1].max(core.regs.x[r2]);
    match inst[2].higher() {
        0 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u8) ^ (core.regs.x[r2] as u8)) as u64;
        }
        1 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u16) ^ (core.regs.x[r2] as u16)) as u64;
        }
        2 => {
            core.regs.x[r3] = ((core.regs.x[r1] as u32) ^ (core.regs.x[r2] as u32)) as u64;
        }
        3 => {
            core.regs.x[r3] = core.regs.x[r1] ^ core.regs.x[r2];
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[r3]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    3
}

pub fn i_jc(inst: &[u8], core: &mut Vcore) -> u64 {
    if core.regs.flag.satisfies_condition(ConditionCode::new(inst[1].higher())) {
        match inst[1].lower() {
            0 => {
                core.regs.ip = ((inst[2] as u16) | ((inst[3] as u16) << 8)) as u64;
            }
            1 => {
                core.regs.ip = ((inst[2] as u32) |
                    ((inst[3] as u32) << 8) |
                    ((inst[4] as u32) << 16) |
                    ((inst[5] as u32) << 24)) as u64;
            }
            2 => {
                core.regs.ip = ((inst[2] as u64) |
                    ((inst[3] as u64) << 8) |
                    ((inst[4] as u64) << 16) |
                    ((inst[5] as u64) << 24) |
                    ((inst[6] as u64) << 32) |
                    ((inst[7] as u64) << 40) |
                    ((inst[8] as u64) << 48) |
                    ((inst[9] as u64) << 56)) as u64;
                println!("{:x}", core.regs.ip);
            }
            _ => (),
        }
        core.transferred = true;
    }
    0
}

pub fn i_cc(inst: &[u8], core: &mut Vcore) -> u64 {
    if core.regs.flag.satisfies_condition(ConditionCode::new(inst[1].higher())) {
        core.regs.ipdump = core.regs.ip;
        match inst[1].lower() {
            0 => {
                core.regs.ip = ((inst[2] as u16) | ((inst[3] as u16) << 8)) as u64;
            }
            1 => {
                core.regs.ip = ((inst[2] as u32) |
                    ((inst[3] as u32) << 8) |
                    ((inst[4] as u32) << 16) |
                    ((inst[5] as u32) << 24)) as u64;
            }
            2 => {
                core.regs.ip = ((inst[2] as u64) |
                    ((inst[3] as u64) << 8) |
                    ((inst[4] as u64) << 16) |
                    ((inst[5] as u64) << 24) |
                    ((inst[6] as u64) << 32) |
                    ((inst[7] as u64) << 40) |
                    ((inst[8] as u64) << 48) |
                    ((inst[9] as u64) << 56)) as u64;
            }
            _ => (),
        }
        core.transferred = true;
    }
    0
}

pub fn i_r(_inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.ip = core.regs.ipdump;
    core.transferred = true;
    0
}

pub fn i_loop(inst: &[u8], core: &mut Vcore) -> u64 {
    if core.regs.x[inst[1] as usize] != 0 {
        let target =
            (inst[2] as u32) |
            ((inst[3] as u32) << 8) |
            ((inst[4] as u32) << 16) |
            ((inst[5] as u32) << 24);
        let target = (if (target & (1 << 31)) != 0 {
            -(target as i32)
        } else {
            target as i32
        }) as i64 as u64;
        core.regs.ip = target;
        core.transferred = true;
        0
    } else {
        6
    }
}

pub fn i_ir(inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    if inst[1] == 0 {
        core.reset();
    } else if inst[1] == 1 {
        core.regs.ip = core.regs.ipdump;
        core.regs.flag = core.regs.flagdump;
        core.transferred = true;
    } else {
        core.intctler.interrupt(InterruptId::InvalidInstruction);
    }
    0
}

pub fn i_sysc(_inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.ipdump = core.regs.ip;
    core.regs.flagdump = core.regs.flag;
    core.regs.ip = core.regs.scp;
    core.regs.flag.bit_reset(FlagRegFlag::Privilege);
    core.transferred = true;
    0
}

pub fn i_sysr(_inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.ip = core.regs.ipdump;
    core.regs.flag = core.regs.flagdump;
    core.transferred = true;
    1
}

pub fn i_ldi(inst: &[u8], core: &mut Vcore) -> u64 {
    let reg_before = core.regs.x[inst[1].higher() as usize];
    match inst[1].lower() {
        0 => {
            let imm = inst[2];
            core.regs.x[inst[1].higher() as usize] = imm as u64;
        }
        1 => {
            let imm = (inst[2] as u16) | ((inst[3] as u16) << 8);
            core.regs.x[inst[1].higher() as usize] = imm as u64;
        }
        2 => {
            let imm =
                (inst[2] as u32) |
                ((inst[3] as u32) << 8) |
                ((inst[4] as u32) << 16) |
                ((inst[5] as u32) << 24);
            core.regs.x[inst[1].higher() as usize] = imm as u64;
        }
        3 => {
            let imm =
                (inst[2] as u64) |
                ((inst[3] as u64) << 8) |
                ((inst[4] as u64) << 16) |
                ((inst[5] as u64) << 24) |
                ((inst[6] as u64) << 32) |
                ((inst[7] as u64) << 40) |
                ((inst[8] as u64) << 48) |
                ((inst[9] as u64) << 56);
            core.regs.x[inst[1].higher() as usize] = imm as u64;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(reg_before, core.regs.x[inst[1].higher() as usize]);
    core.regs.flag.bit_reset(FlagRegFlag::Overflow);
    match inst[1].lower() {
        0 => 3,
        1 => 4,
        2 => 6,
        3 => 10,
        _ => 0,
    }
}

pub fn i_ldm(inst: &[u8], core: &mut Vcore) -> u64 {
    let src = core.regs.x[inst[1].lower() as usize];
    let src = match
        core.memory.address(src, core.regs.flag, core.regs.kpt, core.regs.upt, ReadWrite::Read)
    {
        Ok(addr) => addr,
        Err(err) =>
            match err {
                crate::utils::memory::AddressError::OverSized(addr) => {
                    core.intctler.interrupt(InterruptId::InaccessibleAddress);
                    core.regs.imsg = addr;
                    // 无法访问的内存的情况下，要么地址超过内存大小，要么页被交换
                    // 因此要么处理之后重新运行，要么进程直接停止
                    // 所以返回0即可
                    return 0;
                }
                crate::utils::memory::AddressError::WrongPrivilege => {
                    core.intctler.interrupt(InterruptId::WrongPrivilege);
                    core.regs.imsg = src;
                    // 跨特权级访问，无法恢复
                    // 程序直接停止，不能继续运行
                    return 0;
                }
                crate::utils::memory::AddressError::Unreadable => {
                    core.intctler.interrupt(InterruptId::PageOrTableUnreadable);
                    core.regs.imsg = core.regs.imsg;
                    return 0;
                }
                crate::utils::memory::AddressError::Unwritable => {
                    panic!("出现了意外情况，在读寻址时返回了不可写错误")
                }
                crate::utils::memory::AddressError::Ineffective => {
                    core.intctler.interrupt(InterruptId::InaccessibleAddress);
                    core.regs.imsg = src;
                    return 0;
                }
            }
    };
    let src = core.memory().borrow().slice(src, 8);
    let src =
        (src[0] as u64) |
        ((src[1] as u64) << 8) |
        ((src[2] as u64) << 16) |
        ((src[3] as u64) << 24) |
        ((src[4] as u64) << 32) |
        ((src[5] as u64) << 40) |
        ((src[6] as u64) << 48) |
        ((src[7] as u64) << 56);
    match inst[2] {
        0 => {
            let src = src as u8;
            core.regs.x[inst[1].higher() as usize] = src as u64;
        }
        1 => {
            let src = src as u16;
            core.regs.x[inst[1].higher() as usize] = src as u64;
        }
        2 => {
            let src = src as u32;
            core.regs.x[inst[1].higher() as usize] = src as u64;
        }
        3 => {
            core.regs.x[inst[1].higher() as usize] = src;
        }
        _ => (),
    }
    core.regs.flag.mark_symbol(src, src);
    3
}

pub fn i_stm(inst: &[u8], core: &mut Vcore) -> u64 {
    let src = core.regs.x[inst[1].lower() as usize];
    let mut src = match
        core.memory.address(src, core.regs.flag, core.regs.kpt, core.regs.upt, ReadWrite::Write)
    {
        Ok(addr) => addr,
        Err(err) =>
            match err {
                crate::utils::memory::AddressError::OverSized(addr) => {
                    core.intctler.interrupt(InterruptId::InaccessibleAddress);
                    core.regs.imsg = addr;
                    // 无法访问的内存的情况下，要么地址超过内存大小，要么页被交换
                    // 因此要么处理之后重新运行，要么进程直接停止
                    // 所以返回0即可
                    return 0;
                }
                crate::utils::memory::AddressError::WrongPrivilege => {
                    core.intctler.interrupt(InterruptId::WrongPrivilege);
                    core.regs.imsg = src;
                    // 跨特权级访问，无法恢复
                    // 程序直接停止，不能继续运行
                    return 0;
                }
                crate::utils::memory::AddressError::Unreadable => {
                    panic!("出现了意外情况，在写寻址时返回了不可读错误")
                }
                crate::utils::memory::AddressError::Unwritable => {
                    core.intctler.interrupt(InterruptId::PageOrTableUnwritable);
                    core.regs.imsg = core.regs.imsg;
                    return 0;
                }
                crate::utils::memory::AddressError::Ineffective => {
                    core.intctler.interrupt(InterruptId::InaccessibleAddress);
                    core.regs.imsg = src;
                    return 0;
                }
            }
    };
    let src = match inst[2] {
        0 => vec![(src & 0xff) as u8],
        1 => {
            let mut v = Vec::new();
            for _ in 0..2 {
                v.push((src & 0xff) as u8);
                src >>= 8;
            }
            v
        }
        2 => {
            let mut v = Vec::new();
            for _ in 0..4 {
                v.push((src & 0xff) as u8);
                src >>= 8;
            }
            v
        }
        3 => {
            let mut v = Vec::new();
            for _ in 0..8 {
                v.push((src & 0xff) as u8);
                src >>= 8;
            }
            v
        }
        _ => vec![],
    };
    core.memory.borrow_mut().write_slice(core.regs.x[inst[1].higher() as usize], &src);
    core.regs.flag.mark_symbol(
        core.regs.x[inst[1].lower() as usize],
        core.regs.x[inst[1].lower() as usize]
    );
    3
}

pub fn i_in(_inst: &[u8], _core: &mut Vcore) -> u64 {
    todo!();
}

pub fn i_out(_inst: &[u8], _core: &mut Vcore) -> u64 {
    todo!();
}

pub fn i_ei(_inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.flag.bit_set(FlagRegFlag::InterruptEnabled);
    1
}

pub fn i_di(_inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.flag.bit_reset(FlagRegFlag::InterruptEnabled);
    1
}

pub fn i_ep(_inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.flag.bit_set(FlagRegFlag::PagingEnabled);
    1
}

pub fn i_dp(_inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.flag.bit_reset(FlagRegFlag::PagingEnabled);
    1
}

pub fn i_livt(inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.ivt = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_lkpt(inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.kpt = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_lupt(inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.upt = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_lscp(inst: &[u8], core: &mut Vcore) -> u64 {
    if !core.privilege_test() {
        return 0;
    }
    core.regs.scp = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_lipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.ipdump = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_lflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.flagdump = core.regs.x[inst[1].lower() as usize];
    2
}

pub fn i_sipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.x[inst[1].lower() as usize] = core.regs.ipdump;
    2
}

pub fn i_sflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    core.regs.x[inst[1].lower() as usize] = core.regs.flagdump;
    2
}

pub fn i_cpuid(_inst: &[u8], core: &mut Vcore) -> u64 {
    match core.regs.x[0] {
        0 => {
            // 52 57 53 20 56 72 69 73 | 63 20 56 63 6F 72 65 20 | 30 2E 32 2E 30
            // RWS Vrisc Vcore 0.2.0
            core.regs.x[0] = 0x7369_7256_2053_5752;
            core.regs.x[1] = 0x2065_726f_6356_2063;
            core.regs.x[2] = 0x0000_0030_2e32_2e30;
            core.regs.x[3] = 0x0000_0000_0000_0000;
        }
        1 => {
            core.regs.x[0] = core.total_core() as u64;
        }
        2 => {
            core.regs.x[0] = core.id() as u64;
        }
        3 => {
            let mut i = 0usize;
            let tar = core.regs.x[1];
            while
                *core
                    .memory()
                    .borrow()
                    .at(tar + (i as u64)) != 0
            {
                i += 1;
            }
            let tar = core.memory
                .address(tar, core.regs.flag, core.regs.kpt, core.regs.upt, ReadWrite::Read)
                .unwrap();
            let s = core
                .memory()
                .borrow()
                .slice(tar, i as u64);
            let s = String::from_utf8_lossy(s).to_string();
            core.deliver_string(s);
        }
        4 => {
            core.regs.x[0] = 1;
        }
        _ => (),
    }
    1
}

pub fn i_initext(_inst: &[u8], _core: &mut Vcore) -> u64 {
    2
}

pub fn i_destext(_inst: &[u8], _core: &mut Vcore) -> u64 {
    1
}
