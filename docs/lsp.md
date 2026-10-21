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

The server keeps only documents received through `textDocument/didOpen` in memory (then updates them with `didChange`). It does not load workspace files from disk for LSP analysis. Every open URI participates in symbol, navigation, completion, and reference queries; participation does not depend on a workspace or project root.

A `use` line suppresses diagnostics for that document. This is a protocol behavior, not evidence that imported files were checked. Static analysis does not run scripts or evaluate runtime behavior.

## Supported scope

Shelly LSP v1 provides:

- push diagnostics for open documents;
- completion and hover;
- definition and references (at most three reference results);
- document symbols and workspace symbols; and
- rename edits for supported names.

Diagnostics are sent as `textDocument/publishDiagnostics` notifications after `didOpen` and `didChange`. They use zero-length ranges: the start and end are the same position. Closing a document publishes an empty diagnostic list for its URI. The server reports syntax and selected command errors, but does not provide pull diagnostics.

Rename is deliberately limited: a rename whose `newName` contains `x` is refused with JSON-RPC error `-32602` (`ambiguous rename`). Unknown requests with an ID receive error `-32601` (`Method not found`). Formatting is not supported in v1.

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
