use assert_cmd::Command;

fn veto() -> Command {
    Command::cargo_bin("veto").unwrap()
}

#[test]
fn doctor_runs_without_crash() {
    let output = veto().args(["doctor"]).output().unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(!combined.is_empty());
}
