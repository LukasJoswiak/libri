use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn config() -> Result<(), Box<dyn Error>> {
    let dir = assert_fs::TempDir::new().unwrap();
    let mut dir_path = dir.path().to_path_buf();
    dir_path.push("missing/");

    let mut cmd = Command::cargo_bin("libri")?;
    cmd.arg("--config-dir").arg(dir_path).arg("list");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("assertion failed: dir.is_dir()"));

    Ok(())
}

#[test]
fn unknown_subcommand() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("unknown_subcommand");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unknown subcommand"));

    Ok(())
}

#[test]
fn list_empty_library() -> Result<(), Box<dyn Error>> {
    let dir = assert_fs::TempDir::new().unwrap();
    let config = dir.child("config.ini");
    config.write_str(format!("library = {}", dir.path().to_str().unwrap()).as_str())?;

    let mut cmd = Command::cargo_bin("libri")?;
    cmd.arg("--config-dir").arg(dir.path()).arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n").count(2));

    Ok(())
}

#[test]
fn list_unknown_argument() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("list").arg("extra_argument");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unknown argument"));

    Ok(())
}

#[test]
fn global_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("--help");
    cmd.assert().success().stdout(
        predicate::str::is_match("libri.*SUBCOMMAND")
            .unwrap()
            .count(1),
    );

    Ok(())
}

#[test]
fn config_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("config").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("libri-config"));

    Ok(())
}

#[test]
fn list_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("list").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("libri-list"));

    Ok(())
}

#[test]
fn import_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("import").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("libri-import"));

    Ok(())
}

#[test]
fn upload_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("upload").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("libri-upload"));

    Ok(())
}

#[test]
fn device_help() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("device").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("libri-device"));

    Ok(())
}
