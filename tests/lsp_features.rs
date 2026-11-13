use serde_json::{Value, json};

#[test]
fn utf16_ranges_clamp_invalid_offsets_without_panicking() {
    let text = "a🌍b\nélan";
    assert_eq!(shelly::lsp::features::utf16_pos(text, 2), (0, 1));
    assert_eq!(shelly::lsp::features::utf16_pos(text, usize::MAX), (1, 4));
    assert_eq!(
        shelly::lsp::features::range(text, 2, usize::MAX),
        json!({"start":{"line":0,"character":1},"end":{"line":1,"character":4}})
    );
}
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
        self.initialize_with_root(None);
    }
    fn initialize_with_root(&mut self, root: Option<&str>) {
        let mut params = json!({"capabilities":{}});
        if let Some(uri) = root {
            params["rootUri"] = json!(uri);
        }
        self.request(1, "initialize", params);
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
fn file_uri(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[test]
fn completion_contexts_offer_format_date_http_and_menu_values() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///values.shelly";
    l.open(uri, "formatDate($now, \"Y");
    let date = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,19)}),
    );
    assert!(
        date["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "YYYY")
    );
    l.open(uri, "http GET \"https://x\" ");
    let http = l.request(
        3,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,21)}),
    );
    assert!(
        http["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "timeout")
    );
    l.open(uri, "menu \"Pick\" -> $choice\n    keys");
    let menu = l.request(
        4,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,8)}),
    );
    assert!(
        menu["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "letters lower")
    );
}

#[test]
fn completion_contexts_offer_import_and_include_modules() {
    let mut l = Lsp::start();
    l.initialize();
    l.open("file:///palette.shelly", "export const ERROR = \"red\"\n");
    let uri = "file:///main.shelly";
    l.open(uri, "include \"");
    let include = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,9)}),
    );
    assert!(
        include["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "palette.shelly")
    );
}

#[test]
fn completion_contexts_offer_known_result_and_error_properties() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///properties.shelly";
    l.open(uri, "$result.");
    let result = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,8)}),
    );
    assert!(
        result["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "durationMs")
    );
    l.open(uri, "$error.");
    let error = l.request(
        3,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,7)}),
    );
    assert!(
        error["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "message")
    );
}

#[test]
fn completion_contexts_offer_block_directives() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///blocks.shelly";
    l.open(uri, "pipeline -> $out\n    ");
    let pipeline = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,4)}),
    );
    assert!(
        pipeline["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "command")
    );
    l.open(uri, "menu \"Pick\" -> $choice\n    ");
    let menu = l.request(
        3,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,4)}),
    );
    assert!(
        menu["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "option")
    );
}

#[test]
fn completion_includes_commands_and_block_signatures() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///commands.shelly";
    l.open(uri, "pro");
    let r = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,3)}),
    );
    let items = r["result"]["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|i| i["label"] == "progress"
                && i["detail"].as_str().unwrap().contains("-> $result"))
    );
    assert!(items.iter().any(|i| i["label"] == "menu"));
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
fn completion_suggests_style_descriptors_only_in_the_first_style_argument() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///style-completion.shelly";
    l.open(uri, "shellout style(\"br");
    let r = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,18)}),
    );
    let items = r["result"]["items"].as_array().unwrap();
    assert!(items.iter().any(|i| i["label"] == "bright_blue"));
    assert!(items.iter().any(|i| i["label"] == "bold"));

    l.open(uri, "shellout style(\"bold\", \"va");
    let second = l.request(
        3,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(0,25)}),
    );
    assert!(
        !second["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "bright_blue")
    );
}

#[test]
fn completion_in_style_includes_exported_palette_constants() {
    let mut l = Lsp::start();
    l.initialize();
    l.open(
        "file:///palette.shelly",
        "export const ERROR = \"bold bright_red\"\n",
    );
    let uri = "file:///main.shelly";
    l.open(uri, "import \"palette.shelly\"\nshellout style($E");
    let r = l.request(
        2,
        "textDocument/completion",
        json!({"textDocument":doc(uri),"position":pos(1,18)}),
    );
    assert!(
        r["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["label"] == "$ERROR")
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
    l.open(uri, "import \"lib.shl\"\ncafé()\ncafé()\ncafé()\n");
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
    assert_eq!(refs["result"].as_array().unwrap().len(), 4);
}

#[test]
fn nested_sibling_imports_resolve_protocol_locations_and_cross_file_overlay_rename() {
    let root = std::env::temp_dir().join(format!(
        "shelly-lsp-protocol-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let defs = root.join("lib");
    let app = root.join("app");
    std::fs::create_dir_all(&defs).unwrap();
    std::fs::create_dir_all(&app).unwrap();
    let def = defs.join("defs.shl");
    let main = app.join("main.shl");
    std::fs::write(&def, "export fn shared() {}\n").unwrap();
    std::fs::write(&main, "import \"../lib/defs.shl\"\nshared()\n").unwrap();
    let def_uri = file_uri(&def);
    let main_uri = file_uri(&main);
    let mut l = Lsp::start();
    l.initialize_with_root(Some(&file_uri(&root)));
    // Keep the consumer open as an overlay while the definition remains disk-backed.
    l.open(
        &main_uri,
        "import \"../lib/defs.shl\"\nshared()\nshared()\nshared()\n",
    );

    let definition = l.request(
        2,
        "textDocument/definition",
        json!({
            "textDocument":doc(&main_uri), "position":pos(1,2)
        }),
    );
    assert_eq!(
        definition["result"],
        json!([{
            "uri": def_uri,
            "range":{"start":{"line":0,"character":11},"end":{"line":0,"character":18}}
        }])
    );
    let references = l.request(
        3,
        "textDocument/references",
        json!({
            "textDocument":doc(&main_uri), "position":pos(2,2),
            "context":{"includeDeclaration":true}
        }),
    );
    let refs = references["result"].as_array().unwrap();
    assert_eq!(refs.len(), 4);
    for expected in [
        json!({"uri":main_uri,"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":6}}}),
        json!({"uri":main_uri,"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":6}}}),
        json!({"uri":main_uri,"range":{"start":{"line":3,"character":0},"end":{"line":3,"character":6}}}),
        json!({"uri":def_uri,"range":{"start":{"line":0,"character":10},"end":{"line":0,"character":16}}}),
    ] {
        assert!(
            refs.iter().any(|actual| actual == &expected),
            "missing reference: {expected}"
        );
    }
    let rename = l.request(
        4,
        "textDocument/rename",
        json!({
            "textDocument":doc(&main_uri), "position":pos(1,2), "newName":"renamed"
        }),
    );
    assert_eq!(
        rename["result"]["changes"][&main_uri],
        json!([
            {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":6}},"newText":"renamed"},
            {"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":6}},"newText":"renamed"},
            {"range":{"start":{"line":3,"character":0},"end":{"line":3,"character":6}},"newText":"renamed"}
        ])
    );
    assert_eq!(
        rename["result"]["changes"][&def_uri],
        json!([
            {"range":{"start":{"line":0,"character":10},"end":{"line":0,"character":16}},"newText":"renamed"}
        ])
    );
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
fn formatting_requests_cover_changed_noop_malformed_and_utf16_ranges() {
    let mut l = Lsp::start();
    l.initialize();

    let changed_uri = "file:///format-changed.shl";
    l.open(changed_uri, " if true\n shellout \"🌍\"\nend\n");
    let changed = l.request(
        2,
        "textDocument/formatting",
        json!({"textDocument":doc(changed_uri),"options":{}}),
    );
    assert_eq!(
        changed["result"][0]["newText"],
        "if true\n    shellout \"🌍\"\nend\n"
    );
    assert_eq!(
        changed["result"][0]["range"],
        json!({"start":pos(0,0),"end":pos(3,0)})
    );

    let clean_uri = "file:///format-clean.shl";
    l.open(clean_uri, "shellout \"ok\"\n");
    let clean = l.request(
        3,
        "textDocument/formatting",
        json!({"textDocument":doc(clean_uri),"options":{}}),
    );
    assert_eq!(clean["result"], json!([]));

    let malformed_uri = "file:///format-malformed.shl";
    l.open(malformed_uri, "if true\n");
    let malformed = l.request(
        4,
        "textDocument/formatting",
        json!({"textDocument":doc(malformed_uri),"options":{}}),
    );
    assert_eq!(malformed["result"], json!([]));
}

#[test]
fn safe_code_actions_offer_legacy_import_fix_and_format_edit() {
    let mut l = Lsp::start();
    l.initialize();
    let uri = "file:///actions.shl";
    l.open(uri, "use \"lib.shl\"\n shellout \"ok\"\n");
    let actions = l.request(2, "textDocument/codeAction", json!({
        "textDocument":doc(uri), "range":{"start":pos(0,0),"end":pos(1,0)}, "context":{"diagnostics":[]}
    }));
    let actions = actions["result"].as_array().unwrap();
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0]["title"], "Replace legacy use with import");
    assert_eq!(actions[0]["edit"]["changes"][uri][0]["newText"], "import");
    assert_eq!(actions[1]["title"], "Format document");
    assert_eq!(actions[1]["kind"], "source.format");
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
fn clear_is_accepted_without_diagnostics_and_rejects_arguments() {
    let mut l = Lsp::start();
    l.initialize();
    let valid = l.open("file:///clear-valid.shelly", "clear\n");
    assert_eq!(valid["params"]["diagnostics"], json!([]));

    let invalid = l.open("file:///clear-invalid.shelly", "clear now\n");
    assert!(
        invalid["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"] == "clear takes no arguments")
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
fn unsupported_use_directive_is_diagnosed() {
    let mut l = Lsp::start();
    l.initialize();
    let notification = l.open("file:///unsupported-use.shl", "use \"lib.shl\"\n");
    assert!(
        !notification["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn module_project_diagnostics_use_open_buffer_override() {
    let mut l = Lsp::start();
    l.initialize();
    l.open("file:///lib.shl", "export fn good() {}\n");
    let notification = l.open("file:///main.shl", "import \"lib.shl\"\n");
    assert_eq!(notification["params"]["diagnostics"], json!([]));
}

#[test]
fn module_diagnostics_use_unsaved_cross_file_overlay_and_file_uri_mapping() {
    let root = std::env::temp_dir().join(format!("shelly-lsp-overlay-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let lib = root.join("lib.shl");
    let main = root.join("main.shl");
    std::fs::write(&lib, "export fn good() {}\n").unwrap();
    std::fs::write(&main, "import \"lib.shl\"\n").unwrap();
    let mut l = Lsp::start();
    l.initialize_with_root(Some(&file_uri(&root)));
    l.open(&file_uri(&lib), "export fn broken(\n");
    let notification = l.open(&file_uri(&main), "import \"lib.shl\"\n");
    assert!(
        !notification["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
