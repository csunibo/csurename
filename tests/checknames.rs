use std::vec;

use csurename::CheckOptions;
use test_log::test;

#[test]
fn check_err() {
    let result = csurename::check_names(CheckOptions {
        config_file: Some("./tests/testdata/csurename.toml".to_string()),
        paths: Some(vec!["err".to_string()]),
    });

    assert!(result.is_err());
}

#[test]
fn check_ok() {
    let result = csurename::check_names(CheckOptions {
        config_file: Some("./tests/testdata/csurename.toml".to_string()),
        paths: Some(vec!["ok".to_string(), "root".to_string()]),
    });

    assert!(result.is_err());
}
