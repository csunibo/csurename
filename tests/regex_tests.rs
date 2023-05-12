use csurename::change_naming_convention;
use serde_json::{self, Value};
use std::io::BufReader;
use std::path::Path;
use regex::Regex;
use std::fs::File;

#[test]
fn failed_commits() {
    let naming_file = File::open("config/config/naming.json").unwrap();
    let naming: Value = serde_json::from_reader(BufReader::new(naming_file)).unwrap();

    let commits_file = File::open("tests/failed_commits.json").unwrap();
    let commits: Vec<Value> = serde_json::from_reader(BufReader::new(commits_file)).unwrap();

    for commit in commits {
        if let Some(category) = commit.get("category") {
            if let Some(regex) = naming.get(category.as_str().unwrap()) {
                assert!(
                    commit
                        .get("files")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|file| {
                            Regex::new(regex.as_str().unwrap())
                                .unwrap()
                                .is_match(&change_naming_convention(Path::new(file.as_str().unwrap())).unwrap())
                        })
                );
            }
        }
    }
}

#[test]
fn remove_accents() {
    let my_text: &str = "àÀ";

    assert!(Regex::new(r"[a-zA-Z]+").unwrap().is_match(&change_naming_convention(Path::new(&my_text)).unwrap()));
}
