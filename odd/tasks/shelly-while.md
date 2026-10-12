# Shelly while loops

Feature: add `while` loops for condition-based repetition.

## Decisions

- `while <condition>` repeats a block while the condition is true.
- `while` uses the same condition rules as `if`.
- Supported comparisons: `==`, `!=`, `>`, `>=`, `<`, `<=`.
- Truthiness works like `if`.
- `while` uses `end`, like `fn`, `if`, and `repeat`.
- `while` does not create a new scope; assignments inside the loop persist after the block.
- If the condition is false before the first iteration, the body does not run.
- Add a safety limit to prevent accidental infinite loops: 10,000 iterations.
- Exceeding the safety limit is an error with a clear diagnostic.

## Examples

```ly
$count = 0

while $count < 3
    shellout "Count $count"
    $count = $count + 1
end

shellout "Done at $count"
```

```ly
$enabled = true
while $enabled
    shellout "once"
    $enabled = false
end
```

## Tasks

- [x] Add failing tests for while loops.
- [x] Implement while execution.
- [x] Add negative tests for missing `end`, malformed conditions, and iteration limit.
- [x] Update README, language guide, examples, and task evidence.
- [x] Run focused verification.

## Evidence

- RED: `cargo test` after adding tests failed with 7 while-loop failures, including `unknown command or invalid syntax: while $count < 3` and missing expected while diagnostics.
- GREEN: `cargo fmt --check && cargo test` passed; 55 tests passed.
- Example: `printf 'yes\nblue\n' | cargo run --quiet -- example-01.ly` passed and printed the while countdown (`While countdown 2`, `While countdown 1`, `While finished at 0`) along with the existing example output.
