pub mod config;
pub mod memory;
pub mod utils;

use config::Config;
use memory::Memory;
use nix::unistd;

pub fn run(config: Config) {
    let mut cores = Vec::new();

    for _ in 0..config.cores {
        match unsafe { unistd::fork().unwrap() } {
            unistd::ForkResult::Parent { child } => cores.push(child),
            unistd::ForkResult::Child => {
                vcore(config.memory);
                break;
            }
        }
    }
}

fn vcore(memory_size: usize) {
    let memory = Memory::new(memory_size);
}
