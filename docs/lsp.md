# Shelly LSP

`shelly-lsp` is Shelly's language server. It reads Language Server Protocol (LSP) messages from standard input and writes responses and notifications to standard output.

## Setup

For a plain-language setup, see the [editor support quickstart](editor-support.md). Make `shelly-lsp` available on your `PATH`, then configure your editor to start exactly:

```sh
shelly-lsp
```

This is a stdio server. Do not pass a script path or pipe script output into it; the editor owns the LSP/JSON-RPC stream. The server never executes Shelly scripts. Editor positions use the LSP UTF-16 convention implicitly, so clients should send and consume normal LSP positions.

For Neovim, see the [setup](../editors/nvim/README.md).

## Supported scope

Shelly LSP v1 provides static, workspace-aware analysis for open workspace documents:

- diagnostics, including module diagnostics;
- completion and hover;
- definition and references;
- document symbols and workspace symbols; and
- rename when the target can be resolved safely.

Module diagnostics use the workspace files as a baseline, but an open, unsaved buffer overrides the corresponding file. Diagnostics therefore reflect edits currently held by the editor. Static analysis does not run scripts or evaluate runtime behavior.

Rename and navigation are deliberately conservative: dynamic or ambiguous targets may be refused rather than guessed. Rename can likewise return an error when safety cannot be established. Formatting is not supported in v1; the server is not a formatter.

The server tracks `didOpen`, `didChange`, and `didClose` documents and supports the normal `initialize`/`shutdown` lifecycle. See the [Language Guide](language.md) for syntax and [Project structure](project-structure.md) for workspace organization.
