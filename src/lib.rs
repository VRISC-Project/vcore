pub mod config;
pub mod memory;
pub mod utils;
pub mod vrisc;

use config::Config;
use memory::Memory;
use nix::unistd::{self};
use vrisc::vcore::Vcore;

pub fn run(config: Config) {
    let mut cores = Vec::new();

    for i in 0..config.cores {
        match unsafe { unistd::fork().unwrap() } {
            unistd::ForkResult::Parent { child } => cores.push(child),
            unistd::ForkResult::Child => {
                vcore(config.memory, i);
                break;
            }
        }
    }
}

fn vcore(memory_size: usize, id: usize) {
    let memory = Memory::new(memory_size);
    let core = Vcore::new(id);
}
