# Shelly formatting

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly includes a safe formatter for `.shelly` and `.shl` source files. It changes layout, not program meaning.

## Format a file

```sh
shelly fmt path/to/script.shelly
```

The accepted argument order is:

```text
shelly fmt [--check] <path>
```

`--check` must come before the path. The path must be the final argument; `shelly fmt <path> --check` is rejected with the usage message. Exactly one path is required.

Without `--check`, the formatter rewrites the file only when the result differs. With `--check`, it never writes: it exits successfully when the file is already formatted and exits unsuccessfully with a message telling you to run `shelly fmt <path>` when it differs. Read, parse, and write failures are errors.

## What is changed

The formatter:

- normalizes line endings to LF;
- trims surrounding whitespace from ordinary code lines;
- uses four spaces per nested block;
- preserves a final newline (and adds one to non-empty output); and
- preserves strings, comments, multiline strings, and expression text rather than interpreting or rewriting them.

It does not rename identifiers, evaluate expressions, reorder statements, change literals, expand globs, or otherwise transform program logic. Formatting is idempotent: formatting an already formatted file makes no further change.

Formatting is conservative and safe: the formatter parses enough structure to balance blocks and recognize quoted/commented text before writing. It refuses malformed input, including unterminated quoted or multiline strings, unterminated comments or blocks, and unmatched block terminators. A malformed file is not mutated, even without `--check`.

## UTF-8 and limits

Source files are read as UTF-8. A file that is not valid UTF-8 is rejected and is not rewritten. Unicode text in valid strings and comments is preserved; positions used by the LSP are UTF-16 positions.

There is no configured line-count, character-count, or file-size limit in the formatter. Practical limits are the available memory and filesystem capacity. The LSP's workspace watcher uses a bounded event queue; if it overflows, it performs a full workspace reconciliation rather than silently accepting a partial view.

## Examples

Format and rewrite a file:

```sh
shelly fmt examples/hello.shelly
```

Check it in CI without changing it:

```sh
shelly fmt --check examples/hello.shelly
```

For editor formatting through `shelly-lsp`, see the [LSP reference](lsp.md).
