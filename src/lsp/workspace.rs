use crate::lsp::{document::DocumentStore, watch::{Event, Watcher}};
use std::{fs, path::{Path, PathBuf}};

#[derive(Default)]
pub struct WorkspaceState {
    pub documents: DocumentStore,
    root: Option<PathBuf>,
    watcher: Option<Watcher>,
    #[cfg(test)]
    test_reconcile_scan_error: bool,
    #[cfg(test)]
    test_reconcile_scan_failed: bool,
}
impl WorkspaceState {
    pub fn set_root(&mut self, root: Option<PathBuf>) {
        self.set_root_inner(root, true);
    }
    pub fn shutdown(&mut self) {
        self.watcher = None;
    }
    /// Set a bounded fallback root without eagerly indexing its contents.
    pub fn set_root_lazy(&mut self, root: Option<PathBuf>) {
        self.set_root_inner(root, false);
    }
    fn set_root_inner(&mut self, root: Option<PathBuf>, load_disk: bool) {
        self.documents.clear();
        self.root = root.and_then(|p| fs::canonicalize(p).ok());
        // A fallback root for an unsaved document may be `/`; never recursively
        // watch that broad root, since it would compete with request handling.
        self.watcher = self.root.clone().filter(|p| load_disk || p != Path::new("/")).map(Watcher::start);
        if load_disk { self.load_disk(); }
    }
    pub fn root(&self) -> Option<&Path> { self.root.as_deref() }
    pub fn open(&mut self, uri: &str, version: i32, text: String) { self.documents.open(uri, version, text); }
    pub fn change(&mut self, uri: &str, version: i32, changes: &[serde_json::Value]) { self.documents.change(uri, version, changes); }
    pub fn close(&mut self, uri: &str) {
        self.documents.close(uri);
        if let Some(text) = self.disk_text(uri) { self.documents.set_disk(uri, text); }
    }
    /// Apply all filesystem changes observed since the last request. The bounded
    /// watcher is deliberately polled here so request handling never blocks.
    #[cfg(test)]
    fn trigger_watcher_overflow(&self) { if let Some(watcher) = &self.watcher { watcher.trigger_overflow(); } }
    #[cfg(test)]
    fn inject_reconcile_scan_error(&mut self) { self.test_reconcile_scan_error = true; }
    #[cfg(test)]
    fn reconcile_scan_failed(&self) -> bool { self.test_reconcile_scan_failed }
    pub fn sync_watcher(&mut self) {
        let overflowed = self.watcher.as_ref().map(|w| w.overflowed()).unwrap_or(false);
        let events = self.watcher.as_ref().map(|w| w.drain()).unwrap_or_default();
        for event in events {
            match event {
                Event::Upsert(path) => if let Ok(text) = fs::read_to_string(&path) {
                    self.documents.set_disk(&path_uri(&path), text);
                },
                Event::Remove(path) => self.documents.remove_disk(&path_uri(&path)),
            }
        }
        if overflowed { self.reconcile_disk(); }
    }
    fn reconcile_disk(&mut self) {
        #[cfg(test)]
        if self.test_reconcile_scan_error {
            self.test_reconcile_scan_error = false;
            self.test_reconcile_scan_failed = true;
            return;
        }
        let Some(root) = self.root.clone() else { return };
        let mut current = std::collections::BTreeSet::new();
        let mut pending = vec![root];
        while let Some(dir) = pending.pop() {
            let Ok(entries) = fs::read_dir(dir) else { continue };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() { pending.push(p); }
                else if matches!(p.extension().and_then(|x| x.to_str()), Some("shelly"|"shl")) {
                    if let Ok(p) = fs::canonicalize(p) { current.insert(path_uri(&p)); }
                }
            }
        }
        let existing: Vec<String> = self.documents.uris().filter(|u| self.path_for_uri(u).is_some()).map(str::to_owned).collect();
        for uri in existing { if !current.contains(&uri) { self.documents.remove_disk(&uri); } }
        self.load_disk();
    }
    fn disk_text(&self, uri: &str) -> Option<String> {
        let path = self.path_for_uri(uri)?;
        fs::read_to_string(path).ok()
    }
    /// Resolve a URI lexically under the workspace, canonicalizing only targets
    /// that already exist on disk. Unsaved overlay paths may not exist yet.
    pub fn path_for_uri(&self, uri: &str) -> Option<PathBuf> {
        let path = uri_path(uri)?;
        let root = self.root.as_ref()?;
        let lexical = lexical_path(&path);
        if !lexical.starts_with(root) { return None; }
        Some(fs::canonicalize(&lexical).unwrap_or(lexical))
    }
    fn load_disk(&mut self) {
        let Some(root) = self.root.clone() else { return };
        let mut pending = vec![root.clone()];
        while let Some(dir) = pending.pop() {
            let Ok(entries) = fs::read_dir(dir) else { continue };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() { pending.push(p); continue; }
                if !p.extension().and_then(|x| x.to_str()).map(|x| x == "shelly" || x == "shl").unwrap_or(false) { continue; }
                let Ok(canonical) = fs::canonicalize(&p) else { continue };
                if !canonical.starts_with(&root) { continue; }
                if let Ok(text) = fs::read_to_string(&canonical) { self.documents.set_disk(&path_uri(&canonical), text); }
            }
        }
    }
}
pub fn path_uri(path: &Path) -> String {
    let mut out = String::from("file://");
    for b in path.to_string_lossy().as_bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(b) { out.push(*b as char); }
        else { out.push_str(&format!("%{:02X}", b)); }
    }
    out
}
pub fn read_uri(uri: &str) -> Option<String> { uri_path(uri).and_then(|p| fs::read_to_string(p).ok()) }
pub fn root_from_initialize(params: &serde_json::Value) -> Option<PathBuf> {
    params["workspaceFolders"].as_array().and_then(|x| x.first()).and_then(|x| x["uri"].as_str()).and_then(uri_path)
        .or_else(|| params["rootUri"].as_str().and_then(uri_path))
}
pub fn uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let (authority, path) = if rest.starts_with('/') { ("", rest) } else { rest.split_once('/').unwrap_or((rest, "")) };
    if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") { return None; }
    let decoded = percent_decode(path)?;
    if decoded.contains('\0') { return None; }
    Some(PathBuf::from(decoded))
}
fn percent_decode(input: &str) -> Option<String> {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes(); let mut i = 0;
    while i < bytes.len() { if bytes[i] == b'%' { if i + 2 >= bytes.len() { return None; } let h = |b: u8| (b as char).to_digit(16); let n = h(bytes[i+1])? * 16 + h(bytes[i+2])?; out.push(n as u8); i += 3; } else { out.push(bytes[i]); i += 1; } }
    String::from_utf8(out).ok()
}
pub fn fallback_root(uri: &str) -> Option<PathBuf> { uri_path(uri).and_then(|p| p.parent().map(Path::to_path_buf)) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let path = std::env::temp_dir().join(format!("shelly-workspace-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn overflow_rescan_reconciles_real_deletions() {
        let root = temp_root();
        for i in 0..8 { fs::write(root.join(format!("{i}.shl")), i.to_string()).unwrap(); }
        let removed = root.join("0.shl");
        let uri = path_uri(&removed);
        let mut ws = WorkspaceState::default();
        ws.set_root(Some(root.clone()));
        fs::remove_file(&removed).unwrap();
        ws.trigger_watcher_overflow();
        ws.sync_watcher();
        assert!(ws.documents.text(&uri).is_none());
    }

    #[test]
    fn scan_error_retains_known_document() {
        let root = temp_root();
        let nested = root.join("nested"); fs::create_dir(&nested).unwrap();
        let file = nested.join("known.shl"); fs::write(&file, "known").unwrap();
        let uri = path_uri(&file);
        let mut ws = WorkspaceState::default(); ws.set_root(Some(root.clone()));
        ws.inject_reconcile_scan_error();
        ws.trigger_watcher_overflow();
        ws.sync_watcher();
        assert!(ws.reconcile_scan_failed());
        assert_eq!(ws.documents.text(&uri), Some("known"));
    }
}

fn lexical_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => { out.pop(); }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
