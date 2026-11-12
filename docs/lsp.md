# Connect an editor to Shelly LSP

Use `shelly-lsp` for diagnostics, navigation, completion, hover, rename, and safe document formatting over stdio. The editor owns the LSP/JSON-RPC stream; the server never executes Shelly scripts.

## 🚀 Quick path

Put `shelly-lsp` on `PATH` and configure the client to start exactly:

```sh
shelly-lsp
```

Do not pass a script path or pipe script output into the process. For editor-specific setup, see [Editor support](editor-support.md).

## Document and workspace boundaries

The workspace root is selected in this order:

1. the first `workspaceFolders` URI;
2. `rootUri`; or
3. the parent directory of the first opened file.

A root is optional for an opened document, but workspace-wide features require one. With a root, Shelly recursively loads `.shelly` and `.shl` files; `Shelly.toml` is optional.

Open documents are authoritative overlays: current editor text, including unsaved text, replaces the matching disk file. External changes do not overwrite an open overlay. After `textDocument/didClose`, the readable disk version becomes authoritative again.

Only local `file://` URIs (or `localhost`) are accepted. Resolved paths must remain lexically under the workspace root. Existing paths are canonicalized; unsaved paths need not exist. Other URI authorities and outside-root paths are ignored for workspace analysis. `import` and `include` resolve relative to the importing file under the same rules.

## Positions, diagnostics, and watching

Clients must send LSP UTF-16 character offsets, not byte offsets. Shelly clamps malformed or stale internal byte spans to safe UTF-8 boundaries.

Diagnostics are pushed after `didOpen` and `didChange` through `textDocument/publishDiagnostics`. The target document and other open workspace files use their overlays; closing a document republishes diagnostics for its disk fallback when readable. A `use` line suppresses diagnostics for that document; it does not prove imported files were checked.

The server watches workspace `.shelly` and `.shl` files and refreshes its disk-backed view while handling later LSP messages. The watcher queue is bounded and non-blocking. Overflow triggers full reconciliation; an incomplete scan preserves the last known state instead of treating inaccessible paths as deletions. Open overlays remain authoritative until close.

## Supported features and limits

| Feature | Boundary |
| --- | --- |
| Diagnostics | Push notifications; no pull or workspace diagnostics |
| Completion and hover | Static analysis only |
| Definitions, references, symbols | Workspace-scoped; references return at most three results |
| Rename | Supported names only; a `newName` containing `x` returns `-32602` (`ambiguous rename`) |
| Formatting | Full-document edit using the same safe transformations as `shelly fmt` |
| Code actions | Exactly `Format document` (`source.format`) and top-level `use ` replacement (`quickfix`) |

Malformed documents return no formatting edit. Clients should apply the full-document edit while preserving UTF-16 ranges. Unknown requests with an ID return `-32601` (`Method not found`). Code actions are narrow and never execute code.

## Minimal protocol lifecycle

Messages use normal LSP `Content-Length` framing:

```text
--> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}
<-- {"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":1}}}
--> {"jsonrpc":"2.0","method":"initialized","params":{}}
--> {"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}
<-- {"jsonrpc":"2.0","id":2,"result":null}
--> {"jsonrpc":"2.0","method":"exit","params":null}
```

## Related guides

- [Formatting](formatting.md) — command-line formatting and `fmt --check`.
- [Editor support](editor-support.md) — client configuration.
- [Language guide](language.md) — syntax and static analysis context.
