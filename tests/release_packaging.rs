use std::fs;

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
