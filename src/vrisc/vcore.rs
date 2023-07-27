use std::{cell::RefCell, rc::Rc};

use crate::utils::memory::Memory;

use super::base;

pub type VcoreInstruction = (fn(&[u8], &mut Vcore) -> u64, u64);

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
}

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

pub trait BitOptions {
    fn bit_set(&mut self, flag: FlagRegFlag);
    fn bit_reset(&mut self, flag: FlagRegFlag);
    fn bit_get(&self, flag: FlagRegFlag) -> bool;

    fn mark_symbol(&mut self, reg_before: u64, reg_after: u64);
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
        if *self & (1 << 63) != 0 {
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
#[derive(PartialEq, Clone, Copy)]
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

pub enum InterruptId {
    NI = 0,
    InaccessibleAddress = 1,
    Device = 2,
    Clock = 3,
    InvalidInstruction = 4,
    WrongPrivilege = 5,
    InaccessibleIOPort = 6,
}

impl Clone for InterruptId {
    fn clone(&self) -> Self {
        match self {
            Self::NI => Self::NI,
            Self::InaccessibleAddress => Self::InaccessibleAddress,
            Self::Device => Self::Device,
            Self::Clock => Self::Clock,
            Self::InvalidInstruction => Self::InvalidInstruction,
            Self::WrongPrivilege => Self::WrongPrivilege,
            Self::InaccessibleIOPort => Self::InaccessibleIOPort,
        }
    }
}
impl Copy for InterruptId {}

pub struct InterruptController {
    intflag: bool,
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
        if self.intflag {
            Some(self.intid)
        } else {
            None
        }
    }

    pub fn reset_intflag(&mut self) {
        self.intflag = false;
    }

    pub fn reset(&mut self) {
        self.intflag = false;
        self.intid = InterruptId::NI;
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum DebugMode {
    None,
    Step,
}

/// vcore核心
pub struct Vcore {
    id: usize,
    total: usize,
    pub regs: Registers,
    pub memory: Rc<RefCell<Memory>>,
    pub intctler: InterruptController,
    pub ip_increment: i64,
    pub instruction_space: [Option<VcoreInstruction>; 256],
    pub transferred: bool,
    pub nopflag: bool,
    pub debug_mode: DebugMode,
}

impl Vcore {
    pub fn new(id: usize, total_core: usize, memory: Rc<RefCell<Memory>>) -> Self {
        Vcore {
            id,
            total: total_core,
            regs: Registers::new(),
            memory,
            intctler: InterruptController::new(),
            ip_increment: 0,
            instruction_space: [None; 256],
            transferred: true,
            nopflag: false,
            debug_mode: DebugMode::None,
        }
    }

    pub fn init(&mut self) {
        self.instruction_space[..64].copy_from_slice(&base::BASE);
    }

    /*中断跳转
    当发生中断时，
    dump寄存器转存ip与flag寄存器状态，
    进入内核态
    关闭中断
    ip跳转
    中断控制器复位
    */
    pub fn interrupt_jump(&mut self, intid: InterruptId) {
        if self.regs.flag.bit_get(FlagRegFlag::InterruptEnabled) {
            self.regs.flagdump = self.regs.flag;
            self.regs.ipdump = self.regs.ip;
            self.regs.flag.bit_reset(FlagRegFlag::InterruptEnabled);
            self.regs.flag.bit_reset(FlagRegFlag::Privilege);

            let target = self
                .memory
                .borrow()
                .borrow()
                .slice(self.regs.ivt + 8 * intid as u64, 8);
            let mut addr = 0u64;
            for i in 0..8 {
                addr |= (target[i] as u64) << (i * 8);
            }
            self.regs.ip = addr;
            self.intctler.intflag = false;
            self.transferred = true;

            self.regs.ip += self.ip_increment as u64;

            self.nopflag = false;
        }
    }

    pub fn memory(&self) -> &Rc<RefCell<Memory>> {
        &self.memory
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn total_core(&self) -> usize {
        self.total
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.intctler.reset();
        self.ip_increment = 0;
        self.transferred = true;
        self.instruction_space.copy_from_slice(&[None; 256]);
        self.init();
    }

    // 特权级检查
    // 只需在特权指令中调用
    // 若为内核态返回true，若为用户态返回false
    // 自动产生中断
    pub fn privilege_test(&mut self) -> bool {
        if self.regs.flag.bit_get(FlagRegFlag::Privilege) {
            self.intctler.interrupt(InterruptId::InvalidInstruction);
            false
        } else {
            true
        }
    }
}
