use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use tar::{Archive, Builder, EntryType};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const MAX_ENTRIES: usize = 100_000;
const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_TOTAL: u64 = 1024 * 1024 * 1024;
const MAX_RATIO: u64 = 1000;

fn fmt(p: &Path) -> Result<&'static str, String> {
    match p.extension().and_then(|x| x.to_str()) {
        Some("zip") => Ok("zip"),
        Some("tgz") => Ok("tar.gz"),
        Some("gz")
            if p.file_name()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.ends_with(".tar.gz")) =>
        {
            Ok("tar.gz")
        }
        _ => Err("archive format must be .zip, .tar.gz, or .tgz".into()),
    }
}
fn safe(n: &str) -> Result<String, String> {
    let p = Path::new(n);
    if n.is_empty() || n.contains('\\') || p.is_absolute() {
        return Err(format!("unsafe archive member: {n}"));
    }
    let mut parts = Vec::new();
    for component in p.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(format!("unsafe archive member: {n}"));
                }
            }
            _ => return Err(format!("unsafe archive member: {n}")),
        }
    }
    if parts.is_empty() {
        return Err(format!("unsafe archive member: {n}"));
    }
    Ok(parts.join("/"))
}
fn fields(op: &str, format: &str, entries: usize, bytes: u64, path: &str) -> crate::value::Value {
    let mut d = BTreeMap::new();
    for (k, v) in [
        ("operation", op),
        ("format", format),
        ("entries", &entries.to_string()),
        ("bytes", &bytes.to_string()),
    ] {
        d.insert(k.into(), crate::value::Value::Scalar(v.into()));
    }
    d.insert(
        if op == "extract" {
            "destination"
        } else {
            "archive"
        }
        .into(),
        crate::value::Value::Scalar(path.into()),
    );
    crate::value::Value::Dict(d)
}
fn files(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    fn walk(r: &Path, p: &Path, o: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
        let mut es = fs::read_dir(p)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        es.sort_by_key(|e| e.file_name());
        for e in es {
            let q = e.path();
            let m = fs::symlink_metadata(&q).map_err(|e| e.to_string())?;
            if m.file_type().is_symlink() {
                return Err("source symlink rejected".into());
            }
            if m.is_dir() {
                walk(r, &q, o)?
            } else if m.is_file() {
                o.push((
                    q.strip_prefix(r)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    q,
                ))
            } else {
                return Err("source special file rejected".into());
            }
        }
        Ok(())
    }
    let mut o = vec![];
    walk(root, root, &mut o)?;
    Ok(o)
}
fn validate_names(names: &[String]) -> Result<Vec<String>, String> {
    let normalized = names
        .iter()
        .map(|n| safe(n))
        .collect::<Result<Vec<_>, _>>()?;
    let set: HashSet<_> = normalized.iter().collect();
    if set.len() != normalized.len() {
        return Err("duplicate archive member".into());
    }
    for n in &normalized {
        let mut p = Path::new(n);
        while let Some(parent) = p.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }
            if normalized.iter().any(|x| x == &parent.to_string_lossy()) {
                return Err("archive path collision".into());
            }
            p = parent;
        }
    }
    Ok(normalized)
}

pub(crate) fn create(src: &Path, out: &Path) -> Result<crate::value::Value, String> {
    let f = fmt(out)?;
    if !src.is_dir() {
        return Err("source must be a directory".into());
    }
    let fs = files(src)?;
    if fs.len() > MAX_ENTRIES {
        return Err("entry limit exceeded".into());
    }
    let total: u64 = fs
        .iter()
        .map(|(_, p)| fs::metadata(p).map(|m| m.len()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    if total > MAX_TOTAL {
        return Err("total size limit exceeded".into());
    }
    let tmp = out.with_extension("archive-tmp");
    let result = (|| {
        let staging = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        if f == "zip" {
            let mut z = ZipWriter::new(staging);
            for (n, p) in &fs {
                z.start_file(n, SimpleFileOptions::default())
                    .map_err(|e| e.to_string())?;
                z.write_all(&fs::read(p).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?
            }
            z.finish().map_err(|e| e.to_string())?;
        } else {
            let mut b = Builder::new(GzEncoder::new(staging, Compression::default()));
            for (n, p) in &fs {
                b.append_path_with_name(p, n).map_err(|e| e.to_string())?
            }
            b.into_inner()
                .map_err(|e| e.to_string())?
                .finish()
                .map_err(|e| e.to_string())?;
        }
        // Hard-link publication is create-new: it never replaces an existing output.
        fs::hard_link(&tmp, out).map_err(|e| e.to_string())?;
        fs::remove_file(&tmp).map_err(|e| e.to_string())?;
        Ok(fields("create", f, fs.len(), total, &out.to_string_lossy()))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
pub(crate) fn checksum(p: &Path) -> Result<crate::value::Value, String> {
    let mut f = File::open(p).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut b = [0; 65536];
    loop {
        let n = f.read(&mut b).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&b[..n])
    }
    Ok(crate::value::Value::Scalar(format!("{:x}", h.finalize())))
}
pub(crate) fn extract(a: &Path, d: &Path) -> Result<crate::value::Value, String> {
    if d.exists() {
        return Err("destination already exists".into());
    }
    let f = fmt(a)?;
    let mut names = Vec::new();
    let mut sizes = Vec::new();
    if f == "zip" {
        let mut z = ZipArchive::new(File::open(a).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for i in 0..z.len() {
            let e = z.by_index(i).map_err(|e| e.to_string())?;
            if e.is_dir() || e.unix_mode().is_some_and(|m| m & 0o170000 != 0o100000) {
                return Err("non-regular archive member".into());
            }
            names.push(e.name().map_err(|e| e.to_string())?.to_string());
            sizes.push((e.size(), e.compressed_size()));
        }
    } else {
        let mut ar = Archive::new(GzDecoder::new(File::open(a).map_err(|e| e.to_string())?));
        for e in ar.entries().map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            if e.header().entry_type() != EntryType::Regular {
                return Err("non-regular archive member".into());
            }
            names.push(
                e.path()
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .into_owned(),
            );
            sizes.push((e.size(), e.size()));
        }
    }
    if names.len() > MAX_ENTRIES {
        return Err("entry limit exceeded".into());
    }
    let names = validate_names(&names)?;
    let total: u64 = sizes.iter().map(|x| x.0).sum();
    if total > MAX_TOTAL
        || sizes
            .iter()
            .any(|x| x.0 > MAX_FILE || x.1.saturating_mul(MAX_RATIO) < x.0)
    {
        return Err("archive limits exceeded".into());
    }
    let tmp = d.with_extension("extract-tmp");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir(&tmp).map_err(|e| e.to_string())?;
    let result = (|| {
        if f == "zip" {
            let mut z = ZipArchive::new(File::open(a).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            for (i, n) in names.iter().enumerate() {
                let mut e = z.by_index(i).map_err(|e| e.to_string())?;
                let q = tmp.join(n);
                fs::create_dir_all(q.parent().unwrap()).map_err(|e| e.to_string())?;
                std::io::copy(&mut e, &mut File::create(q).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let mut ar = Archive::new(GzDecoder::new(File::open(a).map_err(|e| e.to_string())?));
            for (i, n) in names.iter().enumerate() {
                let mut e = ar
                    .entries()
                    .map_err(|e| e.to_string())?
                    .nth(i)
                    .ok_or("archive changed during extraction")?
                    .map_err(|e| e.to_string())?;
                let q = tmp.join(n);
                fs::create_dir_all(q.parent().unwrap()).map_err(|e| e.to_string())?;
                e.unpack(q).map_err(|e| e.to_string())?;
            }
        }
        fs::rename(&tmp, d).map_err(|e| e.to_string())?;
        Ok(fields(
            "extract",
            f,
            names.len(),
            total,
            &d.to_string_lossy(),
        ))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&tmp);
    }
    result
}
pub(crate) fn check(a: &Path) -> Result<crate::value::Value, String> {
    let tmp = a.with_extension("check-tmp");
    let r = extract(a, &tmp);
    let _ = fs::remove_dir_all(&tmp);
    r.map(|mut v| {
        if let crate::value::Value::Dict(ref mut d) = v {
            d.insert(
                "operation".into(),
                crate::value::Value::Scalar("check".into()),
            );
            d.remove("destination");
            d.insert(
                "archive".into(),
                crate::value::Value::Scalar(a.to_string_lossy().into()),
            );
        }
        v
    })
}
