use clap::Parser;

/// 基于vrisc指令集的虚拟机
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// 核心数量
    #[arg(short, long, default_value_t = 1)]
    pub cores: usize,

    /// 内存大小
    #[arg(short, long)]
    pub memory: usize,

    /// 虚拟ROM文件
    #[arg(short, long)]
    pub vrom: String,

    /// 是否开启调试（在开启了debugger特性的版本中有效）
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// 是否开启外部时钟(若不开启外部时钟
    ///                 则使用周期为4ms的内部时钟)
    #[arg(short, long, default_value_t = false)]
    pub external_clock: bool,

    #[cfg(target_os = "windows")]
    /// 在windows下有效
    ///                 表示此进程是由父进程创建的core进程
    #[arg(short, long, default_value_t = false)]
    pub process_child: bool,

    #[cfg(target_os = "windows")]
    /// 在windows下有效
    ///                 在传递process_child参数的同时使用，代表核心id
    #[arg(short, long, default_value_t = 1)]
    pub id_core: usize,
}
