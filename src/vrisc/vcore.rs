use std::{cell::RefCell, rc::Rc};

use crate::memory::Memory;

use super::base;

pub type VcoreInstruction = (fn(&[u8], &mut Vcore) -> u64, u64);

pub enum FlagRegFlag {
    Zero = 0,
    Equal = 1,
    Symbol = 2,
    Overflow = 3,
    Higher = 4,
    Lower = 5,
    Bigger = 6,
    Smaller = 7,
    InterruptEnabled = 8,
    PagingEnabled = 9,
    Privilege = 10,
}

pub trait BitOptions {
    fn bit_set(&mut self, flag: FlagRegFlag);
    fn bit_reset(&mut self, flag: FlagRegFlag);
    fn bit_get(&self, flag: FlagRegFlag) -> bool;
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
}

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
}

pub struct Vcore {
    id: usize,
    pub regs: Registers,
    pub memory: Rc<RefCell<Memory>>,
    pub intctler: InterruptController,
    pub ip_increment: i64,
    pub instruction_space: [Option<VcoreInstruction>; 256],
    pub transferred: bool,
}

impl Vcore {
    pub fn new(id: usize, memory: Rc<RefCell<Memory>>) -> Self {
        Vcore {
            id,
            regs: Registers::new(),
            memory,
            intctler: InterruptController::new(),
            ip_increment: 0,
            instruction_space: [None; 256],
            transferred: true,
        }
    }

    pub fn init(&mut self) {
        self.instruction_space.copy_from_slice(&base::BASE)
    }

    /*中断跳转
    当发生中断时，
    dump寄存器转存ip与flag寄存器状态，
    进入内核态
    关闭中断
    ip跳转
    中断控制器复位
    */
    pub fn interrupt_jump(&mut self) {
        if let Some(intid) = self.intctler.interrupted() {
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
        }
    }

    pub fn memory(&self) -> &Rc<RefCell<Memory>> {
        &self.memory
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
