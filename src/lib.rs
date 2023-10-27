pub mod config;
pub mod debugger;
pub mod utils;
pub mod vrisc;

use core::panic;
#[cfg(feature = "debugger")]
use debugger::debug::DebuggerBackend;
#[cfg(target_os = "linux")]
use nix::unistd;
use std::{
    fs::File,
    io::Read,
    process::exit,
    sync::{
        mpsc::{self, Receiver},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};
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
use vrisc::vcore::{
    dma::DirectMemoryAccess,
    intcontroller::InterruptId,
    iocontroller::{IOController, IOPortBuffer, PortRequest},
    Vcore,
};

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

    // 初始化io相关数据结构
    let mut ioreq_delivers = Vec::new();
    let mut ioreq_receivers = Vec::new();
    for _ in 0..config.cores {
        let (tx, rx) = mpsc::channel();
        ioreq_delivers.push(tx);
        ioreq_receivers.push(rx);
    }
    let io_controller = IOController::new(ioreq_delivers);
    let io_controller = Arc::new(RwLock::new(io_controller));
    let solid_io_ports = {
        let mut p = Vec::new();
        for _ in 0..config.cores {
            p.push(Vec::new());
        }
        let mut c = 0;
        for cp in p.iter_mut() {
            for i in 0..256 {
                cp.push(
                    SharedPointer::<IOPortBuffer>::new(format!("VcoreIOPort{}C{}", i, c), 1)
                        .unwrap(),
                );
            }
            c += 1;
        }
        p
    };
    let solid_io_ports = Arc::new(RwLock::new(solid_io_ports));

    // 初始化dma
    let dma_controller = DirectMemoryAccess::new();
    let dma_controller = Arc::new(RwLock::new(dma_controller));

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

    let mut ioreq_receivers = ioreq_receivers.into_iter();
    for i in 0..config.cores {
        cores_startflg
            .push(SharedPointer::<(bool, u64)>::new(format!("VcoreCore{}StartFlg", i), 1).unwrap());
        cores_inst_count
            .push(SharedPointer::<u64>::new(format!("VcoreCore{}InstCount", i), 1).unwrap());
        #[cfg(feature = "debugger")]
        cores_debug_port
            .push(SharedPointer::<VdbApi>::new(format!("VcoreCore{}DebugApi", i), 1).unwrap());
        #[cfg(feature = "debugger")]
        cores_debug_port[i].write(0, VdbApi::None);

        if i == 0 && !config.debug {
            // core0在非debug模式下直接打开
            cores_startflg[i].write(0, (true, 0));
        } else {
            cores_startflg[i].write(0, (false, 0));
        }
        #[cfg(target_os = "linux")]
        match { unsafe { unistd::fork().unwrap() } } {
            unistd::ForkResult::Parent { child } => cores.push(child),
            unistd::ForkResult::Child => {
                vcore(
                    config.memory,
                    i,
                    config.cores,
                    config.debug,
                    config.external_clock,
                    ioreq_receivers.next().unwrap(),
                );
                exit(0);
            }
        }
        #[cfg(target_os = "windows")]
        {}
        #[cfg(target_os = "macos")]
        {}
    }

    let ref_io_controller = Arc::clone(&io_controller);
    thread::spawn(move || {
        ({ ref_io_controller.write().unwrap() }).thr_dispatch_ioreq();
    });

    thread::spawn(move || {
        IOController::do_solid_ports_services(
            solid_io_ports.write().unwrap().as_mut(),
            cores_startflg,
            dma_controller,
        );
    });

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
fn vcore(
    memory_size: usize,
    id: usize,
    total_core: usize,
    debug: bool,
    external_clock: bool,
    ioreq_receiver: Receiver<PortRequest>,
) {
    let mut core_startflg =
        SharedPointer::<(bool, u64)>::bind(format!("VcoreCore{}StartFlg", id), 1).unwrap();
    // 指令计数，计算从a开始运行到现在此核心共运行了多少条指令
    // 用于vcore父进程统计执行速度等
    let mut core_instruction_count =
        SharedPointer::<u64>::bind(format!("VcoreCore{}InstCount", id), 1).unwrap();
    core_instruction_count.write(0, 0);
    // vcore核心
    let mut core = Vcore::new(id, total_core, Memory::bind(memory_size));
    core.init();
    #[cfg(feature = "debugger")]
    // vcore debugger后端
    let mut debugger_backend = DebuggerBackend::new(id);

    while !core_startflg.at(0).0 {
        // 等待核心被允许开始
        thread::sleep(Duration::from_millis(1));

        #[cfg(feature = "debugger")]
        if debug {
            match debugger_backend.before_start(&mut core_startflg, &mut core.debug_mode) {
                Some(true) => {
                    continue;
                }
                None => {
                    break;
                }
                _ => (),
            };
        }
    }

    // 内部时钟 (250Hz)
    let mut clock = Clock::new(4);

    core.regs.ip = core_startflg.at(0).1;

    loop {
        match ioreq_receiver.try_recv() {
            Ok(port) => match port {
                PortRequest::Link(port) => core.link_device(port),
                PortRequest::Interrupt(port) => {
                    core.intctler.interrupt(InterruptId::DeviceCommunication);
                    core.regs.imsg = port as u64;
                }
            },
            _ => (),
        }

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
                &mut core.intctler,
                &mut core.debug_mode,
                &mut core.memory,
            ) {
                Some(true) => {
                    continue;
                }
                None => {
                    break;
                }
                _ => (),
            };
        }
        // nop指令状态下，在此停止
        if core.nopflag {
            thread::sleep(Duration::from_micros(1));
            continue;
        }
        /* 取指令 */
        let opcode = *core.memory.borrow().at(core.lazyaddr.hot_ip);
        // opcode=0x3d,0x3e分别是initext和destext指令
        // 目前不支持
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
        // 更新指令计数
        *core_instruction_count.at_mut(0) += 1;
    }
}
