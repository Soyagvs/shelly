use std::fs;
use std::path::PathBuf;

fn installer(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(path).expect("installer script should be readable")
}

#[test]
fn installers_require_and_install_cli_and_lsp_binaries() {
    let unix = installer("install.sh");
    assert!(unix.contains("[ -f \"$tmpdir/shelly\" ]"));
    assert!(unix.contains("[ -f \"$tmpdir/shelly-lsp\" ]"));
    assert!(unix.contains("cp \"$tmpdir/shelly\" \"$INSTALL_DIR/shelly\""));
    assert!(unix.contains("cp \"$tmpdir/shelly-lsp\" \"$INSTALL_DIR/shelly-lsp\""));
    assert!(unix.contains("chmod +x \"$tmpdir/shelly\" \"$tmpdir/shelly-lsp\""));

    let windows = installer("install.ps1");
    assert!(windows.contains("Join-Path $TempDir \"shelly.exe\""));
    assert!(windows.contains("Join-Path $TempDir \"shelly-lsp.exe\""));
    assert!(windows.contains("Join-Path $InstallDir \"shelly.exe\""));
    assert!(windows.contains("Join-Path $InstallDir \"shelly-lsp.exe\""));
    assert!(windows.contains("Downloaded archive did not contain shelly.exe"));
    assert!(windows.contains("Downloaded archive did not contain shelly-lsp.exe"));
}
