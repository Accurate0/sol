use assert_cmd::cargo::CommandCargoExt;
use insta::assert_snapshot;
use rstest::rstest;
use std::{env::current_dir, path::PathBuf, process::Command};

#[rstest]
fn run_success(#[files("tests/files/success/*.sol")] path: PathBuf) {
    let relative_path = pathdiff::diff_paths(&path, current_dir().unwrap()).unwrap();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let cmd = cmd
        .arg("run")
        .arg(&relative_path)
        .env("NO_COLOR", "true")
        .env("SOL_TEST", "true")
        .env("SOL_LOG", "info");

    let output = cmd.output().unwrap();

    let snapshot_name = format!("success__{}", path.file_name().unwrap().to_string_lossy());

    let output = format!(
        "{}\n\n{}",
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );

    assert_snapshot!(snapshot_name, output);
}

#[rstest]
fn run_fail(#[files("tests/files/fail/*.sol")] path: PathBuf) {
    let relative_path = pathdiff::diff_paths(&path, current_dir().unwrap()).unwrap();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let cmd = cmd
        .arg("run")
        .arg(&relative_path)
        .env("NO_COLOR", "true")
        .env("SOL_TEST", "true")
        .env("SOL_LOG", "info");

    let output = cmd.output().unwrap();

    let snapshot_name = format!("fail__{}", path.file_name().unwrap().to_string_lossy());

    let output = format!(
        "{}\n\n{}",
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );

    assert_snapshot!(snapshot_name, output);
}
