use assert_cmd::Command;
use predicates::prelude::*;

fn veto() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn check_safe_command_exits_zero() {
    veto().args(["check", "echo hello"]).assert().success();
}

#[test]
fn check_safe_command_shows_allow() {
    veto()
        .args(["check", "echo hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ALLOW").or(predicate::str::contains("allow")));
}

#[test]
fn check_critical_exits_nonzero() {
    veto().args(["check", "rm -rf /"]).assert().failure();
}

#[test]
fn check_critical_shows_level() {
    veto()
        .args(["check", "rm -rf /"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("CRITICAL").or(predicate::str::contains("critical")));
}

#[test]
fn check_high_exits_nonzero() {
    veto()
        .args(["check", "sudo rm -rf /tmp"])
        .assert()
        .failure();
}

#[test]
fn check_medium_git_push() {
    veto()
        .args(["check", "git push origin main"])
        .assert()
        .failure();
}

#[test]
fn check_explain_shows_trace() {
    veto()
        .args(["check", "--explain", "rm -rf /"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("destructive").or(predicate::str::contains("CRITICAL")));
}

#[test]
fn check_quiet_mode() {
    veto()
        .args(["--quiet", "check", "echo hello"])
        .assert()
        .success();
}

#[test]
fn check_compound_highest_risk() {
    veto()
        .args(["check", "ls -la && rm -rf /"])
        .assert()
        .failure();
}

#[test]
fn check_whitelist_cargo() {
    veto()
        .args(["check", "cargo test --release"])
        .assert()
        .success();
}

#[test]
fn check_whitelist_git_status() {
    veto().args(["check", "git status"]).assert().success();
}
