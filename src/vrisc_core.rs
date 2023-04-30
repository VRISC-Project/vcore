mod base;
pub mod memory;

use std::{
    sync::{
        mpsc::{self, Receiver},
        RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::utils::vrc::Vrc;

use self::memory::Memory;

pub type VcoreInstruction = fn(&[u8], &mut Vcore) -> u64;

///# vrisc寄存器
#[derive(Clone)]
pub struct Registers {
    pub x: [u64; 16],
    pub ip: u64,
    pub flg: u64,
    pub kpt: u64,
    pub upt: u64,
    pub ivt: u64,
    pub ipdump: u64,
    pub flgdump: u64,
}

pub enum FlagMask {
    None,
    Equal,
    Bigger,
    Smaller,
    ZeroFlag,
    SignFlag,
    OverviewFlag,
    InterruptEnable,
    PagingEnable,
    PrivilegeFlag,
    Higher,
    Lower,
}

trait BitOption {
    fn get_bit(&self, bit: usize) -> bool;
    fn set_bit(&mut self, bit: usize);
    fn reset_bit(&mut self, bit: usize);
}

impl BitOption for u64 {
    fn get_bit(&self, bit: usize) -> bool {
        self & (1 << bit) > 0
    }

    fn set_bit(&mut self, bit: usize) {
        *self &= 1 << bit;
    }

    fn reset_bit(&mut self, bit: usize) {
        *self &= !(1 << bit);
    }
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
            ipdump: 0,
            flgdump: 0,
        }
    }

    pub fn flag(&self, mask: FlagMask) -> bool {
        match mask {
            FlagMask::None => self.flg.get_bit(0),
            FlagMask::Equal => self.flg.get_bit(1),
            FlagMask::Bigger => self.flg.get_bit(2),
            FlagMask::Smaller => self.flg.get_bit(3),
            FlagMask::ZeroFlag => self.flg.get_bit(4),
            FlagMask::SignFlag => self.flg.get_bit(5),
            FlagMask::OverviewFlag => self.flg.get_bit(6),
            FlagMask::InterruptEnable => self.flg.get_bit(7),
            FlagMask::PagingEnable => self.flg.get_bit(8),
            FlagMask::PrivilegeFlag => self.flg.get_bit(9),
            FlagMask::Higher => self.flg.get_bit(10),
            FlagMask::Lower => self.flg.get_bit(11),
        }
    }
}

impl Copy for Registers {}

///# 中断控制器
pub struct InterruptController {
    triggered: bool,
    interrupt_id: u8,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            triggered: false,
            interrupt_id: 0,
        }
    }

    fn interrupt_addr(&self, regs: &Registers, int_id: u8) -> u64 {
        regs.ivt + int_id as u64 * 8
    }
}

pub struct InstructionSpace {
    instructions: [Option<VcoreInstruction>; 256],
}

impl InstructionSpace {
    pub fn new() -> Self {
        InstructionSpace {
            instructions: [None; 256],
        }
    }

    fn load_instruction_set(&mut self, space: &[Option<VcoreInstruction>]) {
        for i in 0..space.len() {
            if let Some(inst) = space[i] {
                self.instructions[i] = Some(inst);
            }
        }
    }

    fn instruction(&self, inst_id: u8) -> Option<VcoreInstruction> {
        self.instructions[inst_id as usize]
    }
}

///# vcore虚拟机结构
pub struct Vcore {
    regs: Registers,
    instructions: InstructionSpace,
    memory: Memory,
    pub intctller: RwLock<InterruptController>,
    thr: Option<JoinHandle<()>>,
    incr: u64,
    started: bool,
    terminated: bool,
}

impl Vcore {
    ///### 初始化一个虚拟机核心  
    /// 由于内存是在核心间共享的，需要外部传入
    pub fn new(memory: Memory) -> Vrc<Vcore> {
        let mut core = Vcore {
            regs: Registers::new(),
            instructions: InstructionSpace::new(),
            memory,
            intctller: RwLock::new(InterruptController::new()),
            thr: None,
            incr: 0,
            started: false,
            terminated: false,
        };
        core.instructions.load_instruction_set(&base::BASE);
        let mut core = Vrc::new(core);
        let core_ret = Vrc::clone(&core);
        let (tx, rx) = mpsc::channel();
        let thr = Some(thread::spawn(move || {
            core.lock().unwrap().core(rx);
        }))
        .unwrap();
        tx.send(thr).unwrap();
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

    fn read_instruction(&mut self, addr: u64) -> Vec<u8> {
        let mut inst = vec![];
        for _ in [0; 10].into_iter() {
            let data = { self.memory.address(addr) };
            let data = { self.memory.read_byte(data) };
            inst.push(data);
        }
        inst
    }

    fn interrupt_addr(&self, int_id: u8) -> u64 {
        self.intctller
            .read()
            .unwrap()
            .interrupt_addr(&self.regs, int_id)
    }

    pub fn core(&mut self, rx: Receiver<JoinHandle<()>>) {
        self.thr = Some(rx.recv().unwrap());
        while !self.started {
            thread::sleep(Duration::from_millis(1));
        }
        while !self.terminated {
            //检测中断
            if self.intctller.read().unwrap().triggered {
                let int_handler = self.interrupt_addr(self.intctller.read().unwrap().interrupt_id);
                self.regs.flgdump = self.regs.flg;
                self.regs.ipdump = self.regs.ip;
                self.regs.ip = int_handler;
                self.regs.flg.reset_bit(6);
                self.regs.flg.reset_bit(8);
            }
            //取指并运行
            let addr = { self.memory.address(self.regs.ip) };
            let opcode = { self.memory.memory as u64 };
            let opcode = (opcode + addr) as *const u8;
            let opcode = unsafe { *opcode } as usize;
            if let Some(instruction) = //
                self.instructions.instruction(opcode.try_into().unwrap())
            {
                let code = &self.read_instruction(addr);
                self.incr = instruction(code, self);
                self.regs.ip += self.incr;
            } else {
                self.intctller.write().unwrap().interrupt_id = 4;
                self.intctller.write().unwrap().triggered = true;
                self.incr = 0;
            }
        }
        //这是最后要做的
        self.terminated = false;
    }
}

impl Drop for Vcore {
    /// 此处drop不执行任何操作
    /// 只有特意调用vcore_drop()才能释放
    /// 方便Vrc指针的操作
    fn drop(&mut self) {}
}

impl Vcore {
    fn vcore_drop(mut self) {
        drop(self.incr);
        drop(self.intctller.write().unwrap());
        drop(self.regs);
        drop(self.started);
        drop(self.terminated);
        drop(self.thr.take());
    }
}
