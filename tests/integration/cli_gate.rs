use assert_cmd::Command;

fn veto() -> Command {
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn gate_safe_command_allows() {
    veto().args(["gate", "echo hello"]).assert().success();
}

#[test]
fn gate_claude_stdin_safe() {
    veto()
        .args(["gate", "--claude"])
        .write_stdin(r#"{"tool_input":{"command":"ls -la"}}"#)
        .assert()
        .success();
}
