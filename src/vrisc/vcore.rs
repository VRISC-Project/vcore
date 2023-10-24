pub mod addresser;
pub mod dma;
pub mod intcontroller;
pub mod iocontroller;
pub mod regs_flags;

use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread, u8,
};

use crate::utils::{
    memory::{AddressError, Memory, ReadWrite},
    shared::SharedPointer,
};

use self::{
    addresser::LazyAddress,
    intcontroller::{InterruptController, InterruptId},
    iocontroller::{IOPortBuffer, PortRequest},
    regs_flags::{ConditionCode, FlagRegFlag, Registers},
};

use super::base;

pub type VcoreInstruction = (fn(&[u8], &mut Vcore) -> u64, u64);

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DebugMode {
    /// 不debug，正常执行
    None,

    /// 单步执行
    Step,
}

/// ## 位操作
pub trait BitOptions {
    fn bit_set(&mut self, flag: FlagRegFlag);
    fn bit_reset(&mut self, flag: FlagRegFlag);
    fn bit_get(&self, flag: FlagRegFlag) -> bool;

    /// 在指令中使用，根据执行前后寄存器的不同对flag寄存器的标志位进行设置
    ///
    /// 此函数只应对Vcore::regs.flag使用
    fn mark_symbol(&mut self, reg_before: u64, reg_after: u64);

    /// 查看对应标志位是否被置位
    fn satisfies_condition(&self, cond: ConditionCode) -> bool;
}

impl BitOptions for u64 {
    fn bit_set(&mut self, flag: FlagRegFlag) {
        *self |= 1 << (flag as u64);
    }

    fn bit_reset(&mut self, flag: FlagRegFlag) {
        *self &= !(1 << (flag as u64));
    }

    fn bit_get(&self, flag: FlagRegFlag) -> bool {
        (*self & (1 << (flag as u64))) != 0
    }

    fn mark_symbol(&mut self, reg_before: u64, reg_after: u64) {
        *self &= 0xffff_ffff_ffff_fff8;
        if reg_after == 0 {
            self.bit_set(FlagRegFlag::Zero);
        }
        if (*self & (1 << 63)) != 0 {
            self.bit_set(FlagRegFlag::Symbol);
        }
        if reg_after < reg_before {
            self.bit_set(FlagRegFlag::Overflow);
        }
    }

    fn satisfies_condition(&self, cond: ConditionCode) -> bool {
        match cond {
            ConditionCode::None => true,
            ConditionCode::Zero => self.bit_get(FlagRegFlag::Zero),
            ConditionCode::Signed => self.bit_get(FlagRegFlag::Symbol),
            ConditionCode::Overflow => self.bit_get(FlagRegFlag::Overflow),
            ConditionCode::Equal => self.bit_get(FlagRegFlag::Equal),
            ConditionCode::NonEqual => !self.bit_get(FlagRegFlag::Equal),
            ConditionCode::Higher => self.bit_get(FlagRegFlag::Higher),
            ConditionCode::Lower => self.bit_get(FlagRegFlag::Lower),
            ConditionCode::NonHigher => !self.bit_get(FlagRegFlag::Higher),
            ConditionCode::NonLower => !self.bit_get(FlagRegFlag::Lower),
            ConditionCode::Bigger => self.bit_get(FlagRegFlag::Bigger),
            ConditionCode::Smaller => self.bit_get(FlagRegFlag::Smaller),
            ConditionCode::NonBigger => !self.bit_get(FlagRegFlag::Bigger),
            ConditionCode::NonSmaller => !self.bit_get(FlagRegFlag::Smaller),
        }
    }
}

/// vcore核心
pub struct Vcore {
    id: usize,
    /// 代表此vcore中共有多少个核心
    total: usize,
    pub regs: Registers,
    pub memory: Memory,
    pub intctler: InterruptController,
    pub lazyaddr: LazyAddress,

    /// ## ip增量
    ///
    /// 每次执行指令后，ip会跳转至下一个应该执行的指令处，
    /// 如果直接更新ip寄存器，每次都要重新寻址。
    ///
    /// 我们使用如下机制来减缓寻址的频率并保证运行时数据正确：
    ///
    /// * 在vcore核心函数中，使用一个hot_ip记录当前ip对应的物理地址
    /// * 指令执行后，不会立即更新ip寄存器，会更新hot_ip，只要没有越过页框边界那物理地址一定是正确的。
    ///     此成员就是用来记录ip跳转情况的。
    /// * 只有越过页框边界或执行转移指令才更新ip寄存器
    /// * 执行转移指令前后都更新ip寄存器，使得中断信息以及中断时的ip及flag转储值正确
    pub ip_increment: i64,

    /// ## 指令空间
    ///
    /// 最初只会加载基本指令集，其它指令集需要在运行时用扩展指令加载。
    pub instruction_space: [Option<VcoreInstruction>; 256],

    /// ## 转移标志
    ///
    /// 在执行转移指令前后，此标志都会置为true，
    /// 然后vcore核心函数会更新ip寄存器
    pub transferred: bool,

    /// ## nop标志
    ///
    /// 由于在nop过程中也要使debugger能够使用，在vcore核心函数的主循环中判断nopflag
    /// 以阻止取指令和执行指令的过程，达到nop指令的效果。
    ///
    /// > `nop`指令详见vrisc结构文档
    pub nopflag: bool,

    /// ## 调试模式
    ///
    /// 在debugger开启时有效，
    pub debug_mode: DebugMode,

    pub io_ports: HashMap<u16, SharedPointer<IOPortBuffer>>,

    /// ## 终端显示管道
    termstr_pipe: Sender<String>,
}

impl Vcore {
    pub fn new(id: usize, total_core: usize, memory: Memory) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let s = { rx.recv().unwrap() };
            println!("{}", s);
        });
        Vcore {
            id,
            total: total_core,
            regs: Registers::new(),
            memory,
            intctler: InterruptController::new(),
            lazyaddr: LazyAddress::new(),
            ip_increment: 0,
            instruction_space: [None; 256],
            transferred: true,
            nopflag: false,
            debug_mode: DebugMode::None,
            io_ports: HashMap::new(),
            termstr_pipe: tx,
        }
    }

    pub fn add_port(&mut self, port: u16, solid: bool, core: usize) {
        self.io_ports.insert(
            port,
            SharedPointer::bind(
                if solid {
                    format!("VcoreIOPort{}C{}", port, core)
                } else {
                    format!("VcoreIOPort{}", port)
                },
                1,
            )
            .unwrap(),
        );
    }

    /// 初始化指令集和io
    pub fn init(&mut self) {
        self.instruction_space[..64].copy_from_slice(&base::BASE);
        for i in 0..256 {
            self.add_port(i, true, self.id());
        }
    }

    #[inline]
    pub fn link_device(&mut self, port: u16) {
        self.add_port(port, false, self.id());
        self.intctler.interrupt(InterruptId::Device);
        self.io_ports
            .get_mut(&0)
            .unwrap()
            .at_mut(0)
            .device_push(port as u64);
    }

    #[inline]
    pub fn execute_instruction(&mut self, opcode: u8, inst: &[u8]) {
        let movement = (self.instruction_space[opcode as usize].unwrap().0)(inst, self);
        self.ip_increment += movement as i64;
        self.lazyaddr.hot_ip += movement;
        self.lazyaddr.had_run_inst = true;
    }

    #[inline]
    /// ## 中断跳转
    ///
    /// 当发生中断时，dump寄存器转存ip与flag寄存器状态，进入内核态，
    /// 关闭中断，ip跳转，中断控制器复位
    pub fn interrupt_jump(&mut self, intid: InterruptId) {
        if self.regs.flag.bit_get(FlagRegFlag::InterruptEnabled) {
            self.regs.flagdump = self.regs.flag;
            self.regs.ipdump = self.regs.ip;
            self.regs.flag.bit_reset(FlagRegFlag::InterruptEnabled);
            self.regs.flag.bit_reset(FlagRegFlag::Privilege);

            let target = self
                .memory
                .borrow()
                .slice(self.regs.ivt + 8 * (intid as u64), 8);
            let mut addr = 0u64;
            for i in 0..8 {
                addr |= (target[i] as u64) << (i * 8);
            }
            self.regs.ip = addr;
            self.intctler.reset_intflag();
            self.transferred = true;

            self.regs.ip += self.ip_increment as u64;

            self.nopflag = false;
        }
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn total_core(&self) -> usize {
        self.total
    }

    /// ## 复位vcore核心
    pub fn reset(&mut self) {
        self.regs.reset();
        self.intctler.reset();
        self.ip_increment = 0;
        self.transferred = true;
        self.instruction_space.copy_from_slice(&[None; 256]);
        self.init();
    }

    /// ## 特权级检查
    ///
    /// 只需在特权指令中调用
    ///
    /// 若权限不符，产生中断
    pub fn privilege_test(&mut self) -> bool {
        if self.regs.flag.bit_get(FlagRegFlag::Privilege) {
            self.intctler.interrupt(InterruptId::WrongPrivilege);
            false
        } else {
            true
        }
    }

    #[inline]
    /// ## 刷新惰性寻址系统
    ///
    /// ### 返回值
    ///
    /// 如果产生了中断需要停止接下来的步骤，从头开始
    ///
    /// 用返回true表示这种情况
    pub fn flush_lazy_address_system(&mut self, debug: bool) -> bool {
        if (!self.transferred && self.lazyaddr.hot_ip % (16 * 1024) == 0) || debug {
            if self.lazyaddr.had_run_inst {
                self.regs.ip += self.ip_increment as u64;
                self.ip_increment = 0;
                self.lazyaddr.had_run_inst = false;
            }
        }
        if self.transferred || self.lazyaddr.hot_ip % (16 * 1024) == 0 || self.lazyaddr.crossed_page
        {
            self.lazyaddr.hot_ip = match self.memory.address(
                self.regs.ip,
                self.regs.flag,
                self.regs.kpt,
                self.regs.upt,
                ReadWrite::Read,
            ) {
                Ok(address) => address,
                Err(error) => match error {
                    AddressError::OverSized(address) => {
                        self.intctler.interrupt(InterruptId::InaccessibleAddress);
                        self.regs.imsg = address;
                        return true;
                    }
                    AddressError::WrongPrivilege => {
                        self.intctler.interrupt(InterruptId::WrongPrivilege);
                        self.regs.imsg = self.regs.ip;
                        return true;
                    }
                    AddressError::Unreadable => {
                        self.intctler.interrupt(InterruptId::PageOrTableUnreadable);
                        self.regs.imsg = self.regs.imsg;
                        return true;
                    }
                    AddressError::Unwritable => {
                        panic!("出现了意外情况，在读寻址时返回了不可写错误")
                    }
                    AddressError::Ineffective => {
                        self.intctler.interrupt(InterruptId::InaccessibleAddress);
                        self.regs.imsg = self.regs.ip;
                        return true;
                    }
                },
            };
            self.transferred = false;
            self.lazyaddr.crossed_page = false;
        }
        false
    }

    #[inline]
    /// ## 读取指令
    /// 先判断指令是否跨越最小页边界
    ///
    /// 若指令跨越最小页边界，对下一个页起始地址寻址，分成前后两部分读取
    ///
    /// ### 返回值
    ///
    /// 如果产生了中断需要停止接下来的步骤，从头开始
    ///
    /// 用返回true表示这种情况
    pub fn read_instruction(&mut self, instlen: u64) -> (Vec<u8>, bool) {
        let mut inst = Vec::new();
        self.regs.ip += self.ip_increment as u64; //恰好在此更新self.regs.ip，寻址失败可以在此中断
        self.ip_increment = 0;
        let inst_st = self.lazyaddr.hot_ip; //最后14位为0
        let inst_end = self.lazyaddr.hot_ip + instlen;
        if (inst_st & 0xffff_ffff_ffff_c000) == (inst_end & 0xffff_ffff_ffff_c000)
            || inst_end == (inst_end & 0xffff_ffff_ffff_c000)
        {
            //指令未跨页
            (
                self.memory()
                    .borrow()
                    .slice(self.lazyaddr.hot_ip, instlen)
                    .to_vec(),
                false,
            )
        } else {
            //指令跨页
            let firstl = inst_end & (0xffff_ffff_ffff_c000 - inst_st);
            let lastl = (inst_end - inst_end) & 0xffff_ffff_ffff_c000;
            inst.copy_from_slice(self.memory().borrow().slice(inst_st, firstl));
            let last_st = match self.memory.address(
                self.regs.ip + firstl,
                self.regs.flag,
                self.regs.kpt,
                self.regs.upt,
                ReadWrite::Read,
            ) {
                Ok(address) => address,
                Err(error) => match error {
                    AddressError::OverSized(address) => {
                        self.intctler.interrupt(InterruptId::InaccessibleAddress);
                        self.regs.imsg = address;
                        return (Vec::new(), true);
                    }
                    AddressError::WrongPrivilege => {
                        self.intctler.interrupt(InterruptId::WrongPrivilege);
                        self.regs.imsg = self.regs.ip;
                        return (Vec::new(), true);
                    }
                    AddressError::Unreadable => {
                        self.intctler.interrupt(InterruptId::PageOrTableUnreadable);
                        self.regs.imsg = self.regs.imsg;
                        return (Vec::new(), true);
                    }
                    AddressError::Unwritable => {
                        panic!("出现了意外情况，在读寻址时发生了不可写错误")
                    }
                    AddressError::Ineffective => {
                        self.intctler.interrupt(InterruptId::InaccessibleAddress);
                        self.regs.imsg = self.regs.ip + firstl;
                        return (Vec::new(), true);
                    }
                },
            };
            inst.append(&mut self.memory().borrow().slice_mut(last_st, lastl).to_vec());
            self.lazyaddr.crossed_page = true;
            (inst, false)
        }
    }

    pub fn deliver_string(&mut self, text: String) {
        self.termstr_pipe.send(text).unwrap();
    }
}
