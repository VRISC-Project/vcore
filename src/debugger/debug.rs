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
