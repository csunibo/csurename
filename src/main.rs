use clap::Parser;
use csurename::Config;
use std::process;

fn main() {
    let config = Config::parse();

    if let Err(e) = csurename::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
