use std::{ u8, sync::mpsc::{ self, Sender }, thread };

use crate::utils::memory::{ AddressError, Memory, ReadWrite };

use super::base;

pub type VcoreInstruction = (fn(&[u8], &mut Vcore) -> u64, u64);

/// flag寄存器的标志位
///
/// > 详见vrisc架构文档
pub enum FlagRegFlag {
    Zero = 0,
    Symbol = 1,
    Overflow = 2,
    Equal = 3,
    Higher = 4,
    Lower = 5,
    Bigger = 6,
    Smaller = 7,
    InterruptEnabled = 8,
    PagingEnabled = 9,
    Privilege = 10,

    UserSpace = 63,
}

/// 指令的条件码
///
/// > 详见vrisc架构文档
pub enum ConditionCode {
    None = 0,
    Zero = 1,
    Signed = 2,
    Overflow = 3,
    Equal = 4,
    NonEqual = 5,
    Higher = 6,
    Lower = 7,
    NonHigher = 8,
    NonLower = 9,
    Bigger = 10,
    Smaller = 11,
    NonBigger = 12,
    NonSmaller = 13,
}

impl ConditionCode {
    pub fn new(cond: u8) -> Self {
        match cond {
            0 => ConditionCode::None,
            1 => ConditionCode::Zero,
            2 => ConditionCode::Signed,
            3 => ConditionCode::Overflow,
            4 => ConditionCode::Equal,
            5 => ConditionCode::NonEqual,
            6 => ConditionCode::Higher,
            7 => ConditionCode::Lower,
            8 => ConditionCode::NonHigher,
            9 => ConditionCode::NonLower,
            10 => ConditionCode::Bigger,
            11 => ConditionCode::Smaller,
            12 => ConditionCode::NonBigger,
            13 => ConditionCode::NonSmaller,
            _ => ConditionCode::None,
        }
    }
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

/// vrisc寄存器
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Registers {
    pub x: [u64; 16],
    pub ip: u64,
    pub flag: u64,
    pub ivt: u64,
    pub kpt: u64,
    pub upt: u64,
    pub scp: u64,
    pub imsg: u64,
    pub ipdump: u64,
    pub flagdump: u64,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            x: [0; 16],
            ip: 0,
            flag: 0,
            ivt: 0,
            kpt: 0,
            upt: 0,
            scp: 0,
            imsg: 0,
            ipdump: 0,
            flagdump: 0,
        }
    }

    pub fn reset(&mut self) {
        self.x.copy_from_slice(&[0; 16]);
        self.ip = 0;
        self.flag = 0;
        self.ivt = 0;
        self.kpt = 0;
        self.upt = 0;
        self.scp = 0;
        self.imsg = 0;
        self.ipdump = 0;
        self.flagdump = 0;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InterruptId {
    NI = 0,
    InaccessibleAddress = 1,
    Device = 2,
    Clock = 3,
    InvalidInstruction = 4,
    WrongPrivilege = 5,
    InaccessibleIOPort = 6,
    PageOrTableUnreadable = 7,
    PageOrTableUnwritable = 8,
}

impl InterruptId {
    /// 只要不在InterruptId中，都会返回NI（Not a Interrupt）
    pub fn generate(id: u8) -> Self {
        match id {
            0 => InterruptId::NI,
            1 => InterruptId::InaccessibleAddress,
            2 => InterruptId::Device,
            3 => InterruptId::Clock,
            4 => InterruptId::InvalidInstruction,
            5 => InterruptId::WrongPrivilege,
            6 => InterruptId::InaccessibleIOPort,
            7 => InterruptId::PageOrTableUnreadable,
            8 => InterruptId::PageOrTableUnwritable,
            _ => InterruptId::NI,
        }
    }
}

/// 中断控制器
///
/// 负责处理中断
pub struct InterruptController {
    intflag: bool,
    /// 在intflag为`true`时使用
    intid: InterruptId,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            intflag: false,
            intid: InterruptId::NI,
        }
    }

    pub fn interrupt(&mut self, intid: InterruptId) {
        self.intflag = true;
        self.intid = intid;
    }

    pub fn interrupted(&self) -> Option<InterruptId> {
        if self.intflag { Some(self.intid) } else { None }
    }

    pub fn reset_intflag(&mut self) {
        self.intflag = false;
    }

    pub fn reset(&mut self) {
        self.intflag = false;
        self.intid = InterruptId::NI;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DebugMode {
    /// 不debug，正常执行
    None,

    /// 单步执行
    Step,
}

/// ## 惰性寻址系统
/// hot_ip时栈上储存的ip寄存器的寻址后值，只有这个值每运行一次指令改变一次。
///
/// 此ip不重新寻址，因为在一个页内，物理地址与线性地址一一对应。
///
/// 满足如下条件时，hot_ip同步至core.regs.ip：
/// * 产生转移：需要将ip（中断转移还需flag）转存至dump寄存器
/// 满足如下条件时，重新为hot_ip寻址：
/// * 产生转移：转移很可能导致ip不在此页中
/// * 遇到最小页边界：此时地址大概率不在同一页中。“大概率”指有时分页会有大页，在大页中的较小页边界两侧的内存
///         都在同一页中，但是由于最小页有16KB，遇到最小页边界的概率也不大，判断一个最小页边界是否是此页
///         的边界会消耗更多时间（这得从顶级页表开始一级一级地查才能查到）。
///
/// 在此顺便说明，core.ip_increment是自core.regs.ip被同步以来的总increment
pub struct LazyAddress {
    pub hot_ip: u64,
    pub had_run_inst: bool,
    pub crossed_page: bool,
}

impl LazyAddress {
    pub fn new() -> Self {
        Self {
            hot_ip: 0,
            had_run_inst: false,
            crossed_page: false,
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
    pub debug_mode: DebugMode,

    /// ## 终端显示管道
    termstr_pipe: Sender<String>,
}

impl Vcore {
    pub fn new(id: usize, total_core: usize, memory: Memory) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let s = { rx.recv().unwrap() };
                println!("{}", s);
            }
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
            termstr_pipe: tx,
        }
    }

    /// 将基本指令集加载到指令空间中
    pub fn init(&mut self) {
        self.instruction_space[..64].copy_from_slice(&base::BASE);
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

            let target = self.memory.borrow().slice(self.regs.ivt + 8 * (intid as u64), 8);
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
            self.intctler.interrupt(InterruptId::InvalidInstruction);
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
        if
            self.transferred ||
            self.lazyaddr.hot_ip % (16 * 1024) == 0 ||
            self.lazyaddr.crossed_page
        {
            self.lazyaddr.hot_ip = match
                self.memory.address(
                    self.regs.ip,
                    self.regs.flag,
                    self.regs.kpt,
                    self.regs.upt,
                    ReadWrite::Read
                )
            {
                Ok(address) => address,
                Err(error) =>
                    match error {
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
                    }
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
        if
            (inst_st & 0xffff_ffff_ffff_c000) == (inst_end & 0xffff_ffff_ffff_c000) ||
            inst_end == (inst_end & 0xffff_ffff_ffff_c000)
        {
            //指令未跨页
            (self.memory().borrow().slice(self.lazyaddr.hot_ip, instlen).to_vec(), false)
        } else {
            //指令跨页
            let firstl = inst_end & (0xffff_ffff_ffff_c000 - inst_st);
            let lastl = (inst_end - inst_end) & 0xffff_ffff_ffff_c000;
            inst.copy_from_slice(self.memory().borrow().slice(inst_st, firstl));
            let last_st = match
                self.memory.address(
                    self.regs.ip + firstl,
                    self.regs.flag,
                    self.regs.kpt,
                    self.regs.upt,
                    ReadWrite::Read
                )
            {
                Ok(address) => address,
                Err(error) =>
                    match error {
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
                            panic!("出现了意外情况，在读寻址时返回了不可写错误")
                        }
                        AddressError::Ineffective => {
                            self.intctler.interrupt(InterruptId::InaccessibleAddress);
                            self.regs.imsg = self.regs.ip + firstl;
                            return (Vec::new(), true);
                        }
                    }
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
