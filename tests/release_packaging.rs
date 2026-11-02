use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn release_matrix_documents_supported_targets_and_bundles_both_tools() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();
    for target in ["linux-x86_64", "macos-x86_64", "macos-arm64"] {
        assert!(workflow.contains(&format!("target: {target}")));
    }
    assert!(workflow.contains("tar -C target/release -czf \"$archive\" shelly shelly-lsp"));
    assert!(workflow.contains("sha256sum \"$archive\" > \"$archive.sha256\""));
    assert!(workflow.contains("${{ matrix.target }}-${GITHUB_REF_NAME}"));
    assert!(workflow.contains("target: macos-arm64"));
}

#[test]
fn release_matrix_uses_supported_macos_runners() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(workflow.contains("label: macos-x86_64\n            os: macos-15-intel"));
    assert!(workflow.contains("label: macos-arm64\n            os: macos-latest"));
    assert!(!workflow.contains("macos-13"));
}

#[test]
fn release_verifier_requires_both_executables() {
    let verifier = fs::read_to_string("scripts/release-verify.sh").unwrap();
    assert!(verifier.contains("shelly-lsp"));
    assert!(verifier.contains("shelly=1"));
    assert!(verifier.contains("lsp=1"));
}

#[test]
fn release_downloads_are_stable_newest_first_and_retained_for_ten_releases() {
    let dir = fixture_copy();
    let output = run_generator(&dir);
    assert!(output.status.success(), "generator failed: {}", String::from_utf8_lossy(&output.stderr));
    let readme = fs::read_to_string(dir.join("README.md")).unwrap();
    let start = readme.find("<!-- release-downloads:start -->").unwrap();
    let end = readme.find("<!-- release-downloads:end -->").unwrap();
    let generated = &readme[start..end + "<!-- release-downloads:end -->".len()];
    let expected = [
        "<!-- release-downloads:start -->",
        "- [v1.4.0](https://github.com/Soyagvs/shelly/releases/tag/v1.4.0): 7 downloads",
        "- [v1.3.0](https://github.com/Soyagvs/shelly/releases/tag/v1.3.0): 2 downloads",
        "- [v1.2.0](https://github.com/Soyagvs/shelly/releases/tag/v1.2.0): 1 downloads",
        "- [v1.1.0](https://github.com/Soyagvs/shelly/releases/tag/v1.1.0): 8 downloads",
        "- [v1.0.0](https://github.com/Soyagvs/shelly/releases/tag/v1.0.0): 5 downloads",
        "- [v0.9.0](https://github.com/Soyagvs/shelly/releases/tag/v0.9.0): 6 downloads",
        "- [v0.8.0](https://github.com/Soyagvs/shelly/releases/tag/v0.8.0): 7 downloads",
        "- [v0.7.0](https://github.com/Soyagvs/shelly/releases/tag/v0.7.0): 8 downloads",
        "- [v0.6.0](https://github.com/Soyagvs/shelly/releases/tag/v0.6.0): 9 downloads",
        "- [v0.5.0](https://github.com/Soyagvs/shelly/releases/tag/v0.5.0): 10 downloads",
        "<!-- release-downloads:end -->",
    ].join("\n");
    assert_eq!(generated, expected);
    assert!(readme.contains("Keep this introduction.") && readme.contains("Keep this conclusion."));
    assert!(!readme.contains("v1.5.0-rc1"));
}

#[test]
fn release_downloads_fail_without_modifying_readme_when_markers_are_invalid() {
    for marker_case in ["missing", "duplicate"] {
        let dir = fixture_copy();
        let path = dir.join("README.md");
        let original = fs::read_to_string(&path).unwrap();
        let invalid = if marker_case == "missing" {
            original.replace("<!-- release-downloads:start -->", "")
        } else {
            original.replace("<!-- release-downloads:start -->", "<!-- release-downloads:start -->\\n<!-- release-downloads:start -->")
        };
        fs::write(&path, &invalid).unwrap();
        let output = run_generator(&dir);
        assert!(!output.status.success(), "{marker_case} markers unexpectedly succeeded");
        assert_eq!(fs::read_to_string(path).unwrap(), invalid);
    }
}

fn fixture_copy() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shelly-release-downloads-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir_all(&dir).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/release-downloads");
    fs::copy(fixtures.join("README.md"), dir.join("README.md")).unwrap();
    fs::copy(fixtures.join("releases.json"), dir.join("releases.json")).unwrap();
    dir
}

fn run_generator(dir: &Path) -> std::process::Output {
    Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/update-release-downloads.sh"))
        .arg(dir.join("releases.json"))
        .arg(dir.join("README.md"))
        .output()
        .unwrap()
}
