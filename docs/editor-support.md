# Editor support quickstart

Shelly can point out many mistakes while you type. This help is for both first-time users and people who do not want to learn editor internals.

## What the corrector does

The Shelly corrector is the `shelly-lsp` program. Your editor sends it the open `.shelly` file, and it sends back diagnostics (underlined problems), suggestions, and information about symbols. It does **not** run your script or its commands.

You need `shelly-lsp` installed and available as `shelly-lsp` in your terminal. Release archives include it alongside `shelly`; installing the main `shelly` command alone is not enough.

> **Easy setup:** use the steps below for [VS Code](https://github.com/Soyagvs/shelly-vscode) or [Neovim](https://github.com/Soyagvs/shelly-nvim). [LSP reference](lsp.md) explains the protocol and the complete feature scope. Mason is not an installation option yet; the Shelly package has not been published to the upstream Mason registry.

## VS Code

1. Install the [Shelly VS Code extension](https://github.com/Soyagvs/shelly-vscode).
2. Install Shelly from a release archive, and make sure the archive's directory is on your `PATH`.
3. Restart VS Code, then open a `.shelly` file (or a folder containing one).

Follow the extension's setup, then open a `.shelly` file and type an obvious syntax mistake. Look for a diagnostic underline or a problem in the **Problems** panel. Remove the mistake: the diagnostic should disappear.

If nothing appears:

- In VS Code's integrated terminal, run `shelly-lsp`. If the command is not found, fix `PATH` and restart VS Code.
- Confirm the file name ends in `.shelly` and that the Shelly extension is enabled.
- Check **View → Output**, select the Shelly extension or language-server channel, and look for a startup error.

## Neovim

1. Install [`shelly-nvim`](https://github.com/Soyagvs/shelly-nvim) by following its README.
2. Install Shelly, including `shelly-lsp`, and put `shelly-lsp` on your `PATH`.
3. Open a `.shelly` file in Neovim.

Use the plugin's setup instructions, then open a `.shelly` file. You should see diagnostics for invalid Shelly syntax, and completion or hover when your Neovim setup displays them.

If nothing appears:

- From the same environment that starts Neovim, run `shelly-lsp` to verify `PATH`.
- Check `:set filetype?`; it should say `filetype=shelly`.
- Check `:messages` for the actionable warning emitted when the executable cannot be started.
- Make sure the buffer is backed by a `.shelly` file, not an unnamed or differently named buffer.

## What to expect—and what not to expect

The language server uses the first `workspaceFolders` URI, then `rootUri`; without either, it falls back to the first opened file's parent. `Shelly.toml` is optional for LSP. With a root, `.shelly` and `.shl` files are loaded from disk recursively, and later external file changes are refreshed automatically while handling LSP messages. An open buffer is an overlay and is never replaced by those disk refreshes; closing the buffer falls back to disk when readable.

Diagnostics, symbols, definitions, references, and rename are workspace-scoped. `import` and `include` resolve relative to the importing file and obey the same local-URI and root-boundary safety rules. Dynamic or ambiguous targets may not be resolved, and references are capped. The server supports document formatting with the safe formatter, and offers two constrained code actions: format the document, or replace a top-level legacy `use ` line with `import`. See the [LSP reference](lsp.md) for exact limits.

The LSP accepts `files` patterns including recursive `**` syntax (for example, `files "**/*.shelly" -> $items`) without reporting a syntax diagnostic. The LSP does not expand globs or inspect the filesystem for that command; expansion happens when Shelly runs the script.

For command-line formatting and `fmt --check`, see [Formatting](formatting.md). For integration details, see the [Shelly VS Code extension](https://github.com/Soyagvs/shelly-vscode) or [Shelly Neovim plugin](https://github.com/Soyagvs/shelly-nvim).
