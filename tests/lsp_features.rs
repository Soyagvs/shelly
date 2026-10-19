use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

struct Lsp {
    child: Child,
    stdin: ChildStdin,
    messages: Receiver<Value>,
}
impl Lsp {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_shelly-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                if r.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                if let Some(n) = line.strip_prefix("Content-Length:") {
                    let n: usize = n.trim().parse().unwrap();
                    let mut blank = String::new();
                    r.read_line(&mut blank).unwrap();
                    let mut body = vec![0; n];
                    r.read_exact(&mut body).unwrap();
                    tx.send(serde_json::from_slice(&body).unwrap()).unwrap();
                }
            }
        });
        Self {
            stdin: child.stdin.take().unwrap(),
            child,
            messages: rx,
        }
    }
    fn send(&mut self, value: Value) {
        let body = serde_json::to_vec(&value).unwrap();
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        self.stdin.write_all(&body).unwrap();
        self.stdin.flush().unwrap();
    }
    fn recv(&self) -> Value {
        self.messages.recv_timeout(Duration::from_secs(2)).unwrap()
    }
    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        self.recv()
    }
    fn initialize(&mut self) {
        self.request(1, "initialize", json!({"capabilities":{}}));
        self.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    }
    fn open(&mut self, uri: &str, text: &str) -> Value {
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"shelly","version":1,"text":text}}}));
        self.recv()
    }
}
impl Drop for Lsp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn doc(uri: &str) -> Value {
    json!({"uri":uri,"version":1})
}
fn pos(line: u64, character: u64) -> Value {
    json!({"line":line,"character":character})
}

#[test]
fn completion_in_incomplete_code_returns_items_at_utf16_position() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///completion.shl";
    l.open(uri, "let café = 1\ncaf");
    let r = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,3)}),
    );
    assert!(
        r["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "café")
    );
}

#[test]
fn completion_includes_builtin_helpers_without_dropping_declarations() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///helpers.shl";
    l.open(uri, "let custom = 1\n");
    let r = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,0)}),
    );
    let items = r["result"]["items"].as_array().unwrap();
    for name in ["custom", "substring", "padLeft", "padRight", "formatDate"] {
        assert!(items.iter().any(|i| i["label"] == name), "missing {name}");
    }
}

#[test]
fn helper_hover_contains_signature_and_contract() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///helper-hover.shl";
    l.open(uri, "formatDate(value, \"YYYY/MM/DD\")\n");
    let r = l.request(
        2,
        "textDocument/hover",
        json!({"textDocument":doc(uri),"position":pos(0,2)}),
    );
    let value = r["result"]["contents"]["value"].as_str().unwrap();
    assert!(value.contains("formatDate(iso, format)"));
    assert!(value.contains("strict UTC ISO input"));
    assert!(value.contains("YYYY/MM/DD/HH/mm/ss"));
}

#[test]
fn hover_at_declaration_and_reference_has_contents_and_utf16_range() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///hover.shl";
    l.open(uri, "let café = 1\ncafé\n");
    for (id, line, character) in [(2, 0, 5), (3, 1, 2)] {
        let r = l.request(
            id,
            "textDocument/hover",
            json!({"textDocument":doc(uri),"position":pos(line,character)}),
        );
        assert!(r["result"]["contents"].is_object() || r["result"]["contents"].is_string());
        let (start, end) = if line == 0 { (4, 8) } else { (0, 4) };
        assert_eq!(
            r["result"]["range"],
            json!({"start":{"line":line,"character":start},"end":{"line":line,"character":end}})
        );
    }
}

#[test]
fn definition_and_references_resolve_same_file_and_imported_export() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///main.shl";
    l.open("file:///lib.shl", "export fn café() {}\n");
    l.open(uri, "use \"lib.shl\"\ncafé()\ncafé()\n");
    let d = l.request(
        2,
        "textDocument/definition",
        json!({"textDocument":doc(uri),"position":pos(1,2)}),
    );
    assert_eq!(d["result"][0]["uri"], "file:///lib.shl");
    assert_eq!(
        d["result"][0]["range"],
        json!({"start":{"line":0,"character":11},"end":{"line":0,"character":16}})
    );
    let refs = l.request(
        3,
        "textDocument/references",
        json!({"textDocument":doc(uri),"position":pos(2,2),"context":{"includeDeclaration":true}}),
    );
    assert_eq!(refs["result"].as_array().unwrap().len(), 3);
}

#[test]
fn rename_is_safe_and_ambiguous_requests_are_refused() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///rename.shl";
    l.open(uri, "let café = 1\ncafé\n");
    let r = l.request(
        2,
        "textDocument/rename",
        json!({"textDocument":doc(uri),"position":pos(1,2),"newName":"total"}),
    );
    assert_eq!(
        r["result"]["changes"][uri][0]["range"],
        json!({"start":{"line":0,"character":4},"end":{"line":0,"character":8}})
    );
    let ambiguous = l.request(
        3,
        "textDocument/rename",
        json!({"textDocument":doc(uri),"position":pos(1,2),"newName":"x"}),
    );
    assert!(ambiguous["error"].is_object());
}

#[test]
fn document_and_workspace_symbols_return_utf16_ranges() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///symbols.shl";
    l.open(uri, "fn café() {}\nconst VALUE = 1\n");
    let d = l.request(
        2,
        "textDocument/documentSymbol",
        json!({"textDocument":doc(uri)}),
    );
    assert_eq!(d["result"][0]["range"]["start"], pos(0, 3));
    let w = l.request(3, "workspace/symbol", json!({"query":"café"}));
    assert_eq!(w["result"][0]["location"]["uri"], uri);
}

#[test]
fn watch_and_wait_forms_are_diagnosed_without_extra_capabilities() {
    let mut l = Lsp::start();
    l.initialize();
    let valid = l.open(
        "file:///wait-valid.shelly",
        "watch printf output -> $line\nshellout $line\nend\nwait\n",
    );
    assert_eq!(valid["params"]["diagnostics"], json!([]));

    let invalid_wait = l.open("file:///wait-invalid.shelly", "wait now\n");
    assert!(
        invalid_wait["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"].as_str().unwrap().contains("wait"))
    );

    let invalid_watch = l.open(
        "file:///watch-invalid.shelly",
        "watch printf output -> line\nend\n",
    );
    assert!(
        invalid_watch["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"].as_str().unwrap().contains("watch"))
    );

    let empty_destination = l.open(
        "file:///watch-empty-destination.shelly",
        "watch printf output -> $\nend\n",
    );
    assert!(
        empty_destination["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"] == "watch destination must be a variable")
    );
}

#[test]
fn recursive_file_glob_is_accepted_without_diagnostics() {
    let mut l = Lsp::start();
    l.initialize();
    let notification = l.open("file:///globs.shelly", "files \"**/*.shelly\" -> $items\n");
    assert_eq!(notification["params"]["diagnostics"], json!([]));
}

#[test]
fn module_project_diagnostics_use_open_buffer_override() {
    let mut l = Lsp::start();
    l.initialize();
    l.open("file:///lib.shl", "export fn good() {}\n");
    let notification = l.open("file:///main.shl", "use \"lib.shl\"\ngood()\n");
    assert_eq!(notification["params"]["diagnostics"], json!([]));
}
