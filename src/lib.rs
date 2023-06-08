//! # csurename
//! csurename is a small command line utility which makes sure your filenames
//! adhere to @csunibo's naming standards.
//!
//! See <https://github.com/csunibo/csurename> for the full documentation.

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::*;
use std::time::Instant;

use clap::{arg, command, value_parser};
use ignore::WalkBuilder;
use inflector::Inflector;
use unicode_normalization::UnicodeNormalization;

pub struct Config {
    target_dir: PathBuf,
    recursive: bool,
    include_dir: bool,
    quiet: bool,
    text: bool,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        let matches = command!()
            .arg(arg!([TARGET] "Specifies a target directory, working dir if none").value_parser(value_parser!(PathBuf)))
            .arg(arg!(-r --recursive "Makes renaming recursive, renaming files in subfolders as well"))
            .arg(arg!(-D --dir "Renames directories as well"))
            .arg(arg!(-T --text "Reads lines from stdin and translates them to the given convention in stdout until the first empty line"))
            .arg(arg!(-q --quiet "Suppress output"))
            .after_help("Full documentation available here: https://github.com/csunibo/csurename")
            .get_matches();

        let target_dir = matches
            .get_one::<PathBuf>("TARGET")
            .map_or(env::current_dir(), |p| Ok(p.to_path_buf()))?;

        let recursive = matches.get_flag("recursive");
        let include_dir = matches.get_flag("dir");
        let quiet = matches.get_flag("quiet");
        let text = matches.get_flag("text");

        Ok(Config {
            target_dir,
            recursive,
            include_dir,
            quiet,
            text,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    let mut files_renamed: u64 = 0;

    // If the text flag is specified, read from stdin and translate to stdout instead of renaming files
    if config.text {
        let stdin = io::stdin();

        loop {
            let mut input = String::new();

            let len = stdin.read_line(&mut input)?;

            if len == 0 || input.trim().is_empty() {
                let running_time: f32 = start_time.elapsed().as_micros() as f32 / 1_000_000f32;

                if !config.quiet {
                    println!(
                        "{files_renamed} names translated in {running_time} s. See you next time!\n(^ _ ^)/"
                    )
                };

                return Ok(());
            } else {
                let translation = change_naming_convention(&PathBuf::from(input.trim()))?;
                println!("{translation}");
                files_renamed += 1;
            }
        }
    }

    let mut walk_builder = WalkBuilder::new(&config.target_dir);

    walk_builder
        .max_depth(if !config.recursive { Some(1) } else { None })
        .require_git(true);

    // Parse different locations for a global config file depending on OS family
    // On unix systems (MacOS or Linux), searches for ~/.config/csurename/ignore
    // On windows systems searches for %USERPROFILE%\AppData\Local\csurename\ignore
    // Outputs errors to stderr if there's one but doesn't stop program execution
    #[cfg(unix)]
    if let Some(home_path) = env::var_os("HOME") {
        let config_location = format!("{}/.config/csurename/ignore", home_path.to_string_lossy());
        if PathBuf::from(&config_location).is_file() {
            if let Some(e) = walk_builder.add_ignore(Path::new(&config_location)) {
                eprintln!("Error parsing global config file: {e}");
            }
        }
    }
    #[cfg(windows)]
    if let Some(user_profile) = env::var_os("USERPROFILE") {
        let config_location = format!(
            "{}\\AppData\\Local\\csurename\\ignore",
            user_profile.to_string_lossy()
        );
        if PathBuf::from(&config_location).is_file() {
            if let Some(e) = walk_builder.add_ignore(Path::new(&config_location)) {
                eprintln!("Error parsing global config file: {}", e);
            }
        }
    }

    for entry in walk_builder.build() {
        let entry = entry?;

        let path = entry.path();

        // Skips any entry that isn't a file if the "-D" flag is not specified.
        // Always skips the target directory to prevent changing paths that the program will try to access.
        // (and because it would be quite unexpected as well)
        if !config.include_dir && !path.is_file() || path.eq(&config.target_dir) {
            continue;
        }

        let new_name = change_naming_convention(path)?;
        let new_path = path
            .parent()
            .ok_or("can't find path parent")?
            .join(new_name);
        if path != new_path {
            fs::rename(path, &new_path)?;
            files_renamed += 1;
        }
    }
    let running_time: f32 = start_time.elapsed().as_micros() as f32 / 1_000_000f32;

    if !config.quiet {
        println!("{files_renamed} files renamed in {running_time} s. See you next time!\n(^ _ ^)/")
    };

    Ok(())
}

pub fn change_naming_convention(path_to_file: &Path) -> Result<String, Box<dyn Error>> {
    let file_stem = path_to_file
        .file_stem()
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .ok_or_else(|| {
            format!("couldn't convert file stem of {path_to_file:?} to valid Unicode")
        })?;

    let file_extension = path_to_file
        .extension()
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .ok_or_else(|| {
            format!("couldn't convert file extension of {path_to_file:?} to valid Unicode")
        })?;

    let file_stem = file_stem
        .nfd()
        .filter(char::is_ascii)
        .collect::<String>()
        .to_kebab_case();

    if file_stem.is_empty() {
        Ok(format!(".{file_extension}"))
    } else if file_extension.is_empty() {
        Ok(file_stem)
    } else {
        Ok(format!("{file_stem}.{file_extension}"))
    }
}
