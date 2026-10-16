use sha2::{Digest, Sha256};
use shelly::update;
use std::{fs, path::PathBuf};

#[cfg(windows)]
use std::process::Command;

fn temp_dir(label: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("shelly-update-test-{label}-{}", std::process::id()));
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn checksum_mismatch_is_rejected() {
    let expected: [u8; 32] = Sha256::digest(b"other").into();
    assert!(update::verify_checksum(b"archive", expected).is_err());
}

#[test]
fn versioned_assets_require_matching_tag_and_target() {
    let name = update::versioned_archive_name("shelly-linux-x86_64.tar.gz", "v1.2.3").unwrap();
    assert_eq!(name, "shelly-linux-x86_64-v1.2.3.tar.gz");
    for tag in ["release", "v1..3", "v1.2", "v1.2.3-alpha", "v1.2.3 "] {
        assert!(
            update::versioned_archive_name("shelly-linux-x86_64.tar.gz", tag).is_err(),
            "{tag}"
        );
    }
    let assets = serde_json::from_str::<serde_json::Value>(&format!(r#"[
      {{"name":"{name}","browser_download_url":"https://github.com/Soyagvs/shelly/releases/download/v1.2.3/{name}"}},
      {{"name":"shelly-windows-x86_64-v1.2.3.zip","browser_download_url":"https://github.com/Soyagvs/shelly/releases/download/v1.2.3/wrong.zip"}}
    ]"#)).unwrap();
    assert!(
        update::asset_url(
            assets.as_array().unwrap(),
            "shelly-windows-x86_64-v1.2.3.zip"
        )
        .is_err()
    );
    assert!(update::asset_url(assets.as_array().unwrap(), "wrong-tag.tar.gz").is_err());
}

#[test]
fn archive_rejects_duplicate_and_hard_link_entries() {
    let mut raw = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        for data in [b"a".as_slice(), b"b".as_slice()] {
            let mut h = tar::Header::new_gnu();
            h.set_size(1);
            h.set_cksum();
            tar.append_data(&mut h, "shelly", data).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    let root = temp_dir("archive");
    assert!(update::extract("x.tar.gz", &raw, &root).is_err());
    let mut raw = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        let mut h = tar::Header::new_gnu();
        h.set_entry_type(tar::EntryType::Link);
        h.set_link_name("shelly").unwrap();
        h.set_size(0);
        h.set_cksum();
        tar.append_data(&mut h, "shelly-lsp", &[][..]).unwrap();
        tar.into_inner().unwrap().finish().unwrap();
    }
    assert!(update::extract("x.tar.gz", &raw, &root).is_err());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn zip_rejects_non_regular_entries() {
    let mut raw = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut raw);
        let options = zip::write::SimpleFileOptions::default();
        zip.add_directory("shelly-lsp", options).unwrap();
        zip.start_file("shelly", options).unwrap();
        use std::io::Write;
        zip.write_all(b"binary").unwrap();
        zip.finish().unwrap();
    }
    let root = temp_dir("zip-entry-kind");
    assert!(update::extract("x.zip", raw.get_ref(), &root).is_err());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn replacement_rolls_back_both_binaries() {
    let root = temp_dir("rollback");
    let a = root.join("shelly");
    let b = root.join("shelly-lsp");
    let sa = root.join("new-shelly");
    let sb = root.join("missing-lsp");
    fs::write(&a, b"old-a").unwrap();
    fs::write(&b, b"old-b").unwrap();
    fs::write(&sa, b"new-a").unwrap();
    assert!(
        update::transactional_replace(&[(sa.clone(), a.clone()), (sb.clone(), b.clone())], true)
            .is_err()
    );
    assert_eq!(fs::read(a).unwrap(), b"old-a");
    assert_eq!(fs::read(b).unwrap(), b"old-b");
    assert!(sa.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn successful_replacement_removes_staged_binaries() {
    let root = temp_dir("success-cleanup");
    let a = root.join("shelly");
    let b = root.join("shelly-lsp");
    let sa = root.join("new-shelly");
    let sb = root.join("new-shelly-lsp");
    fs::write(&a, b"old-a").unwrap();
    fs::write(&b, b"old-b").unwrap();
    fs::write(&sa, b"new-a").unwrap();
    fs::write(&sb, b"new-b").unwrap();

    update::transactional_replace(&[(sa.clone(), a.clone()), (sb.clone(), b.clone())], true)
        .unwrap();
    assert_eq!(fs::read(a).unwrap(), b"new-a");
    assert_eq!(fs::read(b).unwrap(), b"new-b");
    assert!(!sa.exists());
    assert!(!sb.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn staged_cleanup_failure_is_non_fatal_and_retains_temp_path() {
    let root = temp_dir("cleanup-failure");
    let staged = root.join("staged");
    fs::create_dir(&staged).unwrap();

    update::cleanup_staged(&staged);

    assert!(staged.is_dir());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn helper_rejects_unpaired_replacement_arguments() {
    let err = update::helper(["a", "b", "c"].into_iter().map(Into::into));
    assert!(err.unwrap_err().contains("paired update arguments"));
}

#[cfg(windows)]
#[test]
fn windows_deferred_hand_off_replaces_binaries() {
    let root = std::env::temp_dir().join(format!(
        "shelly-update-handoff-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    struct TempDir(PathBuf);
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = TempDir(root.clone());
    let staged_a = root.join("staged-a.exe");
    let destination_a = root.join("destination-a.exe");
    let staged_b = root.join("staged-b.exe");
    let destination_b = root.join("destination-b.exe");
    fs::write(&staged_a, b"new-a").unwrap();
    fs::write(&destination_a, b"old-a").unwrap();
    fs::write(&staged_b, b"new-b").unwrap();
    fs::write(&destination_b, b"old-b").unwrap();

    let mut original = Command::new("cmd.exe")
        .args(["/C", "exit", "0"])
        .spawn()
        .unwrap();
    let original_pid = original.id().to_string();
    let helper = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("--shelly-update-helper")
        .arg(&staged_a)
        .arg(&destination_a)
        .arg(&staged_b)
        .arg(&destination_b)
        .arg("--shelly-update-pid")
        .arg(original_pid)
        .spawn()
        .unwrap();
    let _ = original.wait().unwrap();
    let helper_status = helper.wait_with_output().unwrap();
    assert!(helper_status.status.success(), "{helper_status:?}");
    assert_eq!(fs::read(&destination_a).unwrap(), b"new-a");
    assert_eq!(fs::read(&destination_b).unwrap(), b"new-b");
    assert!(!staged_a.exists());
    assert!(!staged_b.exists());
    assert!(!destination_a.with_extension("shelly-update").exists());
    assert!(!destination_b.with_extension("shelly-update").exists());
    assert!(!destination_a.with_extension("shelly-backup").exists());
    assert!(!destination_b.with_extension("shelly-backup").exists());
    let _ = fs::remove_dir_all(root);
}
