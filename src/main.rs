use clap::Parser;
use vcore::config::Config;

fn main() {
    let config = Config::parse();
    println!(
        "vcore
(c)Copysight Random World Studio 2023. All rights served."
    );
    vcore::run(config);
}
