use assert_cmd::Command;
use tempfile::tempdir;

fn veto() -> Command {
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn log_empty_config_dir() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["log"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
}

#[test]
fn log_clear() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["log", "--clear"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
}
