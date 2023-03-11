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

///# vrisc寄存器
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
            if memory.is_null() {
                panic!("Failed to allocate {} byte memory.", size);
            }
        }
        Memory { memory, size }
    }

    ///加载固件文件中的程序代码
    pub fn load_firmware(&self, path: &str) {
        let mut firmware;
        if let Ok(file) = fs::File::open(&path) {
            firmware = file;
        } else {
            panic!("Failed to load firmware file {}.", path);
        }
        let mut buffer = [0u8; 32];
        while let Ok(len) = firmware.read(&mut buffer) {
            unsafe {
                self.memory.copy_from(buffer.as_ptr(), len);
            }
        }
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

///# vcore虚拟机结构
pub struct Vcore {
    pub regs: Registers,
    pub instructions: [Option<fn(&[u8], Registers) -> u64>; 256],
    pub memory: Arc<RwLock<Memory>>,
    pub thr: Option<JoinHandle<()>>,
    started: bool,
    terminated: bool,
}

impl Vcore {
    ///初始化一个虚拟机核心  
    /// 由于内存是在核心间共享的，需要外部传入
    pub fn new(memory: Arc<RwLock<Memory>>) -> Arc<Mutex<Vcore>> {
        let mut core = Vcore {
            regs: Registers::new(),
            instructions: [None; 256],
            memory: memory,
            thr: None,
            started: false,
            terminated: false,
        };
        core.instructions[..64].copy_from_slice(&base::BASE);
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
        while !self.started {
            thread::sleep(Duration::from_millis(1));
        }
        while !self.terminated {
            //
        }
        //这是最后要做的
        self.terminated = false;
    }
}
