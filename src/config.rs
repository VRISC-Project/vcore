pub struct Config {
    pub cores: usize,          //核心数量
    pub memory: usize,         //内存大小
    pub firmware_file: String, //固件代码文件
    pub debug: bool,           //是否开启调试
    pub clock: bool,           //是否开启外部时钟
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config {
            cores: 0,
            memory: 0,
            firmware_file: String::from(""),
            debug: false,
            clock: false,
        };
        let mut iterator = std::env::args().into_iter();
        let _ = iterator.next(); //第一个参数是可执行文件名，直接跳过
        while let Some(arg) = iterator.next() {
            match arg.as_str() {
                "-m" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.memory = arg.parse().expect("A number after \"-m\" is excepted.");
                }
                "-c" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.cores = arg.parse().expect("A number after \"-c\" is excepted.");
                }
                "-b" => {
                    let arg = if let Some(some) = iterator.next() {
                        some
                    } else {
                        break;
                    };
                    config.firmware_file = arg;
                }
                "-d" => config.debug = true,
                "-t" => config.clock = true,
                &_ => panic!("Unknown option {}", arg),
            }
        }
        config
    }
}