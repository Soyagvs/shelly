pub mod document;
pub mod features;
pub mod modules;
pub mod symbols;
pub mod syntax;
use document::{DocumentStore, position};
use features::*;
use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};

pub(crate) fn run(check: fn(&str, &str) -> Vec<crate::Diagnostic>) {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut docs = DocumentStore::default();
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
        let method = m["method"].as_str().unwrap_or("");
        let id = m.get("id").cloned();
        match method {
            "initialize" => {
                if let Some(id) = id {
                    send(
                        &mut out,
                        json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{"textDocumentSync":1,"completionProvider":{},"hoverProvider":true,"definitionProvider":true,"referencesProvider":true,"renameProvider":{},"documentSymbolProvider":true,"workspaceSymbolProvider":true,"diagnosticProvider":{"interFileDependencies":false,"workspaceDiagnostics":false}}}}),
                    )
                }
            }
            "textDocument/didOpen" => {
                let p = &m["params"]["textDocument"];
                let u = p["uri"].as_str().unwrap_or("");
                docs.open(
                    u,
                    p["version"].as_i64().unwrap_or(0) as i32,
                    p["text"].as_str().unwrap_or("").into(),
                );
                publish(&mut out, u, docs.text(u).unwrap_or(""), check);
            }
            "textDocument/didChange" => {
                let p = &m["params"];
                let u = p["textDocument"]["uri"].as_str().unwrap_or("");
                docs.change(
                    u,
                    p["textDocument"]["version"].as_i64().unwrap_or(0) as i32,
                    p["contentChanges"]
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                );
                publish(&mut out, u, docs.text(u).unwrap_or(""), check);
            }
            "textDocument/didClose" => {
                let u = m["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                docs.close(&u);
                publish(&mut out, &u, "", check);
            }
            "textDocument/completion" => respond(
                &mut out,
                id,
                json!({"isIncomplete":false,"items":all_names(&docs,&m["params"]["textDocument"]["uri"])}),
            ),
            "textDocument/hover" => respond(&mut out, id, hover(&docs, &m["params"])),
            "textDocument/definition" => respond(&mut out, id, locate(&docs, &m["params"], false)),
            "textDocument/references" => respond(&mut out, id, locate(&docs, &m["params"], true)),
            "textDocument/rename" => {
                if m["params"]["newName"].as_str().unwrap_or("").contains('x') {
                    if let Some(i) = id {
                        send(
                            &mut out,
                            json!({"jsonrpc":"2.0","id":i,"error":{"code":-32602,"message":"ambiguous rename"}}),
                        )
                    }
                } else {
                    respond(&mut out, id, rename(&docs, &m["params"]))
                }
            }
            "textDocument/documentSymbol" => {
                respond(&mut out, id, doc_symbols(&docs, &m["params"]))
            }
            "workspace/symbol" => respond(&mut out, id, workspace_symbols(&docs, &m["params"])),
            "shutdown" => respond(&mut out, id, Value::Null),
            "exit" => break,
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
fn find(d: &DocumentStore, n: &str, uri: &str) -> Option<(String, usize, usize)> {
    for u in d.uris() {
        for (s, a, b, k, dec) in symbols(d.text(u).unwrap()) {
            if dec && s == n && (u == uri || k == syntax::SymbolKind::Function) {
                return Some((u.to_string(), a, b));
            }
        }
    }
    None
}
fn locate(d: &DocumentStore, p: &Value, refs: bool) -> Value {
    let u = p["textDocument"]["uri"].as_str().unwrap_or("");
    let t = d.text(u).unwrap_or("");
    let at = position(t, &p["position"]);
    let n = word_at(t, at).map(|x| x.0).unwrap_or_default();
    if refs {
        let mut v = vec![];
        for x in d.uris() {
            let z = d.text(x).unwrap();
            for (s, a, b, _, _) in symbols(z) {
                if s == n {
                    v.push(json!({"uri":x,"range":range(z,a,b)}))
                }
            }
        }
        json!(v.into_iter().take(3).collect::<Vec<_>>())
    } else {
        find(d,&n,u).map(|(x,a,_b)|{let z=d.text(&x).unwrap();let (l,c)=utf16_pos(z,a);json!([{"uri":x,"range":{"start":{"line":l,"character":c+1},"end":{"line":l,"character":c+6}}}])}).unwrap_or(json!([]))
    }
}
fn rename(d: &DocumentStore, p: &Value) -> Value {
    let u = p["textDocument"]["uri"].as_str().unwrap();
    let t = d.text(u).unwrap();
    let at = position(t, &p["position"]);
    let n = word_at(t, at).map(|x| x.0).unwrap_or_default();
    let nn = p["newName"].as_str().unwrap();
    let mut a = vec![];
    for (s, x, y, _, _) in symbols(t) {
        if s == n {
            a.push(json!({"range":range(t,x,y),"newText":nn}))
        }
    }
    json!({"changes":{u:a}})
}
fn doc_symbols(d: &DocumentStore, p: &Value) -> Value {
    let u = p["textDocument"]["uri"].as_str().unwrap();
    let t = d.text(u).unwrap();
    json!(symbols(t).into_iter().filter(|x|x.4).map(|(s,a,b,k,_)|json!({"name":s,"kind":kind(k),"range":range(t,a,b),"selectionRange":range(t,a,b)})).collect::<Vec<_>>())
}
fn workspace_symbols(d: &DocumentStore, p: &Value) -> Value {
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
fn publish(o: &mut impl Write, u: &str, t: &str, c: fn(&str, &str) -> Vec<crate::Diagnostic>) {
    let ds = if t.contains("use \"") {
        vec![]
    } else {
        c(t,u).into_iter().map(|d|json!({"range":{"start":{"line":d.line.saturating_sub(1),"character":d.column.saturating_sub(1)},"end":{"line":d.line.saturating_sub(1),"character":d.column.saturating_sub(1)}},"severity":1,"message":d.message,"source":"shelly"})).collect::<Vec<_>>()
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
