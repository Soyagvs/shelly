use std::fs;

#[test]
fn mason_package_declares_supported_release_assets_and_binary() {
    let package = fs::read_to_string("packaging/mason-registry/packages/shelly-lsp/package.yaml")
        .expect("Mason package definition should exist");

    for required in [
        "name: shelly-lsp",
        "id: github",
        "repo: Soyagvs/shelly",
        "target: linux_x64",
        "file: shelly-linux-x86_64-${version}.tar.gz",
        "target: darwin_x64",
        "file: shelly-macos-x86_64-${version}.tar.gz",
        "target: darwin_arm64",
        "file: shelly-macos-arm64-${version}.tar.gz",
        "shelly-lsp: shelly-lsp",
    ] {
        assert!(
            package.contains(required),
            "missing Mason field: {required}"
        );
    }
}
