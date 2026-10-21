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
            .expect("spawn shelly-lsp");
        let stdout = child.stdout.take().expect("capture shelly-lsp stdout");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                if line.starts_with("Content-Length:") {
                    let length: usize = line[15..].trim().parse().expect("content length");
                    let mut blank = String::new();
                    reader
                        .read_line(&mut blank)
                        .expect("read JSON-RPC separator");
                    let mut body = vec![0; length];
                    reader.read_exact(&mut body).expect("read JSON-RPC body");
                    tx.send(serde_json::from_slice(&body).expect("valid JSON-RPC response"))
                        .expect("receiver is alive");
                }
            }
        });
        Self {
            stdin: child.stdin.take().expect("capture shelly-lsp stdin"),
            child,
            messages: rx,
        }
    }

    fn send(&mut self, message: Value) {
        let body = serde_json::to_vec(&message).expect("serialize JSON-RPC message");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())
            .expect("write JSON-RPC headers");
        self.stdin.write_all(&body).expect("write JSON-RPC body");
        self.stdin.flush().expect("flush JSON-RPC message");
    }

    fn receive(&self) -> Value {
        self.messages
            .recv_timeout(Duration::from_secs(2))
            .expect("receive LSP message")
    }
}

impl Drop for Lsp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn diagnostics_first_lsp_lifecycle_and_zero_based_ranges() {
    let mut lsp = Lsp::start();
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}));
    let initialized = lsp.receive();
    assert_eq!(
        initialized["result"]["capabilities"]["diagnosticProvider"]["interFileDependencies"],
        false
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));

    let uri = "file:///unsaved.shelly";
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"shelly","version":1,"text":"break\n"}}}));
    let invalid = lsp.receive();
    assert_eq!(invalid["method"], "textDocument/publishDiagnostics");
    assert_eq!(
        invalid["params"]["diagnostics"][0]["range"]["start"],
        json!({"line":0,"character":0})
    );
    assert_eq!(
        invalid["params"]["diagnostics"][0]["range"]["end"]["line"],
        0
    );

    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"shellout \"ok\"\n"}]}}));
    let fixed = lsp.receive();
    assert_eq!(fixed["params"]["diagnostics"], json!([]));

    lsp.send(json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}));
    assert_eq!(lsp.receive()["result"], Value::Null);
    lsp.send(json!({"jsonrpc":"2.0","method":"exit","params":null}));
    assert!(lsp.child.wait().expect("wait for shelly-lsp").success());
}

#[test]
fn lsp_v1_advertises_language_capabilities_including_formatting_and_code_actions() {
    let mut lsp = Lsp::start();
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}));
    let capabilities = &lsp.receive()["result"]["capabilities"];
    for capability in [
        "completionProvider",
        "hoverProvider",
        "definitionProvider",
        "referencesProvider",
        "renameProvider",
        "documentSymbolProvider",
        "workspaceSymbolProvider",
    ] {
        assert!(
            capabilities[capability].is_object() || capabilities[capability] == true,
            "missing advertised capability: {capability}"
        );
    }
    assert_eq!(capabilities["documentFormattingProvider"], true);
    assert_eq!(
        capabilities["codeActionProvider"]["codeActionKinds"],
        json!(["quickfix", "source.format"])
    );
}

#[test]
fn lsp_v1_keeps_multiple_documents_isolated_and_applies_utf16_ranged_changes() {
    let mut lsp = Lsp::start();
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}));
    lsp.receive();
    let first = "file:///first.shelly";
    let second = "file:///second.shelly";
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":first,"languageId":"shelly","version":1,"text":"ébreak\n"}}}));
    let first_diagnostics = lsp.receive();
    assert_eq!(first_diagnostics["params"]["uri"], first);
    assert_eq!(
        first_diagnostics["params"]["diagnostics"][0]["range"]["start"]["character"],
        0
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":second,"languageId":"shelly","version":1,"text":"shellout \"ok\"\n"}}}));
    let second_diagnostics = lsp.receive();
    assert_eq!(second_diagnostics["params"]["uri"], second);
    assert_eq!(second_diagnostics["params"]["diagnostics"], json!([]));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":first,"version":2},"contentChanges":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":6}},"rangeLength":5,"text":"shellout \"ok\""}]}}));
    let changed = lsp.receive();
    assert_eq!(changed["params"]["uri"], first);
    assert_eq!(changed["params"]["diagnostics"], json!([]));
}

#[test]
fn lsp_v1_rejects_unknown_request_and_remains_usable() {
    let mut lsp = Lsp::start();
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}));
    lsp.receive();
    lsp.send(json!({"jsonrpc":"2.0","id":2,"method":"textDocument/unknown","params":{}}));
    let error = lsp.receive();
    assert_eq!(error["id"], 2);
    assert!(error["error"].is_object());
    lsp.send(json!({"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}));
    let shutdown = lsp.receive();
    assert_eq!(shutdown["id"], 3);
    assert_eq!(shutdown["result"], Value::Null);
}
