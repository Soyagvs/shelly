# Format Shelly code safely

Use `shelly fmt` to normalize `.shelly` and `.shl` layout without changing program meaning. This page covers the command-line workflow and its boundary with editor formatting.

## 🚀 Quick path

Format one file:

```sh
shelly fmt path/to/script.shelly
```

Check formatting in CI without writing:

```sh
shelly fmt --check path/to/script.shelly
```

The accepted form is `shelly fmt [--check] <path>`: `--check` comes before the path, exactly one path is required, and the path must be last.

## What changes

The formatter:

- converts line endings to LF;
- trims surrounding whitespace from ordinary code lines;
- uses four spaces for nested blocks; and
- preserves a final newline for non-empty output.

Strings, comments, multiline strings, and expression text are preserved rather than interpreted or rewritten. Formatting is idempotent: an already formatted file does not change again.

> **Note:** Formatting never renames identifiers, evaluates expressions, reorders statements, changes literals, expands globs, or otherwise changes program logic.

## Checks and errors

Without `--check`, Shelly rewrites the file only when the formatted result differs. With `--check`, it never writes: an already formatted file succeeds; a difference fails with a message directing you to run `shelly fmt <path>`.

Read, parse, and write failures are errors. Malformed input—including unterminated strings, multiline strings, or comments, unterminated blocks, and unmatched block terminators—is rejected and not mutated, even without `--check`.

## Encoding and limits

Source is read as UTF-8. Invalid UTF-8 is rejected and not rewritten. Unicode in valid strings and comments is preserved. There is no configured line-count, character-count, or file-size limit; practical limits are available memory and filesystem capacity.

Editor positions use UTF-16 because that is the LSP convention. The formatter itself operates on the document text; see [LSP reference](lsp.md) for overlay, URI, and watcher boundaries.

## Related guides

- [LSP reference](lsp.md) — editor formatting and document/workspace behavior.
- [Editor support](editor-support.md) — client setup.
- [Language guide](language.md) — Shelly syntax.
