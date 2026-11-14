pub mod document;
pub mod features;
pub mod modules;
pub mod symbols;
pub mod syntax;
pub mod watch;
pub mod workspace;
use document::{DocumentStore, position};
use features::*;
use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use workspace::WorkspaceState;

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
                        json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{"textDocumentSync":1,"completionProvider":{},"hoverProvider":true,"definitionProvider":true,"referencesProvider":true,"renameProvider":{},"documentSymbolProvider":true,"workspaceSymbolProvider":true,"documentFormattingProvider":true,"codeActionProvider":{"codeActionKinds":["quickfix","source.format"]}}}}),
                    )
                }
            }
            "textDocument/didOpen" => {
                let p = &m["params"]["textDocument"];
                let u = p["uri"].as_str().unwrap_or("");
                if workspace.root().is_none() {
                    workspace.set_root_lazy(workspace::fallback_root(u));
                }
                workspace.open(
                    u,
                    p["version"].as_i64().unwrap_or(0) as i32,
                    p["text"].as_str().unwrap_or("").into(),
                );
                publish(
                    &mut out,
                    u,
                    workspace.documents.text(u).unwrap_or(""),
                    check,
                    &workspace,
                );
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
                publish(
                    &mut out,
                    u,
                    workspace.documents.text(u).unwrap_or(""),
                    check,
                    &workspace,
                );
            }
            "textDocument/didClose" => {
                let u = m["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                workspace.close(&u);
                publish(
                    &mut out,
                    &u,
                    workspace.documents.text(&u).unwrap_or(""),
                    check,
                    &workspace,
                );
            }
            "textDocument/completion" => respond(
                &mut out,
                id,
                json!({"isIncomplete":false,"items":completion(&workspace,&m["params"])}),
            ),
            "textDocument/hover" => {
                respond(&mut out, id, hover(&workspace.documents, &m["params"]))
            }
            "textDocument/definition" => {
                respond(&mut out, id, locate(&workspace, &m["params"], false))
            }
            "textDocument/references" => {
                respond(&mut out, id, locate(&workspace, &m["params"], true))
            }
            "textDocument/formatting" => {
                respond(&mut out, id, formatting(&workspace.documents, &m["params"]))
            }
            "textDocument/codeAction" => respond(
                &mut out,
                id,
                code_actions(&workspace.documents, &m["params"]),
            ),
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
            "textDocument/documentSymbol" => respond(
                &mut out,
                id,
                doc_symbols(&workspace.documents, &m["params"]),
            ),
            "workspace/symbol" => {
                respond(&mut out, id, workspace_symbols(&workspace, &m["params"]))
            }
            "shutdown" => {
                workspace.shutdown();
                respond(&mut out, id, Value::Null)
            }
            "exit" => {
                workspace.shutdown();
                break;
            }
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
const COMMAND_COMPLETIONS: &[(&str, &str)] = &[
    ("progress", "progress \"LABEL\" -> $result ... end"),
    ("menu", "menu \"TITLE\" -> $choice ... end"),
    ("try", "try ... catch ... end"),
    ("retry", "retry COUNT ... end"),
    ("watch", "watch PROGRAM ARGS -> $line ... end"),
    ("parallel", "parallel limit COUNT -> $results ... end"),
    ("pipeline", "pipeline -> $result ... end"),
    ("clear", "clear"),
];

const RESULT_PROPERTIES: &[(&str, &str)] = &[
    ("status", "task outcome"),
    ("durationMs", "elapsed milliseconds"),
];
const ERROR_PROPERTIES: &[(&str, &str)] = &[
    ("kind", "error category"),
    ("message", "error message"),
    ("program", "failed program"),
    ("args", "program arguments"),
    ("code", "exit code"),
    ("stdout", "captured stdout"),
    ("stderr", "captured stderr"),
];

const STYLE_DESCRIPTORS: &[&str] = &[
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
    "bold",
    "dim",
    "italic",
    "underline",
    "blink",
    "reverse",
    "strikethrough",
    "reset",
];

fn completion(workspace: &WorkspaceState, params: &Value) -> Vec<Value> {
    let d = &workspace.documents;
    let uri = &params["textDocument"]["uri"];
    let source = text(d, uri);
    let offset = position(source, &params["position"]);
    let before = &source[..offset.min(source.len())];
    if before.rfind("formatDate(").is_some_and(|start| {
        let args = &before[start + "formatDate(".len()..];
        !args.contains(')') && args.matches(',').count() == 1
    }) {
        return ["YYYY", "MM", "DD", "HH", "mm", "ss"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":12,"detail":"date format token"}))
            .collect();
    }
    if before
        .rfind("import \"")
        .or_else(|| before.rfind("include \""))
        .is_some_and(|start| before[start..].matches('"').count() == 1)
    {
        let base = workspace
            .path_for_uri(uri.as_str().unwrap_or(""))
            .and_then(|path| path.parent().map(Path::to_path_buf));
        return d
            .uris()
            .filter_map(|candidate| workspace.path_for_uri(candidate))
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "shelly" || ext == "shl")
            })
            .filter_map(|path| match &base {
                Some(base) => path.strip_prefix(base).ok().map(Path::to_path_buf),
                None => path.file_name().map(PathBuf::from),
            })
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .map(|label| json!({"label":label,"kind":17,"detail":"Shelly module"}))
            .collect();
    }
    if before.trim_end().ends_with("http") {
        return ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":13,"detail":"HTTP method"}))
            .collect();
    }
    let line = before.lines().last().unwrap_or("").trim();
    if line.is_empty()
        && before
            .rfind("pipeline ")
            .is_some_and(|p| before.rfind("\nend").is_none_or(|end| p > end))
    {
        return ["command", "end"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":14,"detail":"pipeline directive"}))
            .collect();
    }
    if line.is_empty()
        && before
            .rfind("parallel ")
            .is_some_and(|p| before.rfind("\nend").is_none_or(|end| p > end))
    {
        return ["task", "end"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":14,"detail":"parallel directive"}))
            .collect();
    }
    if line.is_empty()
        && before
            .rfind("menu ")
            .is_some_and(|menu| before.rfind("\nend").is_none_or(|end| menu > end))
    {
        return ["subtitle", "keys", "default", "option", "hint", "end"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":14,"detail":"menu directive"}))
            .collect();
    }
    if before.trim_end().starts_with("http ") && before.matches('"').count() >= 2 {
        return ["headers", "body", "timeout", "maxBytes", "-> $response"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":13,"detail":"HTTP option"}))
            .collect();
    }
    if before.trim_end().ends_with("keys") {
        return ["numbers", "letters lower", "letters upper"]
            .into_iter()
            .map(|label| json!({"label":label,"kind":13,"detail":"menu key mode"}))
            .collect();
    }
    if before
        .split_whitespace()
        .last()
        .is_some_and(|token| token.starts_with("$result."))
    {
        return RESULT_PROPERTIES
            .iter()
            .map(|(label, detail)| json!({"label":label,"kind":10,"detail":detail}))
            .collect();
    }
    if before
        .split_whitespace()
        .last()
        .is_some_and(|token| token.starts_with("$error."))
    {
        return ERROR_PROPERTIES
            .iter()
            .map(|(label, detail)| json!({"label":label,"kind":10,"detail":detail}))
            .collect();
    }
    if before.rfind("style(").is_some_and(|start| {
        let args = &before[start + "style(".len()..];
        !args.contains(')') && !args.contains(',')
    }) {
        let mut items: Vec<Value> = STYLE_DESCRIPTORS
            .iter()
            .map(|label| json!({"label":label,"kind":12,"detail":"Shelly terminal style"}))
            .collect();
        items.extend(imported_palette_constants(d, source));
        return items;
    }
    all_names(d, source, offset)
}

fn imported_palette_constants(d: &DocumentStore, source: &str) -> Vec<Value> {
    let imports: Vec<&str> = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("import "))
        .filter_map(|path| {
            path.trim()
                .strip_prefix('"')
                .and_then(|p| p.split_once('"').map(|(p, _)| p))
        })
        .collect();
    d.uris()
        .filter(|uri| imports.iter().any(|path| uri.ends_with(path)))
        .flat_map(|uri| syntax::parse(d.text(uri).unwrap_or("")).symbols)
        .filter(|symbol| {
            symbol.declaration && symbol.exported && symbol.kind == syntax::SymbolKind::Constant
        })
        .map(|symbol| json!({"label":format!("${}", symbol.name),"kind":kind(symbol.kind)}))
        .collect()
}

fn all_names(d: &DocumentStore, source: &str, offset: usize) -> Vec<Value> {
    let mut v = vec![];
    for symbol in
        symbols::SymbolIndex::from_parse(syntax::parse(source)).visible_declarations_at(offset)
    {
        let label = match symbol.kind {
            symbols::SymbolKind::Constant => format!("${}", symbol.name),
            _ => symbol.name.clone(),
        };
        v.push(json!({"label":label,"kind":6}));
        if symbol.kind == symbols::SymbolKind::Variable {
            v.push(json!({"label":format!("${}", symbol.name),"kind":6}));
        }
    }
    v.extend(imported_palette_constants(d, source));
    for h in HELPERS {
        if !v.iter().any(|item| item["label"] == h.name) {
            v.push(
                json!({"label":h.name,"kind":12,"detail":h.signature,"documentation":h.contract}),
            );
        }
    }
    for (label, detail) in COMMAND_COMPLETIONS {
        if !v.iter().any(|item| item["label"] == *label) {
            v.push(json!({"label":label,"kind":14,"detail":detail}));
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
            let contents = helper(&n).map(markdown_helper).unwrap_or_else(|| n.clone());
            json!({"contents":{"kind":"markdown","value":contents},"range":range(t, s, e)})
        }
    } else {
        Value::Null
    }
}
fn imported_target(source: &Path, text: &str) -> Option<(bool, PathBuf)> {
    text.lines().find_map(|line| {
        let line = line.trim();
        let (imported, rest) = line
            .strip_prefix("import ")
            .map(|x| (true, x))
            .or_else(|| line.strip_prefix("include ").map(|x| (false, x)))?;
        let raw = rest.trim().trim_matches('"');
        Some((
            imported,
            source.parent().unwrap_or(Path::new(".")).join(raw),
        ))
    })
}
fn visible(workspace: &WorkspaceState, source: &str, target: &str, name: &str) -> bool {
    if source == target {
        return true;
    }
    let Some(st) = workspace.documents.text(source) else {
        return false;
    };
    let Some(tt) = workspace.documents.text(target) else {
        return false;
    };
    let parsed_target = syntax::parse(tt);
    let target_decl = parsed_target
        .symbols
        .iter()
        .any(|s| s.declaration && s.name == name);
    if !target_decl {
        return false;
    }
    let sp = workspace.path_for_uri(source);
    let tp = workspace.path_for_uri(target);
    let (Some(sp), Some(tp)) = (sp, tp) else {
        return false;
    };
    let source_text = st;
    let Some((imported, module_path)) = imported_target(&sp, source_text) else {
        return false;
    };
    let module_path = workspace.path_for_uri(&workspace::path_uri(&module_path));
    if module_path.as_ref() != Some(&tp) {
        return false;
    }
    if imported
        && !parsed_target
            .symbols
            .iter()
            .any(|s| s.declaration && s.name == name && s.exported)
    {
        return false;
    }
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
        if !visible(workspace, u, x, &n) {
            continue;
        }
        for (s, a, b, _, _) in symbols(z) {
            if s == n {
                out.push(json!({"uri":x,"range":range(z,a,b)}));
            }
        }
    }
    // References are a set-like protocol result: keep every resolved location,
    // but make ordering and duplicate handling independent of hash-map iteration.
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|value| {
        let key = serde_json::to_string(value).unwrap_or_default();
        seen.insert(key)
    });
    out.sort_by(|a, b| {
        serde_json::to_string(a)
            .unwrap_or_default()
            .cmp(&serde_json::to_string(b).unwrap_or_default())
    });
    if refs {
        json!(out)
    } else {
        let mut defs = out
            .into_iter()
            .filter(|v| v["uri"] != u)
            .collect::<Vec<_>>();
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
        if !visible(workspace, u, xuri, &n) {
            continue;
        }
        let z = d.text(xuri).unwrap_or("");
        let mut seen = std::collections::HashSet::new();
        let edits: Vec<Value> = symbols(z)
            .into_iter()
            .filter(|(s, x, y, _, _)| s == &n && seen.insert((*x, *y)))
            .map(|(_, x, y, _, _)| json!({"range":range(z,x,y),"newText":nn}))
            .collect();
        if !edits.is_empty() {
            changes.insert(xuri.to_string(), json!(edits));
        }
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
fn publish(
    o: &mut impl Write,
    u: &str,
    t: &str,
    _c: fn(&str, &str) -> Vec<crate::Diagnostic>,
    workspace: &WorkspaceState,
) {
    let ds = {
        let mut files = std::collections::HashMap::new();
        for uri in workspace.documents.uris() {
            if let Some(path) = workspace.path_for_uri(uri)
                && let Some(text) = workspace.documents.text(uri)
            {
                files.insert(path, text.to_string());
            }
        }
        let diagnostics = crate::check_source_with_files(
            t,
            &workspace
                .path_for_uri(u)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| u.to_string()),
            &files,
        );
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
