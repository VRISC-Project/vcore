use std::{thread, time::Duration};

use super::vcore::Vcore;

type VcoreInstruction = fn(&[u8], &mut Vcore) -> u64;

pub const BASE: [Option<VcoreInstruction>; 64] = [
    Some(i_nop),
    Some(i_add),
    Some(i_sub),
    Some(i_inc),
    Some(i_dec),
    Some(i_shl),
    Some(i_shr),
    Some(i_rol),
    Some(i_ror),
    Some(i_cmp),
    Some(i_and),
    Some(i_or),
    Some(i_not),
    Some(i_xor),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(i_jc),
    Some(i_cc),
    Some(i_r),
    Some(i_loop),
    Some(i_ir),
    Some(i_sysc),
    Some(i_sysr),
    None,
    Some(i_ldi),
    Some(i_ldm),
    Some(i_stm),
    Some(i_mv),
    Some(i_in),
    Some(i_out),
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
    Some(i_ei),
    Some(i_di),
    Some(i_ep),
    Some(i_dp),
    Some(i_livt),
    Some(i_lkpt),
    Some(i_lupt),
    Some(i_lscp),
    Some(i_lipdump),
    Some(i_lflagdump),
    Some(i_sipdump),
    Some(i_sflagdump),
    Some(i_cpuid),
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
    println!("nop");
    loop {
        thread::sleep(Duration::from_millis(1));
        if core.interrupted() {
            break;
        }
    }
    1
}
pub fn i_add(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_sub(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_inc(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_dec(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_shl(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_shr(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_rol(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ror(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_cmp(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_and(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_or(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_not(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_xor(inst: &[u8], core: &mut Vcore) -> u64 {
    0
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
    0
}
pub fn i_cc(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_r(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_loop(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ir(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_sysc(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_sysr(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_1b(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ldi(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ldm(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_stm(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_mv(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_in(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_out(inst: &[u8], core: &mut Vcore) -> u64 {
    0
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
    0
}
pub fn i_di(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_ep(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_dp(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_livt(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_lkpt(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_lupt(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_lscp(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_lipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_lflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_sipdump(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_sflagdump(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_cpuid(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_initext(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_destext(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
pub fn i_3f(inst: &[u8], core: &mut Vcore) -> u64 {
    0
}
