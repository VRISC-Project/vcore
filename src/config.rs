use clap::Parser;

/// 基于vrisc指令集的虚拟机
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// 核心数量
    #[arg(short, long, default_value_t = 1)]
    pub cores: usize,

    ///内存大小
    #[arg(short, long)]
    pub memory: usize,

    ///固件代码文件
    #[arg(short, long)]
    pub firmware_file: String,

    ///是否开启调试
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    ///是否开启外部时钟
    #[arg(short, long, default_value_t = false)]
    pub external_clock: bool,
}
