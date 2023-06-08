//! # csurename
//! csurename is a small command line utility which makes sure your filenames
//! adhere to @csunibo's naming standards.
//!
//! See <https://github.com/csunibo/csurename> for the full documentation.

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;

use std::io;
use std::path::*;
use std::time::Instant;

use ignore::WalkBuilder;
use inflector::Inflector;
use log::debug;
use log::error;
use log::info;
use regex::Regex;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

pub struct FixOptions {
    pub target_dir: PathBuf,
    pub recursive: bool,
    pub include_dir: bool,
    pub quiet: bool,
    pub from_stdin: bool,
}

fn fix_from_stdin(options: FixOptions, start_time: Instant) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();

    let lines = stdin.lines();

    let mut files_renamed: u64 = 0;
    for line in lines {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            break;
        } else {
            let translation = fix_name(&PathBuf::from(trimmed.trim()))?;
            println!("{translation}");
            files_renamed += 1;
        }
    }

    let running_time: f32 = start_time.elapsed().as_micros() as f32 / 1_000_000f32;
    if !options.quiet {
        println!(
            "{files_renamed} names translated in {running_time} s. See you next time!\n(^ _ ^)/"
        )
    };

    Ok(())
}

pub fn fix_names(config: FixOptions) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    // If the text flag is specified, read from stdin and translate to stdout instead of renaming files
    if config.from_stdin {
        return fix_from_stdin(config, start_time);
    }

    let mut walk_builder = WalkBuilder::new(&config.target_dir);

    walk_builder
        .max_depth(if !config.from_stdin { Some(1) } else { None })
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
                error!("Error parsing global config file: {e}");
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
                error!("Error parsing global config file: {}", e);
            }
        }
    }

    let mut files_renamed: u64 = 0;
    for entry in walk_builder.build() {
        let entry = entry?;

        let path = entry.path();

        // Skips any entry that isn't a file if the "-D" flag is not specified.
        // Always skips the target directory to prevent changing paths that the program will try to access.
        // (and because it would be quite unexpected as well)
        if !config.include_dir && !path.is_file() || path.eq(&config.target_dir) {
            continue;
        }

        let new_name = fix_name(path)?;
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
        info!("{files_renamed} files renamed in {running_time} s. See you next time!\n(^ _ ^)/")
    };

    Ok(())
}

fn fix_name(path_to_file: &Path) -> Result<String, Box<dyn Error>> {
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

pub struct CheckOptions {
    pub config_file: Option<String>,
    pub paths: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct CheckConfig {
    paths: HashMap<String, CheckPath>,
}

#[derive(Deserialize, Debug)]
struct CheckPath {
    path: String,
    pattern: String,
    ignore: Option<Vec<String>>,
    recursive: Option<bool>,
}

pub fn check_names(options: CheckOptions) -> Result<(), Box<dyn Error>> {
    let config_path = options
        .config_file
        .unwrap_or_else(|| String::from("csurename.toml"))
        .parse::<PathBuf>()?;

    debug!(
        "Config file: {config_path}",
        config_path = config_path.display()
    );
    let config_file = fs::read_to_string(config_path).map_err(|_| "Could not read config file")?;
    let config: CheckConfig = toml::from_str(&config_file)?;

    let filtered_paths = options.paths.unwrap_or_default();

    let active_paths = if filtered_paths.is_empty() {
        config.paths
    } else {
        config
            .paths
            .into_iter()
            .filter(|(name, _)| filtered_paths.contains(name))
            .collect()
    };

    for (name, config) in active_paths {
        info!("Checking {name}...",);
        info!("\tPath: {path}", path = &config.path);
        info!("\tPattern: {pattern}", pattern = &config.pattern);

        if let Some(ignores) = &config.ignore {
            info!("\tIgnoring:");
            for ignore in ignores {
                info!("\t- {ignore}");
            }
        }

        check_path(config)?;
    }

    Ok(())
}

fn check_path(config: CheckPath) -> Result<(), Box<dyn Error>> {
    let mut walk_builder = WalkBuilder::new(&config.path);

    let pattern = Regex::new(&config.pattern).map_err(|_| "Could not parse pattern")?;

    for ignore in config.ignore.unwrap_or_default() {
        walk_builder
            .add_ignore(ignore)
            .ok_or("Could not add ignored files")?;
    }

    let max_depth = if config.recursive.unwrap_or(true) {
        None
    } else {
        Some(1)
    };

    walk_builder.max_depth(max_depth).require_git(true);

    for file in walk_builder.build() {
        let file = file?;
        if !file.file_type().ok_or("Cannot get filetype")?.is_file() {
            continue;
        }

        let path = file.path();

        let file_name = path
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .ok_or("Could not convert file name to string")?;

        if pattern.is_match(file_name) {
            info!("\t\t OK {path}", path = path.display());
        } else {
            error!("\t\t KO {path}", path = path.display());
            return Err(format!(
                "File {file_name} does not match pattern {pattern}",
                file_name = file_name,
                pattern = &config.pattern
            )
            .into());
        }
    }

    Ok(())
}
