use clap::Parser;
use csurename::{CheckOptions, FixOptions};

use std::{env, path::PathBuf, process};

fn main() {
    let config = Args::parse();

    let target_dir = config
        .target_dir
        .map_or_else(env::current_dir, |p| Ok(PathBuf::from(p)))
        .expect(
            "Could not read target directory. Please make sure you have the right permissions.",
        );

    if let Err(e) = match config.cmd {
        Commands::Check { config } => csurename::check_names(CheckOptions {
            config_file: config,
            paths: None,
        }),
        Commands::Fix {} => csurename::fix_names(FixOptions {
            target_dir,
            recursive: config.recursive,
            include_dir: config.include_dir,
            quiet: config.quiet,
            from_stdin: config.text,
        }),
    } {
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
    #[command(subcommand)]
    cmd: Commands,

    /// Specify a target directory, working dir if none
    target_dir: Option<String>,

    /// Recursively check / translate files in subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Include directories in the renaming process
    #[arg(short = 'D', long = "dir")]
    include_dir: bool,

    /// Read lines from stdin and check / translate them to stdout
    #[arg(short = 'T', long)]
    text: bool,

    /// Suppress output
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Parser, Debug)]
enum Commands {
    Check {
        #[arg(short, long)]
        config: Option<String>,
    },
    Fix {},
}
