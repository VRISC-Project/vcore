pub mod config;
pub mod debugger;
pub mod utils;
pub mod vrisc;

use core::panic;
#[cfg(feature = "debugger")]
use debugger::debug::DebuggerBackend;
#[cfg(target_os = "linux")]
use nix::unistd;
use std::{fs::File, io::Read, process::exit, thread, time::Duration};
#[cfg(target_os = "windows")]
use std::{mem::size_of, ptr::null_mut};
#[cfg(target_os = "windows")]
use winapi::um::{
    errhandlingapi::GetLastError,
    processthreadsapi::STARTUPINFOW,
    processthreadsapi::{CreateProcessW, PROCESS_INFORMATION},
};

use config::Config;
#[cfg(feature = "debugger")]
use debugger::debug::{Debugger, VdbApi};
use utils::{clock::Clock, memory::Memory, shared::SharedPointer};
use vrisc::vcore::{InterruptId, Vcore};

/// # vcore从这里开始运行
///
/// 创建内存、创建vcore核心，在debugger特性打开时创建debugger。
///
/// 每个vcore核心都运行在不同进程中，通过SharedPointer共享内存实现进程间通信。
///
/// ## 在linux上
///
/// 使用fork创建vcore核心。
///
/// ## 在windows上
///
/// 由于windows上没有与fork效果类似的函数，因此在Config中添加了两个参数用于表示
/// 这是一个vcore核心进程以及传递此核心的core id。
/// 在windows平台上会首先检测此参数是否存在并跳至vcore运行，
/// 对于主进程，使用winapi创建进程用于vcore核心。
pub fn run(config: Config) {
    #[cfg(target_os = "windows")]
    if config.process_child {
        vcore(
            config.memory,
            config.id_core,
            config.cores,
            config.debug,
            config.external_clock,
        );
        exit(0);
    }
    let mut cores = Vec::new();
    let mut cores_startflg = Vec::new();
    let mut cores_inst_count = Vec::new();
    #[cfg(feature = "debugger")]
    let mut cores_debug_port = Vec::new();

    let mut memory = Memory::new(config.memory);
    {
        // 加载vrom
        let rom = match File::open(config.vrom) {
            Ok(rom) => rom,
            Err(err) => panic!("{}", err),
        };
        let rom_: Vec<_> = rom.bytes().collect();
        let mut rom = Vec::new();
        for x in rom_ {
            rom.push(x.unwrap());
        }
        memory.borrow_mut().write_slice(0, rom.as_slice());
    }

    for i in 0..config.cores {
        cores_startflg
            .push(SharedPointer::<bool>::new(format!("VcoreCore{}StartFlg", i), 1).unwrap());
        cores_inst_count
            .push(SharedPointer::<u64>::new(format!("VcoreCore{}InstCount", i), 1).unwrap());
        #[cfg(feature = "debugger")]
        cores_debug_port
            .push(SharedPointer::<VdbApi>::new(format!("VcoreCore{}DebugApi", i), 1).unwrap());
        #[cfg(feature = "debugger")]
        cores_debug_port[i].write(0, VdbApi::None);

        if i == 0 && !config.debug {
            // core0在非debug模式下直接打开
            cores_startflg[i].write(0, true);
        } else {
            cores_startflg[i].write(0, false);
        }
        #[cfg(target_os = "linux")]
        match unsafe { unistd::fork().unwrap() } {
            unistd::ForkResult::Parent { child } => cores.push(child),
            unistd::ForkResult::Child => {
                vcore(
                    config.memory,
                    i,
                    config.cores,
                    config.debug,
                    config.external_clock,
                );
                exit(0);
            }
        }
        #[cfg(target_os = "windows")]
        {
            let mut si = STARTUPINFOW {
                cb: size_of::<STARTUPINFOW>() as u32,
                lpReserved: null_mut(),
                lpDesktop: null_mut(),
                lpTitle: null_mut(),
                dwX: 0,
                dwY: 0,
                dwXSize: 0,
                dwYSize: 0,
                dwXCountChars: 0,
                dwYCountChars: 0,
                dwFillAttribute: 0,
                dwFlags: 0,
                wShowWindow: 0,
                cbReserved2: 0,
                lpReserved2: null_mut(),
                hStdInput: null_mut(),
                hStdOutput: null_mut(),
                hStdError: null_mut(),
            };
            let mut pi = PROCESS_INFORMATION {
                hProcess: null_mut(),
                hThread: null_mut(),
                dwProcessId: 0,
                dwThreadId: 0,
            };
            let mut cmd = String::new();
            let cmdp = {
                for s in std::env::args() {
                    cmd.push_str(&format!("{} ", s));
                }
                cmd.push_str("-p ");
                cmd.push_str(&format!("-i {}", i));
                let cmd = cmd.as_bytes();
                let mut res = Vec::new();
                let mut cmd = {
                    for x in cmd {
                        res.push(*x as u16);
                    }
                    res
                };
                cmd.as_mut_ptr()
            };
            println!("{}", cmd);
            if !unsafe {
                if CreateProcessW(
                    null_mut(),
                    cmdp,
                    null_mut(),
                    null_mut(),
                    false as i32,
                    0,
                    null_mut(),
                    null_mut(),
                    &mut si,
                    &mut pi,
                ) != 0
                {
                    true
                } else {
                    false
                }
            } {
                panic!("Failed to create new process. {}", unsafe {
                    GetLastError()
                });
            }
            cores.push(pi.hProcess);
        }
        #[cfg(target_os = "macos")]
        {}
    }

    #[cfg(feature = "debugger")]
    let mut debugger = if config.debug {
        Debugger::new(config.memory, &mut cores_debug_port)
    } else {
        Debugger::none(&mut cores_debug_port)
    };
    let mut running = true;
    while running {
        thread::sleep(Duration::from_millis(1));
        #[cfg(feature = "debugger")]
        if config.debug {
            running = debugger.run();
        }
    }
}

/// ## vcore核心函数
///
/// 首先绑定内存和与主进程通信的共享内存，然后等待核心被打开，最后进入核心主循环。
///
/// 由于debugger的存在，执行一条指令的过程并没有在这里完全体现出来。
fn vcore(memory_size: usize, id: usize, total_core: usize, debug: bool, external_clock: bool) {
    let mut core_startflg =
        SharedPointer::<bool>::bind(format!("VcoreCore{}StartFlg", id), 1).unwrap();
    // 指令计数，计算从a开始运行到现在此核心共运行了多少条指令
    // 用于vcore父进程统计执行速度等
    let mut core_instruction_count =
        SharedPointer::<u64>::bind(format!("VcoreCore{}InstCount", id), 1).unwrap();
    core_instruction_count.write(0, u64::MAX);
    // vcore核心
    let mut core = Vcore::new(id, total_core, Memory::bind(memory_size));
    core.init();
    #[cfg(feature = "debugger")]
    // vcore debugger后端
    let mut debugger_backend = DebuggerBackend::new(id);

    while !*core_startflg.at(0) {
        // 等待核心被允许开始
        thread::sleep(Duration::from_millis(1));

        #[cfg(feature = "debugger")]
        if debug {
            match debugger_backend.before_start(&mut core_startflg, &mut core.debug_mode) {
                Some(true) => continue,
                None => break,
                _ => (),
            };
        }
    }

    // 内部时钟 (250Hz)
    let mut clock = Clock::new(4);

    loop {
        // 执行时钟
        if !debug && !external_clock && clock.hit() {
            core.intctler.interrupt(InterruptId::Clock);
        }
        // 检测中断
        if let Some(intid) = core.intctler.interrupted() {
            core.interrupt_jump(intid);
        }
        // 指令寻址，更新hot_ip
        if core.flush_lazy_address_system(debug) {
            continue;
        }
        // debugger后端
        #[cfg(feature = "debugger")]
        if debug {
            match debugger_backend.after_start(
                core.lazyaddr.hot_ip,
                &mut core.regs,
                &mut core.debug_mode,
                &mut core.memory,
            ) {
                Some(true) => continue,
                None => break,
                _ => (),
            };
        }
        // nop指令状态下，在此停止
        if core.nopflag {
            thread::sleep(Duration::from_micros(1));
            continue;
        }
        // 更新指令计数
        let count = (*core_instruction_count.at(0)).wrapping_add(1);
        core_instruction_count.write(0, count);
        /* 取指令 */
        let opcode = *core.memory.borrow().at(core.lazyaddr.hot_ip);
        // opcode=0x3d,0x3e分别是initext和destext指令
        // 目前不予支持
        // 这两个指令依然会产生InvalidInstruction
        // TODO
        // 添加指令执行内容需在base.rs中实现，并加入到指令空间中
        if let None = core.instruction_space[opcode as usize] {
            core.intctler.interrupt(InterruptId::InvalidInstruction);
            continue;
        }
        let instlen = core.instruction_space[opcode as usize].unwrap().1;
        let (inst, cont) = core.read_instruction(instlen);
        if cont {
            continue;
        }
        /* 执行指令 */
        core.execute_instruction(opcode, inst.as_slice());
    }
}
