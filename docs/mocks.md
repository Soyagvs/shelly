# Test read mocks

Shelly v1 supports one test-only mock effect:

```shelly
mock read "fixtures/input.txt" returns "virtual content"
read "./fixtures/input.txt" -> $text
```

The declaration must use a literal path and a literal response. It is valid only while a file is launched by `shelly test`; ordinary `shelly script.shelly`, `shelly run`, and `shelly --dry-run script.shelly` reject `mock` declarations. Each test file gets a fresh registry; declarations never leak to another test file.

## Matching and safety

Read mock keys use the same script-relative path normalization as `read`: `.` segments are removed and `..` moves to the parent. Matching is exact after that normalization; variables, wildcards, prefixes, and filesystem aliases do not match. A test `read` without an exact matching mock fails closed before filesystem access, with an error naming the safe requested key. A matching mock returns its virtual text and never creates or mutates a file.

Valid:

```shelly
mock read "data/../input.txt" returns "ok"
read "input.txt" -> $value
```

Invalid:

```shelly
mock read $path returns "ok"       # target must be a literal
mock read "input.txt" returns $x   # response must be a literal
mock read "input.txt" returns "ok" # duplicate keys are rejected
```

(The last example is invalid only when the same normalized key is declared twice.)

## Scope and limitations

V1 supports only `read` and the text response above. `size`, `exists`, `files`, archive operations, `shell`, `watch`, temporary paths, and stdin are unsupported and still use their normal behavior (or their separate test restrictions). HTTP and command mocks are separate features and are not read mocks. Mocking does not broaden filesystem permissions or alter normal scripts.

Dry-run remains independent: it records supported effects instead of performing them, but is not a mock environment and does not enable test-only declarations.
