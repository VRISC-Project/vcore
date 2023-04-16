mod base;

use std::{
    alloc,
    alloc::Layout,
    fs,
    io::Read,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

pub type VcoreInstruction = fn(&[u8], &mut Vcore) -> u64;

///# vrisc寄存器
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

///# 内存模块
pub struct Memory {
    pub memory: *mut u8,
    size: usize,
}

impl Memory {
    ///初始化一个大小为`size`的内存
    pub fn new(size: usize) -> Memory {
        let memory;
        unsafe {
            memory = alloc::alloc(Layout::from_size_align(size, 8).unwrap());
        }
        if memory.is_null() {
            panic!("Failed to allocate {} byte memory.", size);
        }
        Memory { memory, size }
    }

    ///加载固件文件中的程序代码
    pub fn load_firmware(&mut self, path: &str) {
        let mut firmware;
        if let Ok(file) = fs::File::open(&path) {
            firmware = file;
        } else {
            panic!("Failed to load firmware file {}.", path);
        }
        let mut buffer = [0u8; 32];
        let mut offset = 0u64;
        while let Ok(len) = firmware.read(&mut buffer) {
            if len == 0 {
                break;
            }
            unsafe {
                let mut i = 0u64;
                while (i as usize) < len {
                    *(((self.memory as u64) + offset + i as u64) as *mut u8) = buffer[i as usize];
                    i += 1;
                }
            }
            offset += len as u64;
            println!("loaded section {}.", offset);
        }
        println!("Found firmware \"{}\".", path);
    }

    pub fn address(&mut self, addr: u64) -> u64 {
        todo!();
    }

    fn read_byte(&self, addr: u64) -> u8 {
        unsafe { *((self.memory as u64 + addr) as *mut u8) }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.memory, Layout::from_size_align(self.size, 8).unwrap());
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

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
        let mut i = 0usize;
        while i < space.len() {
            if let Some(inst) = space[i] {
                self.instructions[i] = Some(inst);
            }
            i += 1;
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
    memory: Arc<RwLock<Memory>>,
    pub intctller: RwLock<InterruptController>,
    thr: Option<JoinHandle<()>>,
    incr: u64,
    started: bool,
    terminated: bool,
}

impl Vcore {
    ///初始化一个虚拟机核心  
    /// 由于内存是在核心间共享的，需要外部传入
    pub fn new(memory: Arc<RwLock<Memory>>) -> Arc<Mutex<Vcore>> {
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
        let core = Arc::new(Mutex::new(core));
        let core_ret = Arc::clone(&core);
        {
            core_ret.lock().unwrap().thr = Some(thread::spawn(move || {
                core.lock().unwrap().core();//TODO 此处core函数无法运行
            }));
        }
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
            let data = { self.memory.write().unwrap().address(addr) };
            let data = { self.memory.read().unwrap().read_byte(data) };
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

    pub fn core(&mut self) {
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
            let addr = { self.memory.write().unwrap().address(self.regs.ip) };
            let opcode = { self.memory.read().unwrap().memory as u64 };
            let opcode = (opcode + addr) as *const u8;
            let opcode = unsafe { *opcode } as usize;
            if let Some(instruction) = self.instructions.instruction(opcode.try_into().unwrap()) {
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
