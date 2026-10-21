use shelly::lsp::workspace::{fallback_root, root_from_initialize, uri_path, path_uri, WorkspaceState};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
fn temp_root() -> PathBuf { let p = std::env::temp_dir().join(format!("shelly-lsp-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())); fs::create_dir_all(&p).unwrap(); p }
use serde_json::json;

#[test]
fn initialize_prefers_workspace_folder_then_root_uri() {
    let p = json!({"workspaceFolders":[{"uri":"file:///workspace"}],"rootUri":"file:///root"});
    assert_eq!(root_from_initialize(&p).unwrap().to_str(), Some("/workspace"));
    assert_eq!(root_from_initialize(&json!({"rootUri":"file:///root"})).unwrap().to_str(), Some("/root"));
}

#[test]
fn standalone_fallback_uses_opened_file_parent() {
    assert_eq!(fallback_root("file:///tmp/project/main.shelly").unwrap().to_str(), Some("/tmp/project"));
}

#[test]
fn lazy_fallback_root_does_not_eagerly_scan_but_close_restores_disk() {
    let root = temp_root();
    let file = root.join("main.shelly");
    fs::write(&file, "disk").unwrap();
    let uri = path_uri(&file);
    let mut ws = WorkspaceState::default();
    ws.set_root_lazy(Some(root));
    assert_eq!(ws.documents.uris().count(), 0);
    ws.open(&uri, 1, "open".into());
    ws.close(&uri);
    assert_eq!(ws.documents.text(&uri), Some("disk"));
}

#[test]
fn loads_nested_disk_files_and_close_restores_disk() {
    let root = temp_root(); let nested = root.join("nested"); fs::create_dir(&nested).unwrap();
    let file = nested.join("x.shelly"); fs::write(&file, "disk").unwrap();
    let uri = path_uri(&file); let mut ws = WorkspaceState::default(); ws.set_root(Some(root.clone()));
    assert_eq!(ws.documents.text(&uri), Some("disk"));
    ws.open(&uri, 1, "open".into()); assert_eq!(ws.documents.text(&uri), Some("open"));
    ws.close(&uri); assert_eq!(ws.documents.text(&uri), Some("disk"));
}

#[test]
fn filters_extensions_and_rejects_unsafe_uris() {
    let root = temp_root(); fs::write(root.join("no.txt"), "x").unwrap();
    let mut ws = WorkspaceState::default(); ws.set_root(Some(root));
    assert_eq!(ws.documents.uris().count(), 0);
    assert_eq!(uri_path("file://remote/tmp/x"), None);
    assert_eq!(uri_path("file:///tmp/a%20b.shelly").unwrap(), PathBuf::from("/tmp/a b.shelly"));
}

#[test]
fn root_change_clears_previous_partition() {
    let a = temp_root(); let b = temp_root(); fs::write(a.join("a.shelly"), "a").unwrap(); fs::write(b.join("b.shelly"), "b").unwrap();
    let mut ws = WorkspaceState::default(); ws.set_root(Some(a)); ws.set_root(Some(b));
    assert_eq!(ws.documents.uris().filter(|u| u.contains("a.shelly")).count(), 0);
}
