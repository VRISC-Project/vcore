use std::{thread, time::Duration};

use crate::{
    memory::Memory,
    utils::shared::SharedPointer,
    vrisc::vcore::{DebugMode, Registers},
};

#[derive(PartialEq, Clone, Copy)]
pub enum Regs {
    None,
    X(usize),
    Ip,
    Flag,
    Ivt,
    Kpt,
    Upt,
    Scp,
    Imsg,
    IpDump,
    FlagDump,
}

#[derive(PartialEq, Clone, Copy)]
pub enum VdbApi {
    None,
    Initialized,
    NotRunning,
    StartCore,
    CoreStarted,
    Register(Option<Registers>),
    WriteRegister(Regs, u64),
    DebugMode(DebugMode),
    Instruction(Option<u8>),
    Continue,
    Exit,
    Ok,
}

impl VdbApi {
    pub fn get_result(&mut self, api: Self) -> &mut Self {
        *self = api;
        {
            let mut counter = 0;
            while counter < 1000 && api == *self {
                thread::sleep(Duration::from_millis(1));
                counter += 1;
            }
        }
        self
    }
}

pub fn command_line(
    cmd: &str,
    memory: &mut Memory,
    debug_ports: &mut Vec<SharedPointer<VdbApi>>,
) -> String {
    let mut cmd: Vec<&str> = cmd.split_ascii_whitespace().collect();
    if cmd.len() == 0 {
        return "".to_string();
    }
    match cmd[0] {
        "" => "".to_string(),
        "mem" => memory_hack(&mut cmd, memory),
        "core" => core_hack(&mut cmd, debug_ports),
        "exit" => "exit".to_string(),
        "help" => "vcore debugger

options:
    mem     Memory hack. Type \"mem help\" for details.
    core    Core hack. Type \"core help\" for details.
    exit    Stop vcore.
    help    Print this text."
            .to_string(),
        _ => "Undefined command. Type \"help\" for help.".to_string(),
    }
}

fn core_hack(cmd: &mut Vec<&str>, debug_ports: &mut Vec<SharedPointer<VdbApi>>) -> String {
    cmd.remove(0);
    if cmd.len() == 0 {
        return "Type \"core help\" for usage.".to_string();
    }
    match cmd[0] {
        "regs" => {
            // 打印寄存器
            let core_id: usize = cmd[1].parse().unwrap();
            if core_id >= debug_ports.len() {
                format!(
                    "There are {} cores. Using core{}.",
                    debug_ports.len(),
                    core_id
                )
            } else {
                match debug_ports[core_id]
                    .at_mut(0)
                    .get_result(VdbApi::Register(None))
                {
                    VdbApi::Register(Some(regs)) => {
                        let mut result = String::new();
                        for i in 0..16 {
                            result.push_str(&format!("x{}\t: {:#016x}\n", i, regs.x[i]));
                        }
                        result.push_str(&format!("\nip\t: {:#016x}\n", regs.ip));
                        result.push_str(&format!("flag\t: {:#016x}\n", regs.flag));
                        result.push_str(&format!("ivt\t: {:#016x}\n", regs.ivt));
                        result.push_str(&format!("kpt\t: {:#016x}\n", regs.kpt));
                        result.push_str(&format!("upt\t: {:#016x}\n", regs.upt));
                        result.push_str(&format!("scp\t: {:#016x}\n", regs.scp));
                        result.push_str(&format!("imsg\t: {:#016x}\n", regs.imsg));
                        result.push_str(&format!("ipdump\t: {:#016x}\n", regs.ipdump));
                        result.push_str(&format!("flagdump: {:#016x}", regs.flagdump));
                        result
                    }
                    VdbApi::NotRunning => format!("Core{} is not running.", core_id),
                    _ => panic!("Internal exception."),
                }
            }
        }
        "amount" => format!("{}", debug_ports.len()),
        "start" => {
            let core_id: usize = cmd[1].parse().unwrap();
            if debug_ports.len() <= core_id {
                format!(
                    "There are {} cores. Using core{}.",
                    debug_ports.len(),
                    core_id
                )
            } else {
                match debug_ports[core_id].at_mut(0).get_result(VdbApi::StartCore) {
                    VdbApi::Ok => "OK".to_string(),
                    VdbApi::CoreStarted => format!("core{} started.", core_id),
                    _ => panic!("Internal exception."),
                }
            }
        }
        "write" => {
            let core_id: usize = cmd[1].parse().unwrap();
            let register = cmd[2];
            let value: u64 = cmd[3].parse().unwrap();
            if register.starts_with("x") {
                let unireg: usize = String::from_utf8(register.as_bytes()[1..].to_vec())
                    .unwrap()
                    .parse()
                    .unwrap();
                if let VdbApi::Ok = debug_ports[core_id]
                    .at_mut(0)
                    .get_result(VdbApi::WriteRegister(Regs::X(unireg), value))
                {
                    "OK".to_string()
                } else {
                    panic!("Internal exception.");
                }
            } else {
                let wregmsg = match register {
                    "ip" => VdbApi::WriteRegister(Regs::Ip, value),
                    "flag" => VdbApi::WriteRegister(Regs::Flag, value),
                    "ivt" => VdbApi::WriteRegister(Regs::Ivt, value),
                    "kpt" => VdbApi::WriteRegister(Regs::Kpt, value),
                    "upt" => VdbApi::WriteRegister(Regs::Upt, value),
                    "scp" => VdbApi::WriteRegister(Regs::Scp, value),
                    "imsg" => VdbApi::WriteRegister(Regs::Imsg, value),
                    "ipdump" => VdbApi::WriteRegister(Regs::IpDump, value),
                    "flagdump" => VdbApi::WriteRegister(Regs::FlagDump, value),
                    _ => VdbApi::WriteRegister(Regs::None, value),
                };
                if let VdbApi::Ok = debug_ports[core_id].at_mut(0).get_result(wregmsg) {
                    "OK".to_string()
                } else {
                    panic!("Internal exception.");
                }
            }
        }
        "dbgmod" => {
            let core_id: usize = cmd[1].parse().unwrap();
            let option = cmd[2];
            let option = if option == "none" {
                DebugMode::None
            } else if option == "step" {
                DebugMode::Step
            } else {
                return format!("Unknow option {}.", option);
            };
            match debug_ports[core_id]
                .at_mut(0)
                .get_result(VdbApi::DebugMode(option))
            {
                VdbApi::Ok => String::new(),
                VdbApi::NotRunning => format!("Core{} is not running.", core_id),
                _ => panic!("Internal exception."),
            }
        }
        "cont" => {
            let core_id: usize = cmd[1].parse().unwrap();
            if let VdbApi::Ok = debug_ports[core_id].at_mut(0).get_result(VdbApi::Continue) {
                String::new()
            } else {
                panic!("Internal exception.");
            }
        }
        "inst" => {
            let core_id: usize = cmd[1].parse().unwrap();
            if let VdbApi::Instruction(Some(inst)) = debug_ports[core_id]
                .at_mut(0)
                .get_result(VdbApi::Instruction(None))
            {
                format!("{:#02x}", inst)
            } else {
                panic!("Internal exception.");
            }
        }
        "help" => "usage: core <options>

options:
    regs <core_id>                      Print all registers.
    write <core_id> <register> <value>  Write value to register.
    amount                              Print core amount.
    start <core_id>                     Start core<core_id>.
    dbgmod <core_id> [none|step]        Set debugger mode.
    cont <core_id>                      Run next instruction when in step mode.
    inst <core_id>                      Print the id of the instruction which will be executed.
    help                                Print this text."
            .to_string(),
        _ => "Undefined command. Type \"mem help\" for help.".to_string(),
    }
}

fn memory_hack(cmd: &mut Vec<&str>, memory: &mut Memory) -> String {
    cmd.remove(0);
    if cmd.len() == 0 {
        return "Type \"mem help\" for usage.".to_string();
    }
    match cmd[0] {
        "read" => {
            if cmd.len() < 3 {
                "Please type read option in correct form.".to_string()
            } else {
                let addr: u64 = cmd[1].parse().unwrap();
                let len: usize = cmd[2].parse().unwrap();
                let len = len + (addr % 16) as usize + 15;
                let addr = addr / 16;
                let mut addr = addr * 16;
                let count = len / 16;
                let mut result = String::new();
                for _ in 0..count {
                    result.push_str(&format!(
                        "{:04x} {:04x} {:04x} {:04x} | ",
                        addr >> 48,
                        addr >> 32,
                        addr >> 16,
                        addr
                    ));
                    for j in 0..16 {
                        if j % 4 == 0 {
                            result.push_str(" ");
                        }
                        let num = memory.borrow().at(addr + j);
                        result.push_str(&format!("{:02x} ", *num));
                    }
                    result.push_str(" | ");
                    let ss = memory.borrow().slice(addr, 16).to_vec();
                    for c in ss {
                        if (c as char).is_alphanumeric() || (c as char).is_ascii_punctuation() {
                            result.push(c as char);
                        } else {
                            result.push('.');
                        }
                    }
                    result.push_str("\n");
                    addr += 16;
                }
                result.pop();
                result
            }
        }
        "write" => {
            if cmd.len() < 4 {
                "Please type read option in correct form.".to_string()
            } else {
                let addr: u64 = cmd[1].parse().unwrap();
                let len: usize = cmd[2].parse().unwrap();
                let content: u64 = cmd[3].parse().unwrap();
                let sl = [
                    content as u8,
                    (content >> 8) as u8,
                    (content >> 16) as u8,
                    (content >> 24) as u8,
                    (content >> 32) as u8,
                    (content >> 40) as u8,
                    (content >> 48) as u8,
                    (content >> 56) as u8,
                ];
                memory.borrow_mut().write_slice(addr, &sl[..len]);
                "".to_string()
            }
        }
        "help" => "usage: mem <options>

options:
    read <addr> <len>               Read memory from the nearest 16-bit aligned address
                                        in after 16-bit aligned <len> length,
                                        print it on screen.
    write <addr> <len> <content>    Write <len> bytes data to <addr>, the max <len> is 8.
    help                            Print this text."
            .to_string(),
        _ => "Undefined command. Type \"mem help\" for help.".to_string(),
    }
}
