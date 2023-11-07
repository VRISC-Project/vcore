use std::io::{Stdout, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode},
    QueueableCommand,
};

use crate::utils::{memory::Memory, shared::SharedPointer};

use super::debug::{Debugger, VdbApi};

#[derive(Debug)]
/// ## 终端
///
/// 为vcore debugger提供的一个shell
///
/// 实现了如下功能：
///
/// * `backspace`删除光标前的字符
/// * `delete`删除光标后的字符
/// * 左右键移动光标
/// * `ctrl`+左右键将光标移动一个字
/// * 上下键还原历史命令（未输入的情况下）
pub struct Terminal {
    cmd: String,
    cursor: usize,

    stdout: Stdout,

    history: Vec<String>,
    cmd_index: usize,
    typed: bool,
}

impl Terminal {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        Terminal {
            cmd: String::new(),
            cursor: 0,
            stdout: std::io::stdout(),
            history: vec![String::from("")],
            cmd_index: 0,
            typed: false,
        }
    }

    pub fn none() -> Self {
        Self {
            cmd: String::new(),
            cursor: 0,
            stdout: std::io::stdout(),
            history: vec![],
            cmd_index: 0,
            typed: false,
        }
    }

    /// ## 打印命令提示符
    ///
    /// 如果有正在调试的核心，在命令提示符中体现
    pub fn prompt(&mut self, debugging_core: Option<usize>) {
        execute!(self.stdout, SetForegroundColor(Color::Blue), Print("vdb"),).unwrap();
        if let Some(core) = debugging_core {
            execute!(
                self.stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("#"),
                ResetColor,
                Print(" "),
            )
            .unwrap();
            execute!(self.stdout, SetForegroundColor(Color::Green)).unwrap();
            write!(self.stdout, "{}", core).unwrap();
        }
        execute!(
            self.stdout,
            SetForegroundColor(Color::Blue),
            Print(">"),
            ResetColor,
            Print(" ")
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }

    /// ## 新的一行
    ///
    /// 此函数只会在raw模式中将光标移到行首，
    /// 换行符需要自己输出。
    pub fn newline(stdout: &mut Stdout) {
        let _ = stdout.queue(cursor::MoveLeft(65535));
    }

    fn do_shutdown(&self, debug_ports: &mut Vec<SharedPointer<VdbApi>>) {
        for i in 0..debug_ports.len() {
            debug_ports[i].get_result(VdbApi::Exit);
        }
    }

    fn do_paste(&mut self, text: String) {
        let cmd = self.cmd.chars().collect::<Vec<char>>();
        let index = cmd.iter().position(|c| c == &'\n');
        let cmd = if let Some(val) = index {
            cmd.split_at(val).0.to_vec()
        } else {
            cmd
        };
        self.cursor += cmd.len();
        write!(self.stdout, "{}", text).unwrap();
    }

    fn do_move_left(&mut self, modifiers: KeyModifiers) {
        let cmd = self.cmd.chars().collect::<Vec<char>>();
        let _ = self.stdout.queue(cursor::MoveLeft(1));
        self.cursor -= 1;
        if modifiers == KeyModifiers::CONTROL {
            while self.cursor != 0
                && !cmd[self.cursor].is_ascii_punctuation()
                && cmd[self.cursor] != ' '
            {
                let _ = self.stdout.queue(cursor::MoveLeft(1));
                self.cursor -= 1;
            }
        }
    }

    fn do_move_right(&mut self, modifiers: KeyModifiers) {
        let cmd = self.cmd.chars().collect::<Vec<char>>();
        let _ = self.stdout.queue(cursor::MoveRight(1));
        self.cursor += 1;
        if modifiers == KeyModifiers::CONTROL {
            while self.cursor != cmd.len()
                && !cmd[self.cursor].is_ascii_punctuation()
                && cmd[self.cursor] != ' '
            {
                let _ = self.stdout.queue(cursor::MoveRight(1));
                self.cursor += 1;
            }
        }
    }

    fn do_backspace(&mut self, modifiers: KeyModifiers) {
        let mut cmd = self.cmd.chars().collect::<Vec<char>>();
        if modifiers != KeyModifiers::CONTROL {
            let _ = self.stdout.queue(cursor::MoveLeft(1));
            for i in self.cursor..cmd.len() {
                write!(self.stdout, "{}", cmd[i]).unwrap();
                if self.cursor == cmd.len() {
                    let _ = self.stdout.queue(cursor::MoveRight(1));
                }
            }
            write!(self.stdout, " ").unwrap();
            self.cursor -= 1;
            cmd.remove(self.cursor);
            let _ = self
                .stdout
                .queue(cursor::MoveLeft((cmd.len() - self.cursor + 1) as u16));
            self.cmd = cmd.iter().collect::<String>();
        } else {
            while !cmd[self.cursor - 1].is_ascii_punctuation()
                && cmd[self.cursor - 1] != ' '
                && self.cursor > 0
            {
                let _ = self.stdout.queue(cursor::MoveLeft(1));
                for i in self.cursor..cmd.len() {
                    write!(self.stdout, "{}", cmd[i]).unwrap();
                    if self.cursor == cmd.len() {
                        let _ = self.stdout.queue(cursor::MoveRight(1));
                    }
                }
                write!(self.stdout, " ").unwrap();
                self.cursor -= 1;
                cmd.remove(self.cursor);
                let _ = self
                    .stdout
                    .queue(cursor::MoveLeft((cmd.len() - self.cursor + 1) as u16));
                self.cmd = cmd.iter().collect::<String>();
            }
        }
    }

    fn do_delete(&mut self, modifiers: KeyModifiers) {
        let mut cmd = self.cmd.chars().collect::<Vec<char>>();
        if modifiers != KeyModifiers::CONTROL {
            for i in (self.cursor + 1)..cmd.len() {
                write!(self.stdout, "{}", cmd[i]).unwrap();
            }
            write!(self.stdout, " ").unwrap();
            let _ = self
                .stdout
                .queue(cursor::MoveLeft((cmd.len() - self.cursor) as u16));
            cmd.remove(self.cursor);
            self.cmd = cmd.iter().collect::<String>();
        } else {
            while self.cursor < cmd.len()
                && !cmd[self.cursor].is_ascii_punctuation()
                && cmd[self.cursor] != ' '
            {
                for i in (self.cursor + 1)..cmd.len() {
                    write!(self.stdout, "{}", cmd[i]).unwrap();
                }
                write!(self.stdout, " ").unwrap();
                let _ = self
                    .stdout
                    .queue(cursor::MoveLeft((cmd.len() - self.cursor) as u16));
                cmd.remove(self.cursor);
                self.cmd = cmd.iter().collect::<String>();
            }
        }
    }

    // 按下enter键后会执行用户输入的命令
    // 由于用户可能输入exit命令来结束程序
    // 因此需要返回一个bool值表示是否要结束
    // 返回false时代表程序结束
    fn do_enter(
        &mut self,
        debugging_core: &mut Option<usize>,
        debug_ports: &mut Vec<SharedPointer<VdbApi>>,
        memory: &mut Memory,
    ) -> bool {
        Self::newline(&mut self.stdout);
        write!(self.stdout, "\n").unwrap();
        let _ = self.stdout.queue(cursor::MoveLeft(
            self.cmd.chars().collect::<Vec<char>>().len() as u16 + 10,
        ));
        if !self.cmd.is_empty() {
            if self.history.len() == 1 || self.cmd != self.history[self.history.len() - 2] {
                let idx = self.history.len() - 1;
                self.history[idx] = self.cmd.clone();
                self.history.push(String::new());
            }
            if !Debugger::exec(
                self.cmd.clone(),
                &mut self.stdout,
                debugging_core,
                debug_ports,
                memory,
            ) {
                Self::newline(&mut self.stdout);
                return false;
            }
            self.cmd_index = self.history.len() - 1;
            self.cmd = String::new();
        }
        Self::newline(&mut self.stdout);
        self.prompt(*debugging_core);
        self.cursor = 0;
        self.typed = false;
        true
    }

    fn do_previous_history(&mut self) {
        self.cmd_index -= 1;
        if self.cmd.chars().collect::<Vec<char>>().len() - self.cursor != 0 {
            let _ = self.stdout.queue(cursor::MoveRight(
                (self.cmd.chars().collect::<Vec<char>>().len() - self.cursor - 1) as u16,
            ));
        }
        for _ in 0..self.cmd.chars().collect::<Vec<char>>().len() {
            let _ = self.stdout.queue(cursor::MoveLeft(1));
            write!(self.stdout, " ").unwrap();
            let _ = self.stdout.queue(cursor::MoveLeft(1));
        }
        self.cmd = self.history[self.cmd_index].clone();
        write!(self.stdout, "{}", self.cmd).unwrap();
        self.cursor = self.cmd.chars().collect::<Vec<char>>().len();
    }

    fn do_next_history(&mut self) {
        self.cmd_index += 1;
        if self.cmd.chars().collect::<Vec<char>>().len() - self.cursor != 0 {
            let _ = self.stdout.queue(cursor::MoveRight(
                (self.cmd.chars().collect::<Vec<char>>().len() - self.cursor - 1) as u16,
            ));
        }
        for _ in 0..self.cmd.chars().collect::<Vec<char>>().len() {
            let _ = self.stdout.queue(cursor::MoveLeft(1));
            write!(self.stdout, " ").unwrap();
            let _ = self.stdout.queue(cursor::MoveLeft(1));
        }
        self.cmd = self.history[self.cmd_index].clone();
        write!(self.stdout, "{}", self.cmd).unwrap();
        self.cursor = self.cmd.chars().collect::<Vec<char>>().len();
    }

    fn do_type_char(&mut self, c: char) {
        let mut cmd = self.cmd.chars().collect::<Vec<char>>();
        let len = cmd.len() - self.cursor;
        cmd.insert(self.cursor, c);
        for i in self.cursor..cmd.len() {
            write!(self.stdout, "{}", cmd[i]).unwrap();
            if self.cursor == cmd.len() - 1 {
                let _ = self.stdout.queue(cursor::MoveRight(1));
            }
        }
        let _ = self.stdout.queue(cursor::MoveLeft(len as u16));
        self.cmd = cmd.iter().collect::<String>();
        self.cursor += 1;
        self.typed = true;
    }

    pub fn run(
        &mut self,
        debugging_core: &mut Option<usize>,
        debug_ports: &mut Vec<SharedPointer<VdbApi>>,
        memory: &mut Memory,
    ) -> bool {
        match event::read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                self.do_shutdown(debug_ports);
                return false;
            }
            Event::Paste(text) => {
                self.do_paste(text);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers,
                ..
            }) => {
                if self.cursor == 0 {
                    return true;
                }
                self.do_move_left(modifiers);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers,
                ..
            }) => {
                let cmd = self.cmd.chars().collect::<Vec<char>>();
                if self.cursor == cmd.len() {
                    return true;
                }
                self.do_move_right(modifiers);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers,
                ..
            }) => {
                if self.cursor == 0 {
                    self.typed = false;
                    return true;
                }
                self.do_backspace(modifiers);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers,
                ..
            }) => {
                let cmd = self.cmd.chars().collect::<Vec<char>>();
                if self.cursor == cmd.len() {
                    return true;
                }
                self.do_delete(modifiers);
            }
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                if modifiers == KeyModifiers::ALT || modifiers == KeyModifiers::META {
                    return true;
                }
                match code {
                    KeyCode::Enter => {
                        if !self.do_enter(debugging_core, debug_ports, memory) {
                            self.do_shutdown(debug_ports);
                            return false;
                        }
                    }
                    KeyCode::Up => {
                        if self.history.len() == 1 {
                            return true;
                        }
                        if !self.typed {
                            if self.cmd_index == 0 {
                                return true;
                            }
                            self.do_previous_history();
                        }
                    }
                    KeyCode::Down => {
                        if self.history.len() == 1 {
                            return true;
                        }
                        if !self.typed {
                            if self.cmd_index == self.history.len() - 1 {
                                return true;
                            }
                            self.do_next_history();
                        }
                    }
                    KeyCode::Tab => {}
                    // KeyCode::F(n) => todo!(),
                    KeyCode::Char(c) => {
                        if modifiers != KeyModifiers::CONTROL {
                            self.do_type_char(c);
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        self.stdout.flush().unwrap();
        true
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        println!();
    }
}
