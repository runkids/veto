use assert_cmd::Command;
use tempfile::tempdir;

fn veto() -> Command {
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn init_creates_config() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["init", "--force"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    assert!(tmp.path().join("config.toml").exists());
}

#[test]
fn init_force_overwrites() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["init", "--force"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["init", "--force"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
}
