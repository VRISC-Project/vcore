use vcore_rust::Config;

fn main() {
    println!(
        "vcore
(c)Copysight Random World Studio 2023. All rights served."
    );
    let config = Config::new();
    vcore_rust::run(config);
}
