use std::io::{Stdout, Write};
use std::{thread, time::Duration};

use crate::{
    utils::{memory::Memory, shared::SharedPointer},
    vrisc::vcore::{DebugMode, Registers},
};

use crossterm::execute;
use crossterm::style::{Attribute, Print, SetAttribute};

use super::terminal::Terminal;

#[derive(PartialEq, Clone, Copy, Debug)]
/// ## 寄存器名
/// 
/// 用于debug中传递需要的寄存器
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum VdbApi {
    None,
    Initialized,
    NotRunning,
    StartCore,
    /// 用于回应某些调试核心的请求
    CoreStarted,
    /// 发送时内部包含None，回复时内部包含Some(regs)
    Register(Option<Registers>),
    WriteRegister(Regs, u64),
    DebugMode(DebugMode),
    /// 获得当前正在执行的指令，发送时内部包含None，回复时内部包含Some(inst)
    Instruction(Option<u8>),
    Continue,
    Exit,
    /// 一般情况下用这个变体回复
    Ok,
}

impl VdbApi {
    /// ## 发送并获得vcore核心的回应
    /// 
    /// 此函数阻塞至vcore核心回复。
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

#[derive(Debug)]
pub struct Debugger<'a> {
    memory: Memory,
    debug_ports: &'a mut Vec<SharedPointer<VdbApi>>,
    debugging_core: Option<usize>,
    terminal: Terminal,
}

impl<'a> Debugger<'a> {
    pub fn new(memory: usize, cores_debug_port: &'a mut Vec<SharedPointer<VdbApi>>) -> Self {
        println!("VCore Debugger.");
        let mut dbgger = Debugger {
            memory: Memory::bind(memory),
            debug_ports: cores_debug_port,
            debugging_core: None,
            terminal: Terminal::new(),
        };
        dbgger.terminal.prompt(dbgger.debugging_core);
        dbgger
    }

    pub fn run(&mut self) -> bool {
        self.terminal
            .run(&mut self.debugging_core, self.debug_ports, &mut self.memory)
    }

    /// ## 整理命令
    /// 
    /// 将命令整理成String数组
    fn disass(cmd: String) -> Vec<String> {
        let cmd = cmd.trim();
        cmd.split(" ")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn exec(
        cmd: String,
        stdout: &mut Stdout,
        debugging_core: &mut Option<usize>,
        debug_ports: &mut Vec<SharedPointer<VdbApi>>,
        memory: &mut Memory,
    ) -> bool {
        let cmd = Self::disass(cmd);
        match cmd[0].as_str() {
            "exit" => return false,
            "help" => {
                execute!(
                    stdout,
                    SetAttribute(Attribute::Underlined),
                    Print("Usage"),
                    SetAttribute(Attribute::NoUnderline),
                    Print(": <command> [options]\n\n"),
                )
                .unwrap();
                Terminal::newline(stdout);
                execute!(
                    stdout,
                    SetAttribute(Attribute::Underlined),
                    Print("Command"),
                    SetAttribute(Attribute::NoUnderline),
                    Print(": \n"),
                )
                .unwrap();
                Terminal::newline(stdout);
                write!(stdout, "  help          打印此帮助文本\n").unwrap();
                Terminal::newline(stdout);
                write!(
                    stdout,
                    "  exit          关闭vcore并退出vcore debugger (使用Ctrl-D组合键同理)\n"
                )
                .unwrap();
            }
            _ => {
                write!(
                    stdout,
                    "Unknown command {}, type \"help\" for more information.\n",
                    cmd[0]
                )
                .unwrap();
            }
        }
        true
    }
}
