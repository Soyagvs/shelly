use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);
fn run(script: &str, args: &[&str], secret: &str) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!(
        "shelly-secret-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.shelly");
    fs::write(&path, script).unwrap();
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(args)
        .arg(&path)
        .env("SHELLY_SENTINEL", secret)
        .output()
        .unwrap()
}
fn clean(o: &std::process::Output, sentinel: &str) {
    assert!(!String::from_utf8_lossy(&o.stdout).contains(sentinel));
    assert!(!String::from_utf8_lossy(&o.stderr).contains(sentinel));
}

#[test]
fn shellout_redacts_secret_rendering() {
    let o = run(
        "secret get \"SHELLY_SENTINEL\" -> $s\nshellout $s\n",
        &[],
        "SECRET_SENTINEL_SHELLOUT",
    );
    assert!(o.status.success());
    clean(&o, "SECRET_SENTINEL_SHELLOUT");
}

#[test]
fn failing_direct_command_redacts_error_surfaces() {
    let o = run(
        "secret get \"SHELLY_SENTINEL\" -> $s\ncommand false $s\n",
        &[],
        "SECRET_SENTINEL_COMMAND",
    );
    assert!(!o.status.success());
    clean(&o, "SECRET_SENTINEL_COMMAND");
}

#[test]
fn failing_http_path_redacts_secret() {
    let o = run(
        "secret get \"SHELLY_SENTINEL\" -> $s\nhttp GET \"https://does-not-exist.invalid/?token=$s\" -> $response\n",
        &[],
        "SECRET_SENTINEL_HTTP",
    );
    assert!(!o.status.success());
    clean(&o, "SECRET_SENTINEL_HTTP");
}

#[test]
fn shell_rejects_secret_interpolation_without_echoing_it() {
    let o = run(
        "secret get \"SHELLY_SENTINEL\" -> $s\nshell \"$s\"\n",
        &[],
        "SECRET_SENTINEL_SHELL",
    );
    assert!(!o.status.success());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("SECRET_SENTINEL_SHELL"));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("secret")
            || String::from_utf8_lossy(&o.stderr).contains("shell")
    );
}

#[test]
fn dry_run_human_and_jsonl_audit_redact_secret() {
    let dir = std::env::temp_dir().join(format!(
        "shelly-secret-audit-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("main.shelly");
    let audit = dir.join("audit.jsonl");
    fs::write(
        &script,
        "secret get \"SHELLY_SENTINEL\" -> $s\ncommand printf $s\n",
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args([
            "--dry-run",
            "--audit-log",
            audit.to_str().unwrap(),
            script.to_str().unwrap(),
        ])
        .env("SHELLY_SENTINEL", "SECRET_SENTINEL_AUDIT")
        .output()
        .unwrap();
    assert!(o.status.success());
    clean(&o, "SECRET_SENTINEL_AUDIT");
    assert!(
        !fs::read_to_string(audit)
            .unwrap()
            .contains("SECRET_SENTINEL_AUDIT")
    );
}
