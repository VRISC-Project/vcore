use std::io::{Stdout, Write};

use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};

use crate::utils::{
    rdxparse::RadixParse,
    {memory::Memory, shared::Addressable},
};

use super::terminal::Terminal;

fn arg_loss(stdout: &mut Stdout) {
    write!(*stdout, "缺少参数, 输入\"mem help\"获得帮助\n").unwrap();
}

fn arg_nan(stdout: &mut Stdout, arg: &str) {
    write!(*stdout, "参数\"{}\"不是数字\n", arg).unwrap();
}

pub fn run(cmd: &mut Vec<String>, stdout: &mut Stdout, memory: &mut Memory) {
    if cmd.len() == 0 {
        arg_loss(stdout);
        return;
    }
    match cmd[0].as_str() {
        "view" => {
            cmd.remove(0);
            view(cmd, stdout, memory);
        }
        "write" => {
            cmd.remove(0);
            write(cmd, stdout, memory);
        }
        "help" => {
            execute!(
                stdout,
                SetAttribute(Attribute::Underlined),
                Print("Usage"),
                SetAttribute(Attribute::NoUnderline),
                Print(": mem [options]\n\n"),
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
            write!(stdout, "  view <start> <length>                         内存数据视图, 一行16字节, 只输出包括参数描述的内存区间的整行\n").unwrap();
            Terminal::newline(stdout);
            write!(stdout, "  write <start> <unit_size<=8> [data0 data1 ..] 将一组数据写入内存, 每个数据的大小都是unit_size\n").unwrap();
            Terminal::newline(stdout);
            write!(
                stdout,
                "  help                                          打印此帮助文档\n"
            )
            .unwrap();
        }
        _ => {
            write!(
                *stdout,
                "未知命令\"mem {}\", 输入\"mem help\"获得帮助\n",
                cmd[0]
            )
            .unwrap();
        }
    }
}

fn write(cmd: &mut Vec<String>, stdout: &mut Stdout, memory: &mut Memory) {
    if cmd.len() < 2 {
        arg_loss(stdout);
        return;
    }
    let mut start: u64 = if let Ok(start) = cmd[0].rdxparse() {
        start
    } else {
        arg_nan(stdout, &cmd[0]);
        return;
    };
    let units: usize = if let Ok(units) = cmd[1].rdxparse() {
        units
    } else {
        arg_nan(stdout, &cmd[1]);
        return;
    };
    if units > 8 {
        write!(
            *stdout,
            "参数unit_size最大为8, 实际为{}, 输入\"mem help\"获得帮助\n",
            units
        )
        .unwrap();
        return;
    }
    cmd.remove(0);
    cmd.remove(0);
    for data in cmd {
        let data: u64 = data.rdxparse().unwrap();
        let data = [
            data as u8,
            (data >> 8) as u8,
            (data >> 16) as u8,
            (data >> 24) as u8,
            (data >> 32) as u8,
            (data >> 40) as u8,
            (data >> 48) as u8,
            (data >> 56) as u8,
        ];
        memory.borrow_mut().write_slice(start, &data[..units]);
        start += units as u64;
    }
}

fn view(cmd: &mut Vec<String>, stdout: &mut Stdout, memory: &mut Memory) {
    if cmd.len() < 2 {
        arg_loss(stdout);
        return;
    }
    let o_start: u64 = if let Ok(start) = cmd[0].rdxparse() {
        start
    } else {
        arg_nan(stdout, &cmd[0]);
        return;
    };
    let o_length: usize = if let Ok(length) = cmd[1].rdxparse() {
        length
    } else {
        arg_nan(stdout, &cmd[1]);
        return;
    };
    let length = o_length + (o_start % 16) as usize;
    let start = o_start - o_start % 16;
    let length = length + 15;
    let length = length - length % 16;
    let data = memory.borrow().slice(start, length as u64);
    for i in 0..(data.len() / 16) {
        let i = i * 16;
        // 打印地址
        execute!(stdout, SetForegroundColor(Color::DarkGreen)).unwrap();
        write!(
            stdout,
            "{:08x} {:08x}",
            (start as usize + i) >> 32,
            (start as usize + i),
        )
        .unwrap();
        execute!(stdout, ResetColor).unwrap();
        write!(stdout, " |").unwrap();
        // 打印数据的16进制形式
        execute!(stdout, SetForegroundColor(Color::DarkGrey)).unwrap();
        for j in 0..16usize {
            if j % 4 == 0 {
                write!(stdout, " ").unwrap();
            }
            if start + (i + j) as u64 >= o_start
                && (start + (i + j) as u64) < (o_start + o_length as u64)
            {
                execute!(stdout, ResetColor).unwrap();
            }
            write!(stdout, " {:02x}", data[i + j]).unwrap();
            execute!(stdout, SetForegroundColor(Color::DarkGrey)).unwrap();
        }
        execute!(stdout, ResetColor).unwrap();
        write!(stdout, "  | ").unwrap();
        // 打印数据的字符形式
        execute!(stdout, SetForegroundColor(Color::DarkGrey)).unwrap();
        for j in 0..16usize {
            if start + (i + j) as u64 >= o_start
                && (start + (i + j) as u64) < (o_start + o_length as u64)
            {
                execute!(stdout, ResetColor).unwrap();
            }
            let c = data[i + j] as char;
            if c.is_ascii_graphic() || c == ' ' {
                write!(stdout, "{}", c).unwrap();
            } else {
                write!(stdout, ".").unwrap();
            }
            execute!(stdout, SetForegroundColor(Color::DarkGrey)).unwrap();
        }
        execute!(stdout, ResetColor).unwrap();
        write!(stdout, "\n").unwrap();
        Terminal::newline(stdout);
    }
}
