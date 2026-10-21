use shelly::formatter::{format_file, format_source};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn temp_script(source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "shelly-formatter-{}-{}.shelly",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).expect("write formatter fixture");
    path
}

#[test]
fn non_ascii_input_does_not_panic() {
    let result = std::panic::catch_unwind(|| format_source("shellout \"こんにちは 🌍\"\n"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}

#[test]
fn strings_comments_and_multiline_strings_are_preserved() {
    let source = "  shellout \"a # b\" # trailing\n#\ncomment # text\n#\n```\n  literal # text\n```\n";
    let formatted = format_source(source).expect("format source");
    assert_eq!(formatted, "shellout \"a # b\" #\n#\ncomment # text\n#\n```\n  literal # text\n```\n");
}

#[test]
fn nested_blocks_are_indented() {
    let source = "if true\n shellout \"outer\"\n for $item in $items\n shellout $item\n end\nend\n";
    assert_eq!(
        format_source(source).unwrap(),
        "if true\n    shellout \"outer\"\n    for $item in $items\n        shellout $item\n    end\nend\n"
    );
}

#[test]
fn formatting_is_idempotent() {
    let once = format_source("if true\n shellout \"x\"\nend\n").unwrap();
    assert_eq!(format_source(&once).unwrap(), once);
}

#[test]
fn malformed_sources_are_rejected() {
    for source in [
        "end\n",
        "if true\n",
        "shellout \"unterminated\n",
        "```\ntext\n",
        "# comment\nstill comment\n",
    ] {
        assert!(format_source(source).is_err(), "accepted {source:?}");
    }
}

#[test]
fn format_file_check_reports_clean_and_dirty_status() {
    let path = temp_script(" if true\n shellout \"x\"\nend\n");
    assert!(!format_file(&path, true).unwrap());
    assert!(format_file(&path, false).unwrap());
    assert!(format_file(&path, true).unwrap());
}

#[test]
fn fmt_cli_check_statuses_and_argument_order_are_exact() {
    let path = temp_script(" shellout \"x\"\n");
    let bin = env!("CARGO_BIN_EXE_shelly");
    let dirty = Command::new(bin).args(["fmt", "--check", path.to_str().unwrap()]).output().unwrap();
    assert!(!dirty.status.success());
    let clean = Command::new(bin).args(["fmt", path.to_str().unwrap()]).output().unwrap();
    assert!(clean.status.success());
    let checked = Command::new(bin).args(["fmt", "--check", path.to_str().unwrap()]).output().unwrap();
    assert!(checked.status.success());
    let wrong_order = Command::new(bin).args(["fmt", path.to_str().unwrap(), "--check"]).output().unwrap();
    assert!(!wrong_order.status.success());
    assert!(String::from_utf8_lossy(&wrong_order.stderr).contains("Usage: shelly fmt"));
}
