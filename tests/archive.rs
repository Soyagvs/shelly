use flate2::{Compression, write::GzEncoder};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tar::{Builder, Header};
use zip::{ZipWriter, write::SimpleFileOptions};

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "shelly-archive-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn run(src: &str) -> std::process::Output {
    let d = tmp("script");
    let f = d.join("x.shelly");
    fs::write(&f, src).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(&f)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(d);
    o
}
fn zip(path: &Path, entries: &[(&str, &[u8])]) {
    let mut z = ZipWriter::new(fs::File::create(path).unwrap());
    for (n, b) in entries {
        z.start_file(*n, SimpleFileOptions::default()).unwrap();
        z.write_all(b).unwrap();
    }
    z.finish().unwrap();
}
fn tgz(path: &Path, entries: &[(&str, &[u8])]) {
    let f = fs::File::create(path).unwrap();
    let mut b = Builder::new(GzEncoder::new(f, Compression::default()));
    for (n, data) in entries {
        let mut h = Header::new_gnu();
        h.set_path(n).unwrap();
        h.set_size(data.len() as u64);
        h.set_cksum();
        b.append(&h, *data).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap();
}
fn extract(a: &Path, d: &Path) -> std::process::Output {
    run(&format!(
        "archive extract \"{}\" into \"{}\" -> $r\n",
        a.display(),
        d.display()
    ))
}

#[test]
fn zip_nested_round_trip() {
    let r = tmp("zip");
    fs::create_dir_all(r.join("a/b")).unwrap();
    fs::write(r.join("a/b/x"), b"hello").unwrap();
    let a = r.join("x.zip");
    let d = r.join("out");
    let o = run(&format!(
        "archive create \"{}\" from \"{}\" -> $r\n",
        r.join("a").display(),
        a.display()
    ));
    assert!(o.status.success(), "{:?}", o);
    assert!(extract(&a, &d).status.success());
    assert_eq!(fs::read(d.join("b/x")).unwrap(), b"hello");
}
#[test]
fn create_refuses_preexisting_archive_without_replacing_it() {
    let r = tmp("create-existing");
    fs::create_dir_all(r.join("src")).unwrap();
    fs::write(r.join("src/file"), b"new").unwrap();
    let a = r.join("x.zip");
    fs::write(&a, b"keep me").unwrap();
    let o = run(&format!(
        "archive create \"{}\" from \"{}\" -> $r\n",
        r.join("src").display(),
        a.display()
    ));
    assert!(!o.status.success());
    assert_eq!(fs::read(a).unwrap(), b"keep me");
    assert!(!r.join("x.archive-tmp").exists());
}

#[test]
fn tar_gz_nested_round_trip() {
    let r = tmp("tgz");
    let a = r.join("x.tar.gz");
    tgz(&a, &[("a/b.txt", b"ok")]);
    let d = r.join("out");
    assert!(extract(&a, &d).status.success());
    assert_eq!(fs::read(d.join("a/b.txt")).unwrap(), b"ok");
}
#[test]
fn traversal_and_absolute_paths_rejected() {
    for n in ["../x", "/tmp/x"] {
        let r = tmp("unsafe");
        let a = r.join("x.zip");
        zip(&a, &[(n, b"x")]);
        let d = r.join("out");
        assert!(!extract(&a, &d).status.success());
        assert!(!d.exists());
    }
}
#[test]
fn duplicate_and_collision_names_rejected() {
    let r = tmp("names");
    let a = r.join("x.zip");
    zip(&a, &[("a", b"x"), ("a/b", b"y")]);
    assert!(!extract(&a, &r.join("out")).status.success());
}
#[cfg(unix)]
#[test]
fn symlink_hardlink_and_special_members_rejected() {
    use std::os::unix::fs::symlink;
    let r = tmp("special");
    symlink("missing", r.join("link")).unwrap();
    let o = run(&format!(
        "archive create \"{}\" from \"{}\" -> $r\n",
        r.display(),
        r.join("x.zip").display()
    ));
    assert!(!o.status.success());
    let a = r.join("h.tar.gz");
    let mut b = Builder::new(GzEncoder::new(
        fs::File::create(&a).unwrap(),
        Compression::default(),
    ));
    let mut h = Header::new_gnu();
    h.set_entry_type(tar::EntryType::Link);
    h.set_link_name("x").unwrap();
    h.set_size(0);
    h.set_cksum();
    b.append(&h, &b""[..]).unwrap();
    b.into_inner().unwrap().finish().unwrap();
    assert!(!extract(&a, &r.join("out")).status.success());
}
#[test]
fn existing_destination_refused() {
    let r = tmp("exists");
    let a = r.join("x.zip");
    zip(&a, &[("x", b"x")]);
    let d = r.join("out");
    fs::create_dir(&d).unwrap();
    assert!(!extract(&a, &d).status.success());
}
#[test]
fn entry_file_total_and_ratio_limits_rejected() {
    let r = tmp("limits");
    let a = r.join("x.zip");
    zip(&a, &[("x", b"small")]);
    assert!(extract(&a, &r.join("out")).status.success());
    let huge = r.join("huge.zip");
    let mut z = ZipWriter::new(fs::File::create(&huge).unwrap());
    z.start_file("x", SimpleFileOptions::default()).unwrap();
    z.write_all(&vec![b'x'; 300 * 1024 * 1024]).unwrap();
    z.finish().unwrap();
    assert!(!extract(&huge, &r.join("huge-out")).status.success());
}
#[test]
fn rollback_no_write_after_invalid_late_entry() {
    let r = tmp("rollback");
    let a = r.join("x.zip");
    zip(&a, &[("good", b"ok"), ("../bad", b"no")]);
    let d = r.join("out");
    assert!(!extract(&a, &d).status.success());
    assert!(!d.exists());
}
#[test]
fn deterministic_ordering() {
    let r = tmp("order");
    fs::create_dir_all(r.join("src")).unwrap();
    fs::write(r.join("src/z"), b"z").unwrap();
    fs::write(r.join("src/a"), b"a").unwrap();
    for i in 0..2 {
        assert!(
            run(&format!(
                "archive create \"{}\" from \"{}\" -> $r\n",
                r.join("src").display(),
                r.join(format!("{i}.zip")).display()
            ))
            .status
            .success()
        );
    }
    assert_eq!(
        fs::read(r.join("0.zip")).unwrap(),
        fs::read(r.join("1.zip")).unwrap()
    );
}
#[test]
fn checksum_matches_sha256() {
    let r = tmp("sum");
    let p = r.join("x");
    fs::write(&p, b"abc").unwrap();
    let o = run(&format!(
        "archive checksum \"{}\" -> $r\nshellout $r\n",
        p.display()
    ));
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("ba7816bf"));
}
#[test]
fn check_grammar_and_no_side_effect() {
    let r = tmp("check");
    let a = r.join("x.zip");
    zip(&a, &[("x", b"x")]);
    let o = run(&format!("archive check \"{}\" -> $r\n", a.display()));
    assert!(o.status.success());
    assert!(!r.join("x.check-tmp").exists());
    let bad = run("archive check -> $r\n");
    assert!(!bad.status.success());
}
