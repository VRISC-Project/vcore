mod vrisc_core;

use std::{
    alloc::{alloc, Layout},
    fs,
    io::Read,
    sync::{Arc, RwLock},
};

use vrisc_core::{Memory, Vcore};

pub struct Config {
    cores: usize,          //核心数量
    memory: usize,         //内存大小
    firmware_file: String, //固件代码文件
    debug: bool,           //是否开启调试
    clock: bool,           //是否开启外部时钟
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config {
            cores: 0,
            memory: 0,
            firmware_file: String::from(""),
            debug: false,
            clock: false,
        };
        let mut iterator = std::env::args().into_iter();
        let _ = iterator.next(); //第一个参数是可执行文件名，直接跳过
        while let Some(arg) = iterator.next() {
            match arg.as_str() {
                "-m" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.memory = arg.parse().expect("A number after \"-m\" is excepted.");
                }
                "-c" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.cores = arg.parse().expect("A number after \"-c\" is excepted.");
                }
                "-b" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.firmware_file = arg;
                }
                "-d" => config.debug = true,
                "-t" => config.clock = true,
                &_ => panic!("Unknown option {}", arg),
            }
        }
        config
    }
}

pub fn run(config: Config) {
    let memory = init_memory(config.memory);

    load_firmware(config.firmware_file, memory);

    let memory = Arc::new(RwLock::new(Memory(memory)));
    let mut cores = Vec::new();
    for _ in 0..config.cores {
        cores.push(Vcore::new(Arc::clone(&memory)));
    }
    //默认开启CPU0
    cores[0].lock().unwrap().start();
    for core in cores.iter() {
        core.lock().unwrap().join();
    }
}

pub fn load_firmware(firmware_file: String, mut memory: *mut u8) {
    let mut firmware;
    if let Ok(file) = fs::File::open(&firmware_file) {
        firmware = file;
    } else {
        panic!("Failed to load firmware file {}.", firmware_file);
    }
    let mut buffer = [0u8; 16];
    while let Ok(len) = firmware.read(&mut buffer) {
        unsafe {
            for i in 0..len {
                *memory = buffer[i];
                let mut addr = memory as usize;
                addr += 1;
                memory = addr as *mut u8;
            }
        }
    }
}

pub fn init_memory(size: usize) -> *mut u8 {
    let memory;
    unsafe {
        let mut v = Vec::new();
        v.resize(size, 0u8);
        memory = alloc(Layout::for_value(&v));
        if memory.is_null() {
            panic!("Failed to allocate {} byte memory.", size);
        }
    }
    memory
}
