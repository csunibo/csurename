mod common;
use common::*;

use predicates::prelude::*;

#[test]
fn fails_with_missing_path() {
    csurename()
        .arg("molise")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
fn fails_with_empty_path() {
    csurename()
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("a value is required"));
}

#[test]
fn fails_with_wrong_number_of_targets() {
    csurename()
        .arg("pythonista")
        .arg("banana")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn fails_with_duplicate_parameter_names() {
    csurename()
        .arg("--recursive")
        .arg("--recursive")
        .arg("BroDoesTooMuchCTRLV")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used multiple times"));
}

#[test]
fn fails_for_unknown_argument() {
    csurename()
        .arg("--whaaaat")
        .arg("BroDidntReadTheDocs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn rename_0_files(){
    let tmp_dir = setup(0).unwrap();

    csurename()
        .arg(tmp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 files renamed"));

    assert!(check_dir(tmp_dir.path(), false));
}

#[test]
fn rename_1024_files(){
    let tmp_dir = setup(1024).unwrap();

    csurename()
        .arg(tmp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1024 files renamed"));

    assert!(check_dir(tmp_dir.path(), false));
}

#[test]
fn rename_half_files(){
    let tmp_dir =setup(2).unwrap();

    csurename()
        .arg(tmp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2 files renamed"));

    add_files(&tmp_dir, 2).unwrap();

    assert!(!check_dir(tmp_dir.path(), false));
    assert_eq!(std::fs::read_dir(tmp_dir.path()).unwrap().count(), 4);

    csurename()
        .arg(tmp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2 files renamed"));

    assert!(check_dir(tmp_dir.path(), false));
}
