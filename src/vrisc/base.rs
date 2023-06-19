use std::{thread, time::Duration};

use super::vcore::{Vcore, VcoreInstruction};

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
    None,
    None,
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
    Some((i_ldi, 10)),
    Some((i_ldm, 3)),
    Some((i_stm, 3)),
    Some((i_mv, 3)),
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

///## 基本指令集的实现
/// 参数`inst`提供一段内存，这段内存是一个vrisc指令
/// 返回一个`u64`类型，表示`ip`寄存器需要增加的数，有的指令
/// 会自行修改ip寄存器，实际返回的值可能与指令长度不符，所以需要
/// 指令返回ip寄存器需要增加的长度
// TODO

pub fn i_nop(inst: &[u8], core: &mut Vcore) -> u64 {
    loop {
        thread::sleep(Duration::from_millis(1));
        if let Some(intid) = core.intctler.interrupted() {
            break;
        }
    }
    1
}
pub fn i_add(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_sub(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_inc(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_dec(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_shl(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_shr(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_rol(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_ror(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_cmp(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_and(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_or(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_not(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_xor(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_0e(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_0f(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_10(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_11(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_12(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_13(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_jc(inst: &[u8], core: &mut Vcore) -> u64 {
    10
}
pub fn i_cc(inst: &[u8], core: &mut Vcore) -> u64 {
    10
}
pub fn i_r(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_loop(inst: &[u8], core: &mut Vcore) -> u64 {
    6
}
pub fn i_ir(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_sysc(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_sysr(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_1b(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ldi(inst: &[u8], core: &mut Vcore) -> u64 {
    10
}
pub fn i_ldm(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_stm(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_mv(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_in(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_out(inst: &[u8], core: &mut Vcore) -> u64 {
    3
}
pub fn i_22(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_23(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_24(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_25(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_26(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_27(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_28(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_29(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2a(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2b(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2c(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2d(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2e(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_2f(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ei(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_di(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_ep(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_dp(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_livt(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_lkpt(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_lupt(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_lscp(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_lipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_lflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_sipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_sflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    2
}
pub fn i_cpuid(inst: &[u8], core: &mut Vcore) -> u64 {
    1
}
pub fn i_initext(_inst: &[u8], _core: &mut Vcore) -> u64 {
    0
}
pub fn i_destext(_inst: &[u8], _core: &mut Vcore) -> u64 {
    0
}
pub fn i_3f(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
