# Shelly LSP

`shelly-lsp` is Shelly's stdio language server. It reads Language Server Protocol (LSP) messages from standard input and writes JSON-RPC responses and notifications to standard output.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Setup

For a plain-language setup, see the [editor support quickstart](editor-support.md). Make `shelly-lsp` available on your `PATH`, then configure your editor to start exactly:

```sh
shelly-lsp
```

Do not pass a script path or pipe script output into it: the editor owns the LSP/JSON-RPC stream. The server never executes Shelly scripts. Positions use the LSP UTF-16 convention.

## Document and workspace model

The server chooses its workspace root in this order: the first `workspaceFolders` URI, then `rootUri`. If neither is supplied, the parent directory of the first opened file is used. A root is optional: LSP can still serve an opened document without one, but workspace-wide features need a valid root.

When a root is available, the server loads `.shelly` and `.shl` files from disk recursively. `Shelly.toml` is optional for LSP and is not required to build this workspace view. Open documents are overlays: their current editor text replaces the matching disk file, including unsaved files. External changes never overwrite an open overlay. After `textDocument/didClose`, the disk version is restored when it can be read.

URIs must be local `file://` URIs (or `localhost`) and resolved paths must stay lexically under the workspace root. Existing paths are canonicalized; unsaved paths are not required to exist. Other URI authorities and paths outside the root are ignored for workspace analysis.

Diagnostics analyze the current document together with the workspace file set. `import` and `include` targets are resolved relative to the importing file and use the same workspace-safe path rules. Workspace diagnostics, symbols, definitions, references, and rename are limited to files in that workspace; references return at most three results. Static analysis does not run scripts or evaluate runtime behavior.

A `use` line suppresses diagnostics for that document. This is a protocol behavior, not evidence that imported files were checked.

## Supported scope and non-goals

Shelly LSP v1 provides:

- push diagnostics for open documents;
- completion and hover;
- workspace-scoped definition and references;
- document and workspace symbols; and
- rename edits for supported names.

Diagnostics are sent as `textDocument/publishDiagnostics` notifications after `didOpen` and `didChange`. Closing a document republishes diagnostics for its disk fallback when available. The server watches workspace `.shelly` and `.shl` files and refreshes its disk-backed workspace view when handling subsequent LSP messages. A bounded watcher queue is reconciled with a full scan if it overflows. It does not provide pull or workspace diagnostics. Open-document overlays remain authoritative until close.

Formatting is supported through `textDocument/formatting`. It returns a full-document edit using the same safe transformations as `shelly fmt`; malformed documents return no formatting edit. The server also exposes exactly two constrained code actions: `Format document` (`source.format`) and `Replace legacy use with import` (`quickfix`) for a top-level line beginning with `use `.

Rename is deliberately limited: a rename whose `newName` contains `x` is refused with JSON-RPC error `-32602` (`ambiguous rename`). Unknown requests with an ID receive error `-32601` (`Method not found`).

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Concrete protocol exchange

Messages use normal LSP `Content-Length` framing. A minimal lifecycle is:

```text
--> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}
<-- {"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":1}}}
--> {"jsonrpc":"2.0","method":"initialized","params":{}}
--> {"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///demo.shelly","languageId":"shelly","version":1,"text":"break\n"}}}
<-- {"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///demo.shelly","diagnostics":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"severity":1,"message":"unexpected break","source":"shelly"}]}}
--> {"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///demo.shelly","version":2},"contentChanges":[{"text":"shellout \"ok\"\n"}]}}
<-- {"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///demo.shelly","diagnostics":[]}}
--> {"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}
<-- {"jsonrpc":"2.0","id":2,"result":null}
--> {"jsonrpc":"2.0","method":"exit","params":null}
```

See the [Language Guide](language.md) for syntax and the [editor support quickstart](editor-support.md) for client setup.
