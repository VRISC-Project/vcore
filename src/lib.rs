pub mod config;
pub mod vrisc_core;

use std::sync::{Arc, RwLock};

use config::Config;
use vrisc_core::{Memory, Vcore};

pub fn run(config: Config) {
    let mut memory = Memory::new(config.memory);
    memory.load_firmware(&config.firmware_file);
    let memory = Arc::new(RwLock::new(memory));
    let mut cores = Vec::new();
    for _ in 0..config.cores {
        cores.push(Vcore::new(Arc::clone(&memory)));
    }
    //默认开启CPU0
    {
        cores[0].lock().unwrap().start();
    }
    for core in cores.iter() {
        core.lock().unwrap().join();
    }
}
