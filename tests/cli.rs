use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;
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

// #[test]
// fn list_empty_library() -> Result<(), Box<dyn Error>> {
//     let mut cmd = Command::cargo_bin("libri")?;
//     // TODO: Set the library path
//
//     cmd.arg("list");
//     cmd.assert()
//         .success()
//         .stdout(predicate::str::contains("\n").count(2));
//
//     Ok(())
// }

#[test]
fn list_unknown_argument() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("libri")?;

    cmd.arg("list").arg("extra_argument");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unknown argument"));

    Ok(())
}

// TODO: Test all help variants
// libri --help list // prints global help
// libri list --help // prints list help
