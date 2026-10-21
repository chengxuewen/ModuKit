use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_modukit-debug-reader");
const SABOTAGE_ENV: &str = "MODUKIT_DEBUG_READER_SABOTAGE";

fn command() -> Command {
    let mut cmd = Command::new(BIN);
    cmd.env_remove(SABOTAGE_ENV);
    cmd
}

#[test]
fn debug_reader_default_prints_pass_line() {
    let output = command().output().expect("run debug reader");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "debug reader failed: stdout={stdout} stderr={stderr}"
    );
    assert!(stdout.contains("debug-reader: ledger plugins="));
    assert!(stdout.contains("debug-reader: snapshot plugins="));
    assert!(stdout.contains("debug-reader: events head="));
    assert!(stdout.contains("debug-reader: PASS"));
}

#[test]
fn debug_reader_help_prints_usage() {
    let output = command().arg("--help").output().expect("run help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "help failed: stdout={stdout}");
    assert!(stdout.contains("usage: modukit-debug-reader"));
}

#[test]
fn debug_reader_rejects_unknown_option() {
    let output = command()
        .arg("--nope")
        .output()
        .expect("run unknown option");

    assert!(!output.status.success(), "unknown option should fail");
    let code = output.status.code().expect("exit code");
    assert_eq!(code, 2);
}

#[test]
fn sabotage_hook_omits_pass_line_and_fails() {
    let output = command()
        .env(SABOTAGE_ENV, "1")
        .output()
        .expect("run sabotage");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !output.status.success(),
        "sabotage should fail: stdout={stdout}"
    );
    assert!(!stdout.contains("debug-reader: PASS"));
    assert!(stdout.contains("debug-reader: sabotage"));
}
