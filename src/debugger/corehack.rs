use std::io::{Stdout, Write};

use crossterm::{
    execute,
    style::{Attribute, Print, SetAttribute},
};

use crate::{
    utils::{memory::Memory, rdxparse::RadixParse, shared::SharedPointer},
    vrisc::vcore::DebugMode,
};

use super::{
    debug::{Regs, VdbApi},
    terminal::Terminal,
};

fn arg_loss(stdout: &mut Stdout) {
    write!(*stdout, "缺少参数, 输入\"core help\"获得帮助\n").unwrap();
}

fn arg_nan(stdout: &mut Stdout, arg: &str) {
    write!(*stdout, "参数\"{}\"不是数字\n", arg).unwrap();
}

fn arg_unireg_nan(stdout: &mut Stdout, arg: &str) {
    write!(*stdout, "参数\"{}\"不是通用寄存器中的一个\n", arg).unwrap();
}

fn arg_spereg_nan(stdout: &mut Stdout, arg: &str) {
    write!(*stdout, "参数\"{}\"不是专用寄存器中的一个\n", arg).unwrap();
}

fn arg_oversize(stdout: &mut Stdout, arg: &str, max: &str) {
    write!(*stdout, "参数\"{}\"超过最大值\"{}\"\n", arg, max).unwrap();
}

fn core_not_entered(stdout: &mut Stdout) {
    write!(*stdout, "未进入任何一个核心, 输入\"core help\"获得帮助\n").unwrap();
}

fn core_noresult(stdout: &mut Stdout, debugging_core: &mut Option<usize>) {
    write!(stdout, "核心{}未响应\n", debugging_core.unwrap()).unwrap();
}

fn core_not_runnig(stdout: &mut Stdout, debugging_core: &mut Option<usize>) {
    write!(stdout, "核心{}启动\n", debugging_core.unwrap()).unwrap();
}

pub fn run(
    cmd: &mut Vec<String>,
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
    memory: &mut Memory,
) {
    if cmd.len() == 0 {
        arg_loss(stdout);
        return;
    }
    match cmd[0].as_str() {
        "debug" => {
            cmd.remove(0);
            if cmd.len() == 0 {
                arg_loss(stdout);
                return;
            }
            let core: usize = if let Ok(core) = cmd[0].rdxparse() {
                core
            } else {
                arg_nan(stdout, &cmd[0]);
                return;
            };
            if core >= debug_ports.len() {
                arg_oversize(stdout, &cmd[0], &(debug_ports.len() - 1).to_string());
                return;
            }
            *debugging_core = Some(core);
        }
        "register" => {
            cmd.remove(0);
            register(cmd, stdout, debugging_core, debug_ports);
        }
        "start" => {
            cmd.remove(0);
            start(stdout, debugging_core, debug_ports);
        }
        "mode" => {
            cmd.remove(0);
            mode(cmd, stdout, debugging_core, debug_ports);
        }
        "continue" => {
            cmd.remove(0);
            continue_instruction(stdout, debugging_core, debug_ports);
        }
        "instruction" => {
            cmd.remove(0);
            instruction(stdout, debugging_core, debug_ports, memory);
        }
        "exit" => {
            *debugging_core = None;
        }
        "help" => {
            execute!(
                stdout,
                SetAttribute(Attribute::Underlined),
                Print("Usage"),
                SetAttribute(Attribute::NoUnderline),
                Print(": core [options]\n\n"),
            )
            .unwrap();
            Terminal::newline(stdout);
            execute!(
                stdout,
                SetAttribute(Attribute::Underlined),
                Print("Options"),
                SetAttribute(Attribute::NoUnderline),
                Print(": \n"),
            )
            .unwrap();
            Terminal::newline(stdout);
            write!(
                stdout,
                "  debug <core_id>               进入某个核心，之后的一切操作都是对于此核心的\n"
            )
            .unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  register [<register> <value>] 寄存器操作, 没有参数则读取寄存器, 有参数则将value写入register中\n").unwrap();
            Terminal::newline(stdout);
            write!(
                stdout,
                "  instruction                   查看当前ip指向的指令id\n"
            )
            .unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  start                         启动当前核心\n").unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  mode <step|none>              设置debug模式, step模式下, 需要使用\"core cont\"命令使核心继续执行一条指令\n").unwrap();
            Terminal::newline(stdout);
            write!(
                stdout,
                "  continue                      在step模式下有效, 执行下一条指令\n"
            )
            .unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  exit                          退出当前核心\n").unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  help                          打印此帮助文档\n").unwrap();
        }
        _ => {
            write!(
                *stdout,
                "未知命令\"core {}\", 输入\"core help\"获得帮助\n",
                cmd[0]
            )
            .unwrap();
        }
    }
}

fn instruction(
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
    memory: &mut Memory,
) {
    if *debugging_core == None {
        core_not_entered(stdout);
        return;
    }
    let result = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::Register(None));
    let regs = if let VdbApi::Register(Some(regs)) = result {
        regs.clone()
    } else if let VdbApi::NotRunning = result {
        core_not_runnig(stdout, debugging_core);
        return;
    } else {
        core_noresult(stdout, debugging_core);
        return;
    };
    let inst = *memory.borrow().at(regs.ip);
    write!(stdout, "0x{:02x}\n", inst).unwrap();
}

fn register(
    cmd: &mut Vec<String>,
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if cmd.len() == 0 {
        register_get(stdout, debugging_core, debug_ports);
    } else {
        register_write(cmd, stdout, debugging_core, debug_ports);
    }
}

fn register_write(
    cmd: &mut Vec<String>,
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if cmd.len() < 2 {
        arg_loss(stdout);
        return;
    }
    let reg = {
        if cmd[0].starts_with("x") {
            let mut reg = cmd[0].to_string();
            reg.remove(0);
            if let Ok(reg) = reg.rdxparse() {
                Regs::X(reg)
            } else {
                arg_unireg_nan(stdout, &cmd[0]);
                return;
            }
        } else {
            match cmd[0].as_str() {
                "ip" => Regs::Ip,
                "flag" => Regs::Flag,
                "ivt" => Regs::Ivt,
                "kpt" => Regs::Kpt,
                "upt" => Regs::Upt,
                "scp" => Regs::Scp,
                "imsg" => Regs::Imsg,
                "ipdump" => Regs::IpDump,
                "flagdump" => Regs::FlagDump,
                _ => {
                    arg_spereg_nan(stdout, &cmd[0]);
                    return;
                }
            }
        }
    };
    let data: u64 = if let Ok(data) = cmd[1].rdxparse() {
        data
    } else {
        arg_nan(stdout, &cmd[1]);
        return;
    };
    let result = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::WriteRegister(reg, data));
    if let VdbApi::Ok = result {
    } else if let VdbApi::NotRunning = result {
        core_not_runnig(stdout, debugging_core);
        return;
    } else {
        core_noresult(stdout, debugging_core);
        return;
    }
}

fn register_get(
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if *debugging_core == None {
        core_not_entered(stdout);
        return;
    }
    let result = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::Register(None));
    let regs = if let VdbApi::Register(Some(regs)) = result {
        regs.clone()
    } else if let VdbApi::NotRunning = result {
        core_not_runnig(stdout, debugging_core);
        return;
    } else {
        core_noresult(stdout, debugging_core);
        return;
    };
    write!(stdout, "通用寄存器:\n").unwrap();
    Terminal::newline(stdout);
    for i in 0..regs.x.len() {
        write!(stdout, "x{:02}: {:016x}\n", i, regs.x[i]).unwrap();
        Terminal::newline(stdout);
    }
    write!(stdout, "内部寄存器:\n").unwrap();
    Terminal::newline(stdout);
    write!(stdout, "ip      : {:016x}\n", regs.ip).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "flag    : {:016x}\n", regs.flag).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "ivt     : {:016x}\n", regs.ivt).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "pkt     : {:016x}\n", regs.kpt).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "upt     : {:016x}\n", regs.upt).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "scp     : {:016x}\n", regs.scp).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "imsg    : {:016x}\n", regs.imsg).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "ipdump  : {:016x}\n", regs.ipdump).unwrap();
    Terminal::newline(stdout);
    write!(stdout, "flagdump: {:016x}\n", regs.flagdump).unwrap();
}

fn start(
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if *debugging_core == None {
        core_not_entered(stdout);
        return;
    }
    let result = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::StartCore);
    if let VdbApi::CoreStarted = result {
        write!(stdout, "核心{}已启动\n", debugging_core.unwrap()).unwrap();
    } else if let VdbApi::Ok = result {
    } else {
        core_noresult(stdout, debugging_core);
    }
}

fn mode(
    cmd: &mut Vec<String>,
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if cmd.len() == 0 {
        arg_loss(stdout);
        return;
    }
    if *debugging_core == None {
        core_not_entered(stdout);
        return;
    }
    let mode = match cmd[0].as_str() {
        "step" => DebugMode::Step,
        "none" => DebugMode::None,
        _ => {
            write!(
                *stdout,
                "参数\"{}\"不是合法的值, 输入\"core help\"获得帮助\n",
                cmd[0]
            )
            .unwrap();
            return;
        }
    };
    if let VdbApi::Ok = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::DebugMode(mode))
    {
    } else {
        core_noresult(stdout, debugging_core);
    }
}

fn continue_instruction(
    stdout: &mut Stdout,
    debugging_core: &mut Option<usize>,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) {
    if *debugging_core == None {
        core_not_entered(stdout);
        return;
    }
    let result = debug_ports[debugging_core.unwrap()]
        .at_mut(0)
        .get_result(VdbApi::Continue);
    if let VdbApi::NotRunning = result {
        core_not_runnig(stdout, debugging_core);
    } else if let VdbApi::Ok = result {
    } else if let VdbApi::None = result {
        write!(
            stdout,
            "核心{}的debug模式是none, core cont命令无效\n",
            debugging_core.unwrap()
        )
        .unwrap();
    } else {
        core_noresult(stdout, debugging_core);
    }
}
