use std::env;
use std::error::Error;
use std::fs;
use std::path::*;

use inflector::Inflector;

pub struct Config {
    current_dir: PathBuf,
    naming_convention: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        let naming_convention = if args.len() >= 2 {
            args[1].clone()
        } else {
            "kebab-case".to_owned()
        };

        let current_dir = if args.len() >= 3 {
            PathBuf::from(args[2].clone())
        } else {
            env::current_dir()?
        };

        Ok(Config {
            current_dir,
            naming_convention,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(&config.current_dir)?;

    for entry in entries {
        let entry = entry?;

        let path = entry.path();

        if path.is_file() {
            let new_name = change_naming_convention(&path, &config.naming_convention)?;
            let new_path = config.current_dir.join(new_name);

            fs::rename(&path, &new_path)?;
        }
    }

    Ok(())
}

pub fn change_naming_convention(
    path_to_file: &Path,
    new_naming_convention: &str,
) -> Result<String, &'static str> {
    let file_stem = match path_to_file.file_stem() {
        Some(s_opt) => match s_opt.to_str() {
            Some(s) => s,
            None => return Err("couldn't convert file stem to valid Unicode"),
        },
        None => "",
    };

    let file_extension = match path_to_file.extension() {
        Some(extension_opt) => match extension_opt.to_str() {
            Some(extension) => extension,
            None => return Err("couldn't convert file extension to valid Unicode"),
        },
        None => "",
    };

    let file_stem = match new_naming_convention {
        "camelCase" => file_stem.to_camel_case(),
        "kebab-case" => file_stem.to_kebab_case(),
        "PascalCase" => file_stem.to_pascal_case(),
        "SCREAMING_SNAKE_CASE" => file_stem.to_screaming_snake_case(),
        "Sentence_case" => file_stem.to_sentence_case(),
        "snake_case" => file_stem.to_snake_case(),
        "Title_Case" => file_stem.to_title_case(),
        "Train-Case" => file_stem.to_train_case(),
        _ => return Err("naming convention not found"),
    };

    if file_stem.is_empty() {
        Ok(format!(".{}", file_extension))
    } else if file_extension.is_empty() {
        Ok(format!("{}", file_stem))
    } else {
        Ok(format!("{}.{}", file_stem, file_extension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doesnt_rename_files_with_no_breaks() {
        assert_eq!(
            Ok("file.txt".to_string()),
            change_naming_convention(Path::new("file.txt"), "snake_case")
        );
    }

    #[test]
    fn renames_files_to_snake_case() {
        assert_eq!(
            Ok("i_like_to_fly.txt".to_string()),
            change_naming_convention(Path::new("iLikeToFly.txt"), "snake_case")
        )
    }
}
