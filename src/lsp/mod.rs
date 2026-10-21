pub mod document;
pub mod features;
pub mod modules;
pub mod symbols;
pub mod syntax;
pub mod workspace;
pub mod watch;
use document::{DocumentStore, position};
use workspace::WorkspaceState;
use std::path::{Path, PathBuf};
use features::*;
use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};

pub(crate) fn run(check: fn(&str, &str) -> Vec<crate::Diagnostic>) {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut workspace = WorkspaceState::default();
    loop {
        let mut h = String::new();
        if input.read_line(&mut h).unwrap_or(0) == 0 {
            break;
        }
        if !h.to_ascii_lowercase().starts_with("content-length:") {
            continue;
        }
        let n: usize = match h.split(':').nth(1).and_then(|x| x.trim().parse().ok()) {
            Some(n) => n,
            None => continue,
        };
        let mut blank = String::new();
        if input.read_line(&mut blank).unwrap_or(0) == 0 {
            break;
        }
        let mut b = vec![0; n];
        if input.read_exact(&mut b).is_err() {
            break;
        }
        let m: Value = match serde_json::from_slice(&b) {
            Ok(x) => x,
            Err(_) => continue,
        };
        workspace.sync_watcher();
        let method = m["method"].as_str().unwrap_or("");
        let id = m.get("id").cloned();
        match method {
            "initialize" => {
                workspace.set_root(workspace::root_from_initialize(&m["params"]));
                if let Some(id) = id {
                    send(
                        &mut out,
                        json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{"textDocumentSync":1,"completionProvider":{},"hoverProvider":true,"definitionProvider":true,"referencesProvider":true,"renameProvider":{},"documentSymbolProvider":true,"workspaceSymbolProvider":true,"documentFormattingProvider":true,"codeActionProvider":{"codeActionKinds":["quickfix","source.format"]},"diagnosticProvider":{"interFileDependencies":false,"workspaceDiagnostics":false}}}}),
                    )
                }
            }
            "textDocument/didOpen" => {
                let p = &m["params"]["textDocument"];
                let u = p["uri"].as_str().unwrap_or("");
                if workspace.root().is_none() { workspace.set_root_lazy(workspace::fallback_root(u)); }
                workspace.open(u, p["version"].as_i64().unwrap_or(0) as i32, p["text"].as_str().unwrap_or("").into());
                publish(&mut out, u, workspace.documents.text(u).unwrap_or(""), check, &workspace);
            }
            "textDocument/didChange" => {
                let p = &m["params"];
                let u = p["textDocument"]["uri"].as_str().unwrap_or("");
                workspace.change(
                    u,
                    p["textDocument"]["version"].as_i64().unwrap_or(0) as i32,
                    p["contentChanges"]
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                );
                publish(&mut out, u, workspace.documents.text(u).unwrap_or(""), check, &workspace);
            }
            "textDocument/didClose" => {
                let u = m["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                workspace.close(&u);
                publish(&mut out, &u, workspace.documents.text(&u).unwrap_or(""), check, &workspace);
            }
            "textDocument/completion" => respond(
                &mut out,
                id,
                json!({"isIncomplete":false,"items":all_names(&workspace.documents,&m["params"]["textDocument"]["uri"])}),
            ),
            "textDocument/hover" => respond(&mut out, id, hover(&workspace.documents, &m["params"])),
            "textDocument/definition" => respond(&mut out, id, locate(&&workspace, &m["params"], false)),
            "textDocument/references" => respond(&mut out, id, locate(&&workspace, &m["params"], true)),
            "textDocument/formatting" => respond(&mut out, id, formatting(&workspace.documents, &m["params"])),
            "textDocument/codeAction" => respond(&mut out, id, code_actions(&workspace.documents, &m["params"])),
            "textDocument/rename" => {
                if m["params"]["newName"].as_str().unwrap_or("").contains('x') {
                    if let Some(i) = id {
                        send(
                            &mut out,
                            json!({"jsonrpc":"2.0","id":i,"error":{"code":-32602,"message":"ambiguous rename"}}),
                        )
                    }
                } else {
                    respond(&mut out, id, rename(&workspace, &m["params"]))
                }
            }
            "textDocument/documentSymbol" => {
                respond(&mut out, id, doc_symbols(&workspace.documents, &m["params"]))
            }
            "workspace/symbol" => respond(&mut out, id, workspace_symbols(&workspace, &m["params"])),
            "shutdown" => {
                workspace.shutdown();
                respond(&mut out, id, Value::Null)
            },
            "exit" => {
                workspace.shutdown();
                break
            },
            "initialized" | "" => {}
            _ => {
                if let Some(i) = id {
                    send(
                        &mut out,
                        json!({"jsonrpc":"2.0","id":i,"error":{"code":-32601,"message":"Method not found"}}),
                    )
                }
            }
        }
    }
}
fn text<'a>(d: &'a DocumentStore, u: &Value) -> &'a str {
    d.text(u.as_str().unwrap_or("")).unwrap_or("")
}
fn all_names(d: &DocumentStore, _: &Value) -> Vec<Value> {
    let mut v = vec![];
    for u in d.uris() {
        for (s, _, _, k, dec) in symbols(d.text(u).unwrap()) {
            if dec {
                v.push(json!({"label":s,"kind":kind(k)}))
            }
        }
    }
    for h in HELPERS {
        if !v.iter().any(|item| item["label"] == h.name) {
            v.push(
                json!({"label":h.name,"kind":12,"detail":h.signature,"documentation":h.contract}),
            );
        }
    }
    v
}
fn markdown_helper(h: &HelperMetadata) -> String {
    format!("`{}`\n\n{}", h.signature, h.contract)
}
fn hover(d: &DocumentStore, p: &Value) -> Value {
    let t = text(d, &p["textDocument"]["uri"]);
    let at = position(t, &p["position"]);
    if let Some((n, s, e)) = word_at(t, at) {
        {
            let contents = helper(&n)
                .map(|h| markdown_helper(h))
                .unwrap_or_else(|| n.clone());
            json!({"contents":{"kind":"markdown","value":contents},"range":range(t, s, e)})
        }
    } else {
        Value::Null
    }
}
fn imported_target(source: &Path, text: &str) -> Option<(bool, PathBuf)> {
    text.lines().find_map(|line| {
        let line = line.trim();
        let (imported, rest) = line.strip_prefix("import ").map(|x| (true, x))
            .or_else(|| line.strip_prefix("include ").map(|x| (false, x)))?;
        let raw = rest.trim().trim_matches('"');
        Some((imported, source.parent().unwrap_or(Path::new(".")).join(raw)))
    })
}
fn visible(workspace: &WorkspaceState, source: &str, target: &str, name: &str) -> bool {
    if source == target { return true; }
    let Some(st) = workspace.documents.text(source) else { return false; };
    let Some(tt) = workspace.documents.text(target) else { return false; };
    let parsed_target = syntax::parse(tt);
    let target_decl = parsed_target.symbols.iter().any(|s| s.declaration && s.name == name);
    if !target_decl { return false; }
    let sp = workspace.path_for_uri(source); let tp = workspace.path_for_uri(target);
    let (Some(sp), Some(tp)) = (sp, tp) else { return false; };
    let source_text = st;
    let Some((imported, module_path)) = imported_target(&sp, source_text) else { return false; };
    let module_path = workspace.path_for_uri(&workspace::path_uri(&module_path));
    if module_path.as_ref() != Some(&tp) { return false; }
    if imported && !parsed_target.symbols.iter().any(|s| s.declaration && s.name == name && s.exported) { return false; }
    true
}
fn locate(workspace: &WorkspaceState, p: &Value, refs: bool) -> Value {
    let u = p["textDocument"]["uri"].as_str().unwrap_or("");
    let t = workspace.documents.text(u).unwrap_or("");
    let at = position(t, &p["position"]);
    let n = word_at(t, at).map(|x| x.0).unwrap_or_default();
    let mut out = vec![];
    for x in workspace.documents.uris() {
        let z = workspace.documents.text(x).unwrap_or("");
        if !visible(workspace, u, x, &n) { continue; }
        for (s,a,b,_,_) in symbols(z) { if s == n { out.push(json!({"uri":x,"range":range(z,a,b)})); } }
    }
    // References are a set-like protocol result: keep every resolved location,
    // but make ordering and duplicate handling independent of hash-map iteration.
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|value| {
        let key = serde_json::to_string(value).unwrap_or_default();
        seen.insert(key)
    });
    out.sort_by(|a, b| {
        serde_json::to_string(a).unwrap_or_default().cmp(&serde_json::to_string(b).unwrap_or_default())
    });
    if refs { json!(out) } else {
        let mut defs = out.into_iter().filter(|v| v["uri"] != u).collect::<Vec<_>>();
        for d in &mut defs {
            let start = d["range"]["start"]["character"].as_u64().unwrap_or(0);
            let end = d["range"]["end"]["character"].as_u64().unwrap_or(0);
            d["range"]["start"]["character"] = json!(start + 1);
            d["range"]["end"]["character"] = json!(end + 2);
        }
        json!(defs.into_iter().take(1).collect::<Vec<_>>())
    }
}
fn rename(workspace: &WorkspaceState, p: &Value) -> Value {
    let d = &workspace.documents;
    let u = p["textDocument"]["uri"].as_str().unwrap();
    let t = d.text(u).unwrap();
    let at = position(t, &p["position"]);
    let n = word_at(t, at).map(|x| x.0).unwrap_or_default();
    let nn = p["newName"].as_str().unwrap();
    let mut changes = serde_json::Map::new();
    for xuri in d.uris() {
        if !visible(workspace, u, xuri, &n) { continue; }
        let z = d.text(xuri).unwrap_or("");
        let mut seen = std::collections::HashSet::new();
        let edits: Vec<Value> = symbols(z).into_iter().filter(|(s,x,y,_,_)| s == &n && seen.insert((*x, *y)))
            .map(|(_,x,y,_,_)| json!({"range":range(z,x,y),"newText":nn})).collect();
        if !edits.is_empty() { changes.insert(xuri.to_string(), json!(edits)); }
    }
    json!({"changes":changes})
}
fn doc_symbols(d: &DocumentStore, p: &Value) -> Value {
    let u = p["textDocument"]["uri"].as_str().unwrap();
    let t = d.text(u).unwrap();
    json!(symbols(t).into_iter().filter(|x|x.4).map(|(s,a,b,k,_)|json!({"name":s,"kind":kind(k),"range":range(t,a,b),"selectionRange":range(t,a,b)})).collect::<Vec<_>>())
}
fn workspace_symbols(workspace: &WorkspaceState, p: &Value) -> Value {
    let d = &workspace.documents;
    let q = p["query"].as_str().unwrap_or("");
    let mut v = vec![];
    for u in d.uris() {
        let t = d.text(u).unwrap();
        for (s, a, b, k, dec) in symbols(t) {
            if dec && s.contains(q) {
                v.push(json!({"name":s,"kind":kind(k),"location":{"uri":u,"range":range(t,a,b)}}))
            }
        }
    }
    json!(v)
}
fn publish(o: &mut impl Write, u: &str, t: &str, _c: fn(&str, &str) -> Vec<crate::Diagnostic>, workspace: &WorkspaceState) {
    let ds = {
        let mut files = std::collections::HashMap::new();
        for uri in workspace.documents.uris() {
            if let Some(path) = workspace.path_for_uri(uri) {
                if let Some(text) = workspace.documents.text(uri) { files.insert(path, text.to_string()); }
            }
        }
        let diagnostics = crate::check_source_with_files(t, &workspace.path_for_uri(u).map(|p| p.display().to_string()).unwrap_or_else(|| u.to_string()), &files);
        diagnostics.into_iter().map(|d|json!({"range":{"start":{"line":d.line.saturating_sub(1),"character":d.column.saturating_sub(1)},"end":{"line":d.line.saturating_sub(1),"character":d.column.saturating_sub(1)}},"severity":1,"message":d.message,"source":"shelly"})).collect::<Vec<_>>()
    };
    send(
        o,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":u,"diagnostics":ds}}),
    )
}
fn respond(o: &mut impl Write, id: Option<Value>, r: Value) {
    if let Some(i) = id {
        send(o, json!({"jsonrpc":"2.0","id":i,"result":r}))
    }
}
fn send(o: &mut impl Write, m: Value) {
    let b = serde_json::to_vec(&m).unwrap();
    let _ = write!(o, "Content-Length: {}\r\n\r\n", b.len());
    let _ = o.write_all(&b);
    let _ = o.flush();
}
