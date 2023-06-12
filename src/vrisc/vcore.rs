use std::{cell::RefCell, rc::Rc};

use crate::memory::Memory;

pub struct Registers {
    pub x: [u64; 16],
    pub ip: u64,
    pub flag: u64,
    pub ivt: u64,
    pub kpt: u64,
    pub upt: u64,
    pub scp: u64,
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
            ipdump: 0,
            flagdump: 0,
        }
    }
}

pub struct InterruptController {
    intflag: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController { intflag: false }
    }

    pub fn interrupted(&self) -> bool {
        self.intflag
    }
}

pub struct Vcore {
    id: usize,
    regs: Registers,
    memory: Rc<RefCell<Memory>>,
    intctler: InterruptController,
}

impl Vcore {
    pub fn new(id: usize, memory: Rc<RefCell<Memory>>) -> Self {
        Vcore {
            id,
            regs: Registers::new(),
            memory,
            intctler: InterruptController::new(),
        }
    }

    pub fn interrupted(&self) -> bool {
        self.intctler.interrupted()
    }
}
