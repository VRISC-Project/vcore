use std::io::{Stdout, Write};
use std::{thread, time::Duration};

use crate::vrisc::vcore::{InterruptController, InterruptId};
use crate::{
    utils::{memory::Memory, shared::SharedPointer},
    vrisc::vcore::{DebugMode, Registers},
};

use crossterm::execute;
use crossterm::style::{Attribute, Print, SetAttribute};

use super::terminal::Terminal;
use super::{corehack, memhack};

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
    Interrupt(u8),
    Continue,
    Exit,
    /// 一般情况下用这个变体回复
    Ok,
}

unsafe impl Sync for VdbApi {}

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

    pub fn none(cores_debug_port: &'a mut Vec<SharedPointer<VdbApi>>) -> Self {
        Debugger {
            memory: Memory::bind(1),
            debug_ports: cores_debug_port,
            debugging_core: None,
            terminal: Terminal::none(),
        }
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
        let mut cmd = Self::disass(cmd);
        match cmd[0].as_str() {
            "mem" => {
                cmd.remove(0);
                memhack::run(&mut cmd, stdout, memory);
            }
            "core" => {
                cmd.remove(0);
                corehack::run(&mut cmd, stdout, debugging_core, debug_ports, memory);
            }
            "exit" => {
                return false;
            }
            "help" => {
                execute!(
                    stdout,
                    SetAttribute(Attribute::Underlined),
                    Print("Usage"),
                    SetAttribute(Attribute::NoUnderline),
                    Print(": <command> [options]\n\n")
                )
                .unwrap();
                Terminal::newline(stdout);
                execute!(
                    stdout,
                    SetAttribute(Attribute::Underlined),
                    Print("Command"),
                    SetAttribute(Attribute::NoUnderline),
                    Print(": \n")
                )
                .unwrap();
                Terminal::newline(stdout);
                write!(
                    stdout,
                    "  mem           调试内存, 输入\"mem help\"获得帮助\n"
                )
                .unwrap();
                Terminal::newline(stdout);
                write!(
                    stdout,
                    "  core          调试vcore核心, 输入\"core help\"获得帮助\n"
                )
                .unwrap();
                Terminal::newline(stdout);
                write!(stdout, "  help          打印此帮助文档\n").unwrap();
                Terminal::newline(stdout);
                write!(stdout, "  exit(Ctrl-D)  关闭vcore并退出vcore debugger\n").unwrap();
            }
            _ => {
                write!(stdout, "未知命令\"{}\", 输入\"help\"获得帮助\n", cmd[0]).unwrap();
            }
        }
        true
    }
}

pub struct DebuggerBackend {
    pub core_debug_port: Box<SharedPointer<VdbApi>>,
}

impl DebuggerBackend {
    pub fn new(id: usize) -> Self {
        let mut res = Self {
            core_debug_port: Box::new(
                SharedPointer::<VdbApi>::bind(format!("VcoreCore{}DebugApi", id), 1).unwrap(),
            ),
        };
        res.core_debug_port.write(0, VdbApi::Initialized);
        res
    }

    #[inline]
    /// ## 调试器后端
    ///
    /// 在核心开启前使用
    ///
    /// ### 返回值
    ///
    /// None - 退出
    /// Some(true) - 返回后要continue
    /// Some(false) - ()
    pub fn before_start(
        &mut self,
        core_startflg: &mut SharedPointer<(bool, u64)>,
        debug_mode: &mut DebugMode,
    ) -> Option<bool> {
        match *self.core_debug_port.at(0) {
            VdbApi::Initialized => Some(true),
            VdbApi::StartCore => {
                core_startflg.write(0, (true, 0));
                self.core_debug_port.write(0, VdbApi::Ok);
                Some(false)
            }
            VdbApi::Exit => None,
            VdbApi::DebugMode(mode) => {
                *debug_mode = mode;
                self.core_debug_port.write(0, VdbApi::Ok);
                Some(false)
            }
            _ => {
                self.core_debug_port.write(0, VdbApi::NotRunning);
                Some(false)
            }
        }
    }

    #[inline]
    /// ## 调试器后端
    ///
    /// 在核心开启后使用
    ///
    /// ### 返回值
    ///
    /// None - 退出
    /// Some(true) - 返回后要continue
    /// Some(false) - ()
    pub fn after_start(
        &mut self,
        hot_ip: u64,
        regs: &mut Registers,
        intctl: &mut InterruptController,
        debug_mode: &mut DebugMode,
        memory: &mut Memory,
    ) -> Option<bool> {
        match *self.core_debug_port.at(0) {
            VdbApi::Exit => {
                return None;
            }
            VdbApi::Register(None) => {
                self.core_debug_port
                    .write(0, VdbApi::Register(Some(regs.clone())));
            }
            VdbApi::WriteRegister(register, value) => {
                match register {
                    Regs::X(uni) => {
                        regs.x[uni] = value;
                    }
                    Regs::Ip => {
                        regs.ip = value;
                    }
                    Regs::Flag => {
                        regs.flag = value;
                    }
                    Regs::Ivt => {
                        regs.ivt = value;
                    }
                    Regs::Kpt => {
                        regs.kpt = value;
                    }
                    Regs::Upt => {
                        regs.upt = value;
                    }
                    Regs::Scp => {
                        regs.scp = value;
                    }
                    Regs::Imsg => {
                        regs.imsg = value;
                    }
                    Regs::IpDump => {
                        regs.ipdump = value;
                    }
                    Regs::FlagDump => {
                        regs.flagdump = value;
                    }
                    _ => (),
                }
                self.core_debug_port.write(0, VdbApi::Ok);
            }
            VdbApi::StartCore => {
                self.core_debug_port.write(0, VdbApi::CoreStarted);
            }
            VdbApi::DebugMode(mode) => {
                *debug_mode = mode;
                self.core_debug_port.write(0, VdbApi::Ok);
            }
            VdbApi::Instruction(None) => {
                self.core_debug_port
                    .write(0, VdbApi::Instruction(Some(*memory.borrow().at(hot_ip))));
            }
            VdbApi::Continue => {
                if *debug_mode == DebugMode::None {
                    self.core_debug_port.write(0, VdbApi::None);
                }
            }
            VdbApi::Interrupt(id) => {
                intctl.interrupt(InterruptId::generate(id));
                self.core_debug_port.write(0, VdbApi::Ok);
            }
            _ => (),
        }
        if *debug_mode == DebugMode::Step {
            if let VdbApi::Continue = *self.core_debug_port.at(0) {
                self.core_debug_port.write(0, VdbApi::Ok);
                Some(false)
            } else {
                thread::sleep(Duration::from_millis(1));
                Some(true)
            }
        } else {
            Some(false)
        }
    }
}
