use std::process::Command;

#[test]
fn both_binaries_report_package_version_without_starting_protocol() {
    for binary in [
        env!("CARGO_BIN_EXE_shelly"),
        env!("CARGO_BIN_EXE_shelly-lsp"),
    ] {
        let output = Command::new(binary).arg("--version").output().unwrap();
        assert!(output.status.success());
        let expected_name = if binary.ends_with("shelly-lsp") {
            "shelly-lsp"
        } else {
            "shelly"
        };
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            format!("{expected_name} {}", env!("CARGO_PKG_VERSION"))
        );
        assert!(output.stderr.is_empty());
    }
}
