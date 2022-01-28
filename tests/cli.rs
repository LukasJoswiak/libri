use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

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
