pub mod config;
pub mod debug;
pub mod utils;
pub mod vrisc;

use core::panic;
#[cfg(target_os = "linux")]
use nix::unistd;
use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, Read, Write},
    process::exit,
    rc::Rc,
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
use debug::VdbApi;
use utils::{
    clock::Clock,
    memory::{AddressError, Memory},
    shared::SharedPointer,
};
use vrisc::vcore::{DebugMode, InterruptId, Vcore};

use crate::debug::command_line;

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
    let mut cores_debug_port = Vec::new();

    let mut memory = Memory::new(config.memory);
    {
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
        cores_debug_port
            .push(SharedPointer::<VdbApi>::new(format!("VcoreCore{}DebugApi", i), 1).unwrap());
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

    let mut stdin = std::io::BufReader::new(std::io::stdin());
    let mut stdout = std::io::BufWriter::new(std::io::stdout());
    if config.debug {
        for db in cores_debug_port.as_mut_slice() {
            while *db.at(0) != VdbApi::Initialized {
                // 在release模式中，这个循环会被优化成死循环，必须在
                // 循环里做点io操作，让它不被优化成死循环。
                // 相当于手动同步共享变量
                stdout.flush().unwrap();
            }
            *db.at_mut(0) = VdbApi::None;
        }
        print!("\nType \'help\' to learn useage.\n\x1b[34mvdb >\x1b[0m ");
        stdout.flush().unwrap();
    }
    loop {
        thread::sleep(Duration::from_micros(1));
        // debug信息输入输出机制
        // 命令提示符及终端输入等操作由如下代码进行，command_line()函数只负责处理
        // 输入的命令以及对vcore虚拟机对应的状态进行修改和查询。
        if config.debug {
            if let Some(cmd) = {
                let buffer = stdin.fill_buf().unwrap();
                if buffer.len() == 0 {
                    None
                } else {
                    let mut cmd = String::new();
                    stdin.read_line(&mut cmd).unwrap();
                    Some(cmd.trim().to_string())
                }
            } {
                let result = command_line(&cmd, &mut memory, &mut cores_debug_port);
                if result == "exit" {
                    for db in cores_debug_port.as_mut_slice() {
                        db.at_mut(0).get_result(VdbApi::Exit);
                    }
                    break;
                }
                if !result.is_empty() {
                    println!("{}", result);
                }
                print!("\x1b[34mvdb >\x1b[0m ");
                stdout.flush().unwrap();
            }
        }
    }
}

fn vcore(memory_size: usize, id: usize, total_core: usize, debug: bool, external_clock: bool) {
    let mut core_startflg =
        SharedPointer::<bool>::bind(format!("VcoreCore{}StartFlg", id), 1).unwrap();
    // 指令计数，计算从a开始运行到现在此核心共运行了多少条指令
    // 用于vcore父进程统计执行速度等
    let mut core_instruction_count =
        SharedPointer::<u64>::bind(format!("VcoreCore{}InstCount", id), 1).unwrap();
    let core_debug_port =
        SharedPointer::<VdbApi>::bind(format!("VcoreCore{}DebugApi", id), 1).unwrap();

    let memory = Memory::bind(memory_size);
    let memory = Rc::new(RefCell::new(memory));
    let mut core = Vcore::new(id, total_core, Rc::clone(&memory));
    core.init();

    if debug {
        *core_debug_port.at_mut(0) = VdbApi::Initialized;
    }

    while !*core_startflg.at(0) {
        //等待核心被允许开始
        thread::sleep(Duration::from_millis(1));

        if debug {
            match *core_debug_port.at(0) {
                VdbApi::Initialized => continue,
                VdbApi::StartCore => {
                    core_startflg.write(0, true);
                    *core_debug_port.at_mut(0) = VdbApi::Ok;
                }
                VdbApi::Exit => break,
                VdbApi::DebugMode(mode) => {
                    core.debug_mode = mode;
                    *core_debug_port.at_mut(0) = VdbApi::Ok;
                }
                _ => *core_debug_port.at_mut(0) = VdbApi::NotRunning,
            }
            // *core_debug_port.at_mut(0) = VdbApi::None;
        }
    }

    /*
    hot_ip时栈上储存的ip寄存器的寻址后值，只有这个值每运行一次指令改变一次。
    此ip不重新寻址，因为在一个页内，物理地址与线性地址一一对应。
    满足如下条件时，hot_ip同步至core.regs.ip：
        产生转移：需要将ip（中断转移还需flag）转存至dump寄存器
    满足如下条件时，重新为hot_ip寻址：
        产生转移：转移很可能导致ip不在此页中
        遇到最小页边界：此时地址大概率不在同一页中。“大概率”指有时分页会有
            大页，在大页中的较小页边界两侧的内存都在同一页中，但是由于最小页
            有16KB，遇到最小页边界的概率也不大，判断一个最小页边界是否是此页S
            的边界会消耗更多时间（这得从顶级页表开始一级一级地查才能查到）。

    在此顺便说明，core.ip_increment是自core.regs.ip被同步以来的总increment
     */
    let mut hot_ip = 0;

    let mut crossed_page = false;

    // 内部时钟 (250Hz)
    let mut clock = Clock::new(4);

    core_instruction_count.write(0, u64::MAX);

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
        if (!core.transferred && hot_ip % (16 * 1024) == 0) || debug {
            core.regs.ip += core.ip_increment as u64;
            core.ip_increment = 0;
        }
        if core.transferred || hot_ip % (16 * 1024) == 0 || crossed_page {
            hot_ip = match core
                .memory
                .borrow_mut()
                .address(core.regs.ip, core.regs.flag)
            {
                Ok(address) => address,
                Err(error) => match error {
                    AddressError::OverSized(address) => {
                        core.intctler.interrupt(InterruptId::InaccessibleAddress);
                        core.regs.imsg = address;
                        continue;
                    }
                    AddressError::WrongPrivilege => {
                        core.intctler.interrupt(InterruptId::WrongPrivilege);
                        core.regs.imsg = core.regs.ip;
                        continue;
                    }
                },
            };
            core.transferred = false;
            crossed_page = false;
        }

        if debug {
            match *core_debug_port.at(0) {
                VdbApi::Exit => break,
                VdbApi::Register(None) => {
                    *core_debug_port.at_mut(0) = VdbApi::Register(Some(core.regs.clone()));
                }
                VdbApi::WriteRegister(register, value) => {
                    match register {
                        debug::Regs::X(uni) => core.regs.x[uni] = value,
                        debug::Regs::Ip => core.regs.ip = value,
                        debug::Regs::Flag => core.regs.flag = value,
                        debug::Regs::Ivt => core.regs.ivt = value,
                        debug::Regs::Kpt => core.regs.kpt = value,
                        debug::Regs::Upt => core.regs.upt = value,
                        debug::Regs::Scp => core.regs.scp = value,
                        debug::Regs::Imsg => core.regs.imsg = value,
                        debug::Regs::IpDump => core.regs.ipdump = value,
                        debug::Regs::FlagDump => core.regs.flagdump = value,
                        _ => (),
                    }
                    *core_debug_port.at_mut(0) = VdbApi::Ok;
                }
                VdbApi::StartCore => *core_debug_port.at_mut(0) = VdbApi::CoreStarted,
                VdbApi::DebugMode(mode) => {
                    core.debug_mode = mode;
                    *core_debug_port.at_mut(0) = VdbApi::Ok;
                }
                VdbApi::Instruction(None) => {
                    *core_debug_port.at_mut(0) =
                        VdbApi::Instruction(Some(*core.memory.borrow().borrow().at(hot_ip)))
                }
                _ => (),
            }
            if core.debug_mode == DebugMode::Step {
                if let VdbApi::Continue = *core_debug_port.at(0) {
                    *core_debug_port.at_mut(0) = VdbApi::Ok;
                } else {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
            }
        }

        if core.nopflag {
            thread::sleep(Duration::from_micros(1));
            continue;
        }

        // 更新指令计数
        let count = (*core_instruction_count.at(0)).wrapping_add(1);
        core_instruction_count.write(0, count);

        /* 取指令 */
        let opcode = *core.memory.borrow().borrow().at(hot_ip);
        // 这里有个例外
        // opcode=0x3d,0x3e分别是initext和destext指令
        // 目前不予支持
        // 等项目成熟之后再添加这两个指令
        // 现在这两个指令依然会产生InvalidInstruction
        // TODO
        // 添加指令执行内容需在base.rs中实现，并加入到指令空间中
        if let None = core.instruction_space[opcode as usize] {
            core.intctler.interrupt(InterruptId::InvalidInstruction);
            continue;
        }
        let instlen = core.instruction_space[opcode as usize].unwrap().1;
        // 读取指令，首先判断指令是否跨越最小页边界
        // 若指令跨越最小页边界
        // 对下一个页起始地址寻址
        // 分成前后两部分读取
        let mut inst = Vec::new();
        let inst = {
            core.regs.ip += core.ip_increment as u64; //恰好在此更新core.regs.ip，寻址失败可以在此中断
            let inst_st = hot_ip; //最后14位为0
            let inst_end = hot_ip + instlen;
            if inst_st & 0xffff_ffff_ffff_c000 == inst_end & 0xffff_ffff_ffff_c000
                || inst_end == inst_end & 0xffff_ffff_ffff_c000
            {
                //指令未跨页
                core.memory().borrow().borrow().slice(hot_ip, instlen)
            } else {
                //指令跨页
                let firstl = inst_end & 0xffff_ffff_ffff_c000 - inst_st;
                let lastl = inst_end - inst_end & 0xffff_ffff_ffff_c000;
                inst.copy_from_slice(core.memory().borrow().borrow().slice(inst_st, firstl));
                let last_st = match core
                    .memory
                    .borrow_mut()
                    .address(core.regs.ip + firstl, core.regs.flag)
                {
                    Ok(address) => address,
                    Err(error) => match error {
                        AddressError::OverSized(address) => {
                            core.intctler.interrupt(InterruptId::InaccessibleAddress);
                            core.regs.imsg = address;
                            continue;
                        }
                        AddressError::WrongPrivilege => {
                            core.intctler.interrupt(InterruptId::WrongPrivilege);
                            core.regs.imsg = core.regs.ip;
                            continue;
                        }
                    },
                };
                inst.append(
                    &mut core
                        .memory()
                        .borrow()
                        .borrow()
                        .slice_mut(last_st, lastl)
                        .to_vec(),
                );
                crossed_page = true;
                inst.as_slice()
            }
        };

        /* 执行指令 */
        let movement = core.instruction_space[opcode as usize].unwrap().0(inst, &mut core);
        core.ip_increment += movement as i64;
        hot_ip += movement;
    }
}
