use clap::Parser;
use csurename::RunConfig;

use std::{env, path::PathBuf, process};

fn main() {
    let config = Args::parse();

    let target_dir = config
        .target_dir
        .map_or_else(env::current_dir, |p| Ok(PathBuf::from(p)))
        .expect(
            "Could not read target directory. Please make sure you have the right permissions.",
        );

    let run_config = RunConfig {
        target_dir,
        recursive: config.recursive,
        include_dir: config.include_dir,
        quiet: config.quiet,
        from_stdin: config.text,
    };

    if let Err(e) = csurename::check_names(run_config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = "Full documentation available here: https://github.com/csunibo/csurename"
)]
pub struct Args {
    /// Specifies a target directory, working dir if none
    target_dir: Option<String>,

    /// Makes renaming recursive, renaming files in subfolders as well
    #[arg(short, long)]
    recursive: bool,

    /// Renames directories as well
    #[arg(short = 'D', long = "dir")]
    include_dir: bool,

    /// Suppress output
    #[arg(short, long)]
    quiet: bool,

    /// Reads lines from stdin and translates them to the given convention in stdout until the first empty line
    #[arg(short = 'T', long)]
    text: bool,
}
