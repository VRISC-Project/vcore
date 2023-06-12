pub mod config;
pub mod memory;
pub mod utils;
pub mod vrisc;

use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use config::Config;
use memory::Memory;
use nix::unistd;
use utils::shared::SharedPointer;
use vrisc::vcore::Vcore;

pub fn run(config: Config) {
    let mut cores = Vec::new();
    let mut cores_startflg = Vec::new();

    for i in 0..config.cores {
        cores_startflg
            .push(SharedPointer::<bool>::new(format!("VcoreCore{}StartFlg", i), 1).unwrap());
        if i == 0 {
            //core0直接打开
            cores_startflg[0].write(0, true);
        }
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
    let core_startflg = SharedPointer::<bool>::new(format!("VcoreCore{}StartFlg", id), 1).unwrap();

    let memory = Memory::new(memory_size);
    let memory = Rc::new(RefCell::new(memory));
    let core = Vcore::new(id, Rc::clone(&memory));

    while !core_startflg.at(0) {
        //等待核心被允许开始
        thread::sleep(Duration::from_millis(1));
    }

    loop { //主循环
    }
}
