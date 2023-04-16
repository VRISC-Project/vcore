use vcore::config::Config;

fn main() {
    println!(
        "vcore
(c)Copysight Random World Studio 2023. All rights served."
    );
    let config = Config::new();
    vcore::run(config);
}
