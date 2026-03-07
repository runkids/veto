use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn veto() -> Command {
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn allow_list_runs() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["allow", "list"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
}

#[test]
fn allow_add_and_list() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["init", "--force"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["allow", "add", "git push*", "--global"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["allow", "list"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("git push*"));
}

#[test]
fn allow_remove() {
    let tmp = tempdir().unwrap();
    veto()
        .args(["init", "--force"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["allow", "add", "test-pattern*", "--global"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["allow", "remove", "test-pattern*", "--global"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success();
    veto()
        .args(["allow", "list"])
        .env("VETO_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-pattern*").not());
}
