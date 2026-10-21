use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
static N: AtomicUsize = AtomicUsize::new(0);

fn run(script: &str) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!(
        "shelly-dry-run-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.shelly");
    std::fs::write(&path, script).unwrap();
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(["--dry-run", path.to_str().unwrap()])
        .output()
        .unwrap()
}

#[test]
fn audit_log_is_typed_jsonl_with_terminal_summary() {
    let dir = std::env::temp_dir().join(format!(
        "shelly-audit-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("main.shelly");
    let log = dir.join("audit.jsonl");
    std::fs::write(
        &script,
        format!("create \"{}\"\n", dir.join("out").display()),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args([
            "--dry-run",
            "--audit-log",
            log.to_str().unwrap(),
            script.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records: Vec<serde_json::Value> = std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(records.len() >= 2);
    assert_eq!(records.last().unwrap()["event"], "summary");
    for (i, record) in records.iter().enumerate() {
        assert_eq!(record["schema"], "shelly.audit.v1");
        assert_eq!(record["seq"], i as u64 + 1);
        assert!(record["data"].is_object());
    }
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("summary")
            || String::from_utf8_lossy(&output.stdout).contains("DRY-RUN")
    );
}

#[test]
fn dry_run_validates_and_reports_safe_ordered_effect_details_without_mutation() {
    let dir = std::env::temp_dir().join(format!("shelly-dry-state-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sentinel = dir.join("sentinel");
    std::fs::write(&sentinel, "keep").unwrap();
    let script = format!(
        "write \"{}\" \"secret payload\"\nremove \"{}\"\ncommand printf secret -> $out\nshell \"touch {}\"\nhttp GET \"https://example.com/a\" -> $response\n",
        sentinel.display(),
        sentinel.display(),
        dir.join("network").display()
    );
    let output = run(&script);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "keep");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("write") && stdout.contains(sentinel.to_string_lossy().as_ref()));
    assert!(stdout.contains("command") && stdout.contains("printf"));
    assert!(
        stdout.contains("shell")
            && stdout.contains("http")
            && stdout.contains("GET")
            && stdout.contains("https://example.com/a")
    );
    assert!(!stdout.contains("secret payload") && !stdout.contains("blocked"));
    let positions: Vec<_> = [
        "DRY-RUN action: write ",
        "DRY-RUN action: remove ",
        "DRY-RUN action: command ",
        "DRY-RUN action: shell ",
        "DRY-RUN action: http ",
    ]
    .iter()
    .map(|x| stdout.find(x).unwrap())
    .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn dry_run_wait_reports_blocked_action_and_returns_promptly() {
    let dir = std::env::temp_dir().join(format!(
        "shelly-dry-wait-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.shelly");
    std::fs::write(&path, "wait\n").unwrap();
    let output = Command::new("timeout")
        .args([
            "2",
            env!("CARGO_BIN_EXE_shelly"),
            "--dry-run",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "wait did not return promptly: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY-RUN action: wait "), "{stdout}");
    assert!(
        stdout.contains("planned") && stdout.contains("blocked"),
        "{stdout}"
    );
}

#[test]
fn dry_run_watch_validates_without_spawning_or_blocking() {
    let output = run("watch sh -c \"sleep 30\" -> $line\n    shellout $line\nend\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY-RUN action: watch "), "{stdout}");
    assert!(!stdout.contains("sleep 30"), "{stdout}");
}

#[test]
fn dry_run_rejects_malformed_effect_syntax() {
    let output = run("write \"x\"\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("write"));
}

#[test]
fn dry_run_archive_rename_and_temp_do_not_mutate() {
    let output = run(
        "rename \"missing\" \"new\"\ntempFile -> $tmp\narchive create \"missing\" from \"out.zip\" -> $a\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rename") && stdout.contains("temp") && stdout.contains("archive"));
}

#[test]
fn dry_run_records_resolved_mutation_paths_and_unavailable_captures() {
    let output = run(
        "create \"new-file\"\ncopy \"new-file\" \"copy-file\"\nmove \"copy-file\" \"moved-file\"\nappend \"moved-file\" \"payload\"\nremove \"moved-file\"\ncommand echo hi -> $out\nshell \"echo hi\" -> $sh\nhttp GET \"https://example.com\" -> $web\nshellout $web\n",
    );
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("dry-run unavailable-result"), "{text}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for path in ["new-file", "copy-file", "moved-file"] {
        assert!(stdout.contains(path), "{stdout}");
    }
    assert!(stdout.contains("DRY-RUN action: create "));
    assert!(!stdout.contains("payload"));
}

#[test]
fn dry_run_capture_destinations_hide_stale_values() {
    for script in [
        "$result = \"stale\"\narchive create \"missing\" from \"out.zip\" -> $result\nshellout $result\n",
        "$result = \"stale\"\ntempFile -> $result\nshellout $result\n",
        "$result = \"stale\"\ntempDir -> $result\nshellout $result\n",
    ] {
        let output = run(script);
        assert!(
            !output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("dry-run unavailable-result"));
    }
}

#[test]
fn dry_run_choose_blocks_without_reading_stdin_or_faking_result() {
    let output = run("choose \"Pick\" [\"one\", \"two\"] -> $choice\n");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("dry-run unavailable-result"), "{stderr}");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Choose:"));
}

#[test]
fn dry_run_archive_validates_arguments_before_blocking() {
    let output = run("archive create \"missing\" -> $a\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("archive create syntax invalid"));
}

#[test]
fn normal_script_execution_is_unchanged() {
    let dir = std::env::temp_dir().join(format!("shelly-normal-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.shelly");
    std::fs::write(&path, "shellout \"hello\"\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}
