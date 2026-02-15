// @specre 01KHFFCX8BCDAYP8YHG0J65H0E
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

// -- Scenario: Dispatches a valid subcommand to its handler --

#[test]
fn dispatch_valid_subcommand_exits_successfully() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();
}

// -- Scenario: Exits with error when handler fails --

#[test]
fn dispatch_handler_error_prints_to_stderr_and_exits_with_code_1() {
    let tmp = TempDir::new().unwrap();

    // Running `specre status` without specre.toml triggers a handler error
    specre()
        .args(["status"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::starts_with("Error: "));
}
