pub mod config;
pub mod vrisc_core;

use config::Config;
use vrisc_core::{memory::Memory, Vcore};

pub fn run(config: Config) {
    let mut memory = Memory::new(config.memory);
    memory.load_firmware(&config.firmware_file);
    let mut cores = Vec::new();
    for _ in 0..config.cores {
        cores.push(Vcore::new(memory.clone()));
    }
    //默认开启CPU0
    {
        cores[0].write().unwrap().start();
    }
    for core in cores.iter() {
        core.read().unwrap().join();
    }
}
