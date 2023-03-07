mod inst;

use std::{
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Registers {
    pub x: [u64; 16],
    pub ip: u64,
    pub flg: u64,
    pub kpt: u64,
    pub upt: u64,
    pub ivt: u64,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            x: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ip: 0,
            flg: 0,
            kpt: 0,
            upt: 0,
            ivt: 0,
        }
    }
}

pub struct Memory(pub *mut u8);
unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

pub struct Vcore {
    pub regs: Registers,
    pub instructions: [Option<fn(&[u8], Registers) -> u64>; 256],
    pub memory: Arc<RwLock<Memory>>,
    pub thr: Option<JoinHandle<()>>,
    started:bool,
    terminated: bool,
}

impl Vcore {
    pub fn new(memory: Arc<RwLock<Memory>>) -> Arc<Mutex<Vcore>> {
        let mut core = Vcore {
            regs: Registers::new(),
            instructions: [None; 256],
            memory: memory,
            thr: None,
            started:false,
            terminated: false,
        };
        core.instructions[..64].copy_from_slice(&inst::BASE);
        let core = Arc::new(Mutex::new(core));
        let core_ret = Arc::clone(&core);
        core_ret.lock().unwrap().thr = Some(thread::spawn(move || {
            core.lock().unwrap().core();
        }));
        core_ret
    }

    pub fn join(&self) {
        while !self.terminated {
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn start(&mut self) {
        self.started = true;
    }

    pub fn core(&mut self) {
        //这是最后要做的
        self.terminated = false;
    }
}
