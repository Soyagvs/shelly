use std::fs;
use std::process::Command;

fn dir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "shelly-mock-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&p);
    p
}
fn run(d: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(args)
        .env_remove("SHELLY_TEST_MODE")
        .current_dir(d)
        .output()
        .unwrap()
}
#[test]
fn ordinary_scripts_reject_mock() {
    let d = dir();
    fs::write(d.join("main.shelly"), "mock read \"x\" returns \"y\"\n").unwrap();
    let o = run(&d, &["main.shelly"]);
    assert!(!o.status.success());
}
#[test]
fn mocked_read_returns_virtual_content_without_creating_file() {
    let d = dir();
    fs::create_dir_all(d.join("tests")).unwrap();
    fs::write(
        d.join("Shelly.toml"),
        "[project]\nname=\"x\"\nversion=\"1\"\nentry=\"main.shelly\"\n",
    )
    .unwrap();
    fs::write(d.join("main.shelly"), "").unwrap();
    fs::write(d.join("tests/read_test.shelly"), "mock read \"virtual.txt\" returns \"hello\"\nread \"./virtual.txt\" -> $value\nassertEqual $value \"hello\"\n").unwrap();
    let o = run(&d, &["test"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!d.join("virtual.txt").exists());
}

#[test]
fn unmatched_mocked_read_fails_closed() {
    let d = dir();
    fs::create_dir_all(d.join("tests")).unwrap();
    fs::write(
        d.join("Shelly.toml"),
        "[project]\nname=\"x\"\nversion=\"1\"\nentry=\"main.shelly\"\n",
    )
    .unwrap();
    fs::write(d.join("main.shelly"), "").unwrap();
    fs::write(
        d.join("tests/read_test.shelly"),
        "mock read \"virtual.txt\" returns \"hello\"\nread \"other.txt\" -> $value\n",
    )
    .unwrap();
    let o = run(&d, &["test"]);
    assert!(!o.status.success());
    let error = String::from_utf8_lossy(&o.stderr);
    assert!(error.contains("no exact read mock"), "{error}");
}

#[test]
fn test_runner_allows_declarations_but_does_not_share_registries() {
    let d = dir();
    fs::create_dir_all(d.join("tests")).unwrap();
    fs::write(
        d.join("Shelly.toml"),
        "[project]\nname=\"x\"\nversion=\"1\"\nentry=\"main.shelly\"\n",
    )
    .unwrap();
    fs::write(d.join("main.shelly"), "").unwrap();
    let declaration = "mock read \"x\" returns \"y\"\n";
    fs::write(d.join("tests/a_test.shelly"), declaration).unwrap();
    fs::write(d.join("tests/b_test.shelly"), declaration).unwrap();
    let o = run(&d, &["test"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
