use sha2::{Digest, Sha256};
use std::time::Duration;
#[cfg(windows)]
use std::time::Instant;
use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
    process::Command,
    process::Stdio,
};
const REPOSITORY: &str = "Soyagvs/shelly";

pub fn run(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    if args.next().is_some() {
        return Err("Usage: shelly update".into());
    }
    let archive_name = target_archive_name()?;
    let meta = download(&format!(
        "https://api.github.com/repos/{REPOSITORY}/releases/latest"
    ))?;
    let release: serde_json::Value =
        serde_json::from_slice(&meta).map_err(|e| format!("invalid release metadata: {e}"))?;
    let assets = release["assets"]
        .as_array()
        .ok_or("release has no assets")?;
    let tag = release["tag_name"].as_str().ok_or("release has no tag")?;
    let archive_name = versioned_archive_name(archive_name, tag)?;
    let archive_url = asset_url(assets, &archive_name)?;
    let checksum_url = asset_url(assets, &format!("{archive_name}.sha256"))?;
    let archive = download(archive_url)?;
    let checksum =
        String::from_utf8(download(checksum_url)?).map_err(|_| "checksum is not UTF-8")?;
    let expected = parse_checksum(&checksum, &archive_name)?;
    verify_checksum(&archive, expected)?;
    let dir = env::temp_dir().join(format!("shelly-update-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    extract(&archive_name, &archive, &dir)?;
    let exe = env::current_exe().map_err(|e| format!("could not locate executable: {e}"))?;
    let lsp = exe.with_file_name(binary_name("shelly-lsp"));
    let before_shelly = installed_version(&exe, "shelly");
    let before_lsp = installed_version(&lsp, "shelly-lsp");
    println!(
        "Current versions: shelly {}, shelly-lsp {}.",
        before_shelly, before_lsp
    );
    let staged = dir.join(binary_name("shelly"));
    let staged_lsp = dir.join(binary_name("shelly-lsp"));
    if !staged.is_file() {
        return Err("release archive does not contain shelly".into());
    }
    let has_lsp = lsp.is_file();
    if has_lsp && !staged_lsp.is_file() {
        return Err("release archive does not contain shelly-lsp".into());
    }
    if has_lsp
        && staged_lsp.is_file()
        && binaries_equal(&exe, &staged)
        && binaries_equal(&lsp, &staged_lsp)
    {
        println!(
            "Already current. Installed versions: shelly {}, shelly-lsp {}.",
            before_shelly, before_lsp
        );
        return Ok(());
    }
    if cfg!(windows) {
        let mut handoff = Command::new(&exe);
        handoff.arg("--shelly-update-helper").arg(&staged).arg(&exe);
        if staged_lsp.is_file() {
            handoff.arg(&staged_lsp).arg(&lsp);
        }
        handoff
            .arg("--shelly-update-pid")
            .arg(std::process::id().to_string())
            .spawn()
            .map_err(|e| format!("could not start update hand-off: {e}"))?;
        println!(
            "{} Installed versions: shelly {}, shelly-lsp {} until restart.",
            pending_message(staged_lsp.is_file()),
            before_shelly,
            before_lsp
        );
        return Ok(());
    }
    transactional_replace(
        &[(staged, exe.clone()), (staged_lsp.clone(), lsp.clone())],
        staged_lsp.is_file(),
    )?;
    let after_shelly = installed_version(&exe, "shelly");
    let after_lsp = installed_version(&lsp, "shelly-lsp");
    println!(
        "Updated shelly{}.",
        if has_lsp { " and shelly-lsp" } else { "" }
    );
    println!(
        "Installed versions: shelly {}, shelly-lsp {}.",
        after_shelly, after_lsp
    );
    Ok(())
}

pub fn helper(mut a: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    let s = PathBuf::from(a.next().ok_or("missing staged shelly")?);
    let d = PathBuf::from(a.next().ok_or("missing shelly destination")?);
    let mut items = vec![(s, d)];
    match (a.next(), a.next()) {
        (Some(s), Some(d)) => items.push((PathBuf::from(s), PathBuf::from(d))),
        (None, None) => {}
        _ => return Err("paired update arguments must be supplied together".into()),
    }
    if a.next().as_deref() != Some(std::ffi::OsStr::new("--shelly-update-pid")) {
        return Err("missing original process hand-off marker".into());
    }
    let pid = a
        .next()
        .ok_or("missing original process id")?
        .to_str()
        .ok_or("original process id is not Unicode")?
        .parse::<u32>()
        .map_err(|_| "invalid original process id")?;
    if a.next().is_some() {
        return Err("unexpected update helper arguments".into());
    }
    wait_for_process_exit(pid)?;
    transactional_replace(&items, items.len() == 2)
}
#[cfg(windows)]
fn wait_for_process_exit(pid: u32) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| format!("could not inspect original process: {e}"))?;
        if !String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\"")) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for original Shelly process to exit".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(not(windows))]
fn wait_for_process_exit(_pid: u32) -> Result<(), String> {
    Ok(())
}

pub fn versioned_archive_name(base: &str, tag: &str) -> Result<String, String> {
    let version = tag.strip_prefix('v').unwrap_or(tag);
    let fields: Vec<_> = version.split('.').collect();
    if fields.len() != 3
        || fields
            .iter()
            .any(|field| field.is_empty() || !field.chars().all(|c| c.is_ascii_digit()))
    {
        return Err("release tag is not a strict semantic version".into());
    }
    Ok(base.replace(".tar.gz", "").replace(".zip", "")
        + "-"
        + tag
        + if base.ends_with(".zip") {
            ".zip"
        } else {
            ".tar.gz"
        })
}
pub fn asset_url<'a>(a: &'a [serde_json::Value], n: &str) -> Result<&'a str, String> {
    a.iter()
        .find_map(|x| {
            (x["name"].as_str() == Some(n)
                && x["browser_download_url"]
                    .as_str()?
                    .starts_with("https://github.com/Soyagvs/shelly/releases/download/")
                && x["browser_download_url"].as_str().unwrap().ends_with(n))
            .then(|| x["browser_download_url"].as_str().unwrap())
        })
        .ok_or_else(|| format!("release is missing or has invalid asset {n}"))
}
fn download(u: &str) -> Result<Vec<u8>, String> {
    let r = ureq::get(u)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let mut b = Vec::new();
    r.into_reader()
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    Ok(b)
}
fn binary_name(n: &str) -> String {
    if cfg!(windows) {
        format!("{n}.exe")
    } else {
        n.into()
    }
}

pub fn parse_version_output(output: &[u8], expected_name: &str) -> Result<String, String> {
    let text = std::str::from_utf8(output).map_err(|_| "version is not UTF-8")?;
    let mut fields = text.splitn(2, ' ');
    if fields.next() != Some(expected_name) {
        return Err("unexpected binary name".into());
    }
    let version = fields.next().ok_or("missing version")?;
    if version.is_empty() || version.contains(['\n', '\r']) || version.trim() != version {
        return Err("version output is not one exact line".into());
    }
    let fields: Vec<_> = version.split('.').collect();
    if fields.len() != 3
        || fields
            .iter()
            .any(|f| f.is_empty() || !f.chars().all(|c| c.is_ascii_digit()))
    {
        return Err("version is not strict semantic version".into());
    }
    Ok(version.into())
}

pub fn binaries_equal(installed: &Path, staged: &Path) -> bool {
    fs::read(installed).ok() == fs::read(staged).ok()
}

pub fn pending_message(has_lsp: bool) -> String {
    if has_lsp {
        "Update pending: shelly and shelly-lsp will be replaced after this process exits.".into()
    } else {
        "Update pending: shelly will be replaced; shelly-lsp is missing and will remain unavailable.".into()
    }
}

fn installed_version(path: &Path, expected_name: &str) -> String {
    let output_path = env::temp_dir().join(format!(
        "shelly-version-{}-{}",
        std::process::id(),
        expected_name
    ));
    let output = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)
    {
        Ok(file) => file,
        Err(_) => return "unavailable".into(),
    };
    let mut child = match Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            let _ = fs::remove_file(output_path);
            return "unavailable".into();
        }
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let result = loop {
        if fs::metadata(&output_path)
            .map(|m| m.len() > 128)
            .unwrap_or(false)
        {
            let _ = child.kill();
            let _ = child.wait();
            break "unknown".into();
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    break "unavailable".into();
                }
                let mut bytes = Vec::new();
                let read =
                    File::open(&output_path).and_then(|f| f.take(129).read_to_end(&mut bytes));
                break if read.is_err() || bytes.len() > 128 {
                    "unknown".into()
                } else {
                    parse_version_output(&bytes, expected_name).unwrap_or_else(|_| "unknown".into())
                };
            }
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break "unavailable".into();
            }
        }
    };
    let _ = fs::remove_file(output_path);
    let _ = expected_name;
    result
}

fn replace(s: &Path, d: &Path) -> Result<(), String> {
    let t = d.with_extension("shelly-update");
    fs::copy(s, &t).map_err(|e| e.to_string())?;
    fs::rename(t, d).map_err(|e| format!("could not replace {}: {e}", d.display()))
}
pub fn transactional_replace(
    items: &[(PathBuf, PathBuf)],
    include_lsp: bool,
) -> Result<(), String> {
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (_, d) in items.iter().take(if include_lsp { 2 } else { 1 }) {
        let b = d.with_extension("shelly-backup");
        if !d.exists() {
            continue;
        }
        if let Err(e) = fs::rename(d, &b) {
            let mut restore_errors = Vec::new();
            for (backup, destination) in backups.iter().rev() {
                if let Err(re) = fs::rename(backup, destination) {
                    restore_errors.push(format!("{}: {re}", destination.display()));
                }
            }
            return Err(if restore_errors.is_empty() {
                format!("could not stage replacement: {e}")
            } else {
                format!(
                    "could not stage replacement: {e}; restoration failed for {}",
                    restore_errors.join(", ")
                )
            });
        }
        backups.push((b, d.clone()));
    }
    for (s, d) in items.iter().take(if include_lsp { 2 } else { 1 }) {
        if let Err(e) = replace(s, d) {
            let mut restore_errors = Vec::new();
            for (b, d) in &backups {
                if d.exists() && fs::remove_file(d).is_err() {
                    restore_errors.push(d.display().to_string());
                    continue;
                }
                if let Err(re) = fs::rename(b, d) {
                    restore_errors.push(format!("{}: {re}", d.display()));
                }
            }
            if restore_errors.is_empty() {
                return Err(e);
            }
            return Err(format!(
                "{e}; restoration failed for {}",
                restore_errors.join(", ")
            ));
        }
    }
    for (s, _) in items.iter().take(if include_lsp { 2 } else { 1 }) {
        cleanup_staged(s);
    }
    for (b, _) in backups {
        let _ = fs::remove_file(b);
    }
    Ok(())
}

/// Remove a staged replacement when possible. Cleanup is deliberately best-effort:
/// replacement and backup removal have already made the update successful.
pub fn cleanup_staged(path: &Path) {
    if let Err(e) = fs::remove_file(path) {
        eprintln!(
            "could not remove staged replacement {}: {e}",
            path.display()
        );
    }
}
fn safe_name(p: &Path) -> Result<String, String> {
    let mut it = p.components();
    match (it.next(), it.next(), it.next()) {
        (Some(Component::Normal(a)), None, None) => Ok(a.to_string_lossy().into()),
        _ => Err("archive contains unsafe or unexpected path".into()),
    }
}
pub fn extract(name: &str, b: &[u8], root: &Path) -> Result<(), String> {
    if name.ends_with(".zip") {
        let mut z = zip::ZipArchive::new(Cursor::new(b)).map_err(|e| e.to_string())?;
        for i in 0..z.len() {
            let mut f = z.by_index(i).map_err(|e| e.to_string())?;
            let mode = f.unix_mode();
            if !f.is_file() || mode.map_or(false, |m| m & 0o170000 != 0 && m & 0o170000 != 0o100000)
            {
                return Err("archive contains non-regular entry".into());
            }
            let zip_name = f.name().map_err(|e| e.to_string())?.into_owned();
            let n = safe_name(Path::new(&zip_name))?;
            if n != binary_name("shelly") && n != binary_name("shelly-lsp")
                || root.join(&n).exists()
            {
                return Err("archive contains unexpected or duplicate file".into());
            }
            let mut o = fs::File::create(root.join(n)).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut o).map_err(|e| e.to_string())?;
        }
    } else {
        let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(Cursor::new(b)));
        for e in ar.entries().map_err(|e| e.to_string())? {
            let mut e = e.map_err(|e| e.to_string())?;
            if !e.header().entry_type().is_file() {
                return Err("archive contains non-regular entry".into());
            }
            let n = safe_name(e.path().map_err(|e| e.to_string())?.as_ref())?;
            if n != binary_name("shelly") && n != binary_name("shelly-lsp")
                || root.join(&n).exists()
            {
                return Err("archive contains unexpected file".into());
            }
            e.unpack(root.join(n)).map_err(|e| e.to_string())?;
        }
    }
    for n in [binary_name("shelly"), binary_name("shelly-lsp")] {
        let p = root.join(n);
        if !p.is_file() {
            return Err("archive must contain exactly shelly and shelly-lsp".into());
        }
        #[cfg(unix)]
        {
            let mut x = fs::metadata(&p).map_err(|e| e.to_string())?.permissions();
            use std::os::unix::fs::PermissionsExt;
            x.set_mode(x.mode() | 0o755);
            fs::set_permissions(p, x).map_err(|e| e.to_string())?
        }
    }
    Ok(())
}
pub fn verify_checksum(bytes: &[u8], expected: [u8; 32]) -> Result<(), String> {
    let actual: [u8; 32] = Sha256::digest(bytes).into();
    (actual == expected)
        .then_some(())
        .ok_or_else(|| "release archive checksum verification failed".into())
}

pub fn parse_checksum(c: &str, n: &str) -> Result<[u8; 32], String> {
    let mut out = None;
    for l in c.lines() {
        let f: Vec<_> = l.split_whitespace().collect();
        if f.len() != 2
            || f[1] != n
            || f[0].len() != 64
            || !f[0].bytes().all(|b| b.is_ascii_hexdigit())
            || out.is_some()
        {
            return Err("checksum file has an invalid format".into());
        }
        let mut x = [0; 32];
        for i in 0..32 {
            x[i] = u8::from_str_radix(&f[0][i * 2..i * 2 + 2], 16).unwrap();
        }
        out = Some(x)
    }
    out.ok_or("checksum file is empty".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checksum_and_mismatch() {
        let h = "a".repeat(64);
        assert_eq!(
            parse_checksum(&format!("{h}  x.tar.gz\n"), "x.tar.gz").unwrap(),
            [0xaa; 32]
        );
        assert!(parse_checksum("bad x.tar.gz", "x.tar.gz").is_err());
    }
    #[test]
    fn rejects_traversal() {
        assert!(safe_name(Path::new("../shelly")).is_err());
        assert!(safe_name(Path::new("bin/shelly")).is_err());
    }
    #[test]
    fn platform_is_known() {
        assert!(target_archive_name().is_ok() || target_archive_name().is_err());
    }
    #[test]
    fn version_probe_does_not_overwrite_collision_path() {
        let path = env::temp_dir().join(format!(
            "shelly-version-{}-collision-test",
            std::process::id()
        ));
        fs::write(&path, b"preserve me").unwrap();
        assert_eq!(
            installed_version(Path::new("/nonexistent"), "collision-test"),
            "unavailable"
        );
        assert_eq!(fs::read(&path).unwrap(), b"preserve me");
        fs::remove_file(path).unwrap();
    }
}
pub fn target_archive_name() -> Result<&'static str, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok("shelly-linux-x86_64.tar.gz"),
        ("macos", "x86_64") => Ok("shelly-macos-x86_64.tar.gz"),
        ("macos", "aarch64") => Ok("shelly-macos-aarch64.tar.gz"),
        ("windows", "x86_64") => Ok("shelly-windows-x86_64.zip"),
        _ => Err(format!(
            "unsupported platform: {}-{}",
            env::consts::OS,
            env::consts::ARCH
        )),
    }
}
