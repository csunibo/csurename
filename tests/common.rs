use std::process::Command;
use std::path::Path;
use tempfile::{TempDir, tempdir};
use assert_cmd::cargo::CommandCargoExt;
use inflector::cases::kebabcase::is_kebab_case;

pub fn csurename_raw_command() -> Command {
    let mut cmd = Command::cargo_bin("csurename").unwrap();
    cmd.current_dir("tests/");
    cmd
}

pub fn csurename() -> assert_cmd::Command {
    assert_cmd::Command::from_std(csurename_raw_command())
}

pub fn setup(n: u32) -> std::io::Result<TempDir> {
    let tmp_dir = create_dir()?;

    add_files(&tmp_dir, n)?;

    Ok(tmp_dir)
}

pub fn create_dir() -> std::io::Result<TempDir> {
    let tmp_dir = tempdir()?;
    Ok(tmp_dir)
}

pub fn add_files(tmp_dir: &TempDir, n: u32) -> std::io::Result<()> {
    let current_index = std::fs::read_dir(tmp_dir.path())?.count() as u32;

    for i in 0..n {
        let file_name = format!("mY_wRong File-{}", i + current_index);
        let file_path = tmp_dir.path().join(file_name);
        std::fs::File::create(&file_path)?;
    }
    
    Ok(())
}

pub fn check_dir<P: AsRef<Path>>(dir_path: P, recursive: bool) -> bool {
    let dir_entries = std::fs::read_dir(dir_path).unwrap();

    for entry in dir_entries {
        let file_path = entry.unwrap().path();

        if recursive && file_path.is_dir() {
            if !check_dir(&file_path, recursive) {
                return false;
            }
        } else {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            if !is_kebab_case(&file_name) {
                return false;
            }
        }
    }

    true
}
