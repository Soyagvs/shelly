# Test scripts with virtual file reads

Mocks are test-only declarations for replacing a supported read with known text. They help a `shelly test` file exercise logic without creating or mutating the requested file.

## Quick path

```shelly
mock read "fixtures/input.txt" returns "virtual content"
read "./fixtures/input.txt" -> $text
```

The declaration uses a literal path and literal response. It is valid only while the file is launched by `shelly test`; ordinary script execution, `shelly run`, and `shelly --dry-run script.shelly` reject `mock` declarations.

## Matching rules

- Each test file gets a fresh registry; declarations do not leak between files.
- Paths use the same script-relative normalization as `read`: `.` segments disappear and `..` moves to the parent.
- Matching is exact after normalization. Variables, wildcards, prefixes, and filesystem aliases do not match.
- A test read without an exact mock fails closed before filesystem access and names the safe requested key.
- A matching mock returns virtual text and does not create or mutate a file.

```shelly
mock read "data/../input.txt" returns "ok"
read "input.txt" -> $value
```

## Invalid declarations

```shelly
mock read $path returns "ok"       # path must be literal
mock read "input.txt" returns $x   # response must be literal
mock read "input.txt" returns "ok" # duplicate normalized key
```

A duplicate normalized key is rejected only when the same key is declared twice.

## Limits and boundaries

Shelly v1 supports only `read` mocks with text responses. `size`, `exists`, `files`, archive operations, `shell`, `watch`, temporary paths, and stdin are not covered by read mocks and retain their normal behavior or separate test restrictions. HTTP and command mocks are not read mocks. Mocking does not broaden filesystem permissions or change ordinary scripts.

Dry-run is independent: it records supported effects instead of performing them and does not enable test-only declarations.

## Related guides

- [Dry-run and audit logs](dry-run.md) — planning without execution.
- [Testing commands](commands.md) — command-line entry points.
- [Files and paths](files.md) — real filesystem operations.
