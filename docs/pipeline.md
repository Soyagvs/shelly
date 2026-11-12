# Capture output from direct-argv process pipelines

Use a `pipeline` block to pass buffered stdout through native process stages and capture the final stdout in one variable. This v1 feature is deliberately not a shell: every stage must be a direct-argv `command` line.

## Quick path

```shelly
pipeline -> $result
    command printf "hello\n"
    command tr a-z A-Z
end
shellout $result
```

Shelly passes only each stage's stdout to the next stage's stdin. Stderr is never chained. The final stdout is captured in the destination variable.

## Stage rules

Each stage must be a direct-argv `command` line. Pipeline arguments use Shelly command-atom rules, not shell source.

```shelly
pipeline -> $lines
    command printf "one\ntwo\n"
    command wc -l
end
shellout $lines
```

`shell` is forbidden inside a pipeline. Shell parsing, expansion, operators, redirections, and implicit environment expansion are not supported. Use explicit argv arguments instead. A stage that tries to capture output or use shell syntax is rejected.

## Buffering and failure

Stages run one at a time with stdout buffered between stages, so this v1 implementation is not streaming and is unsuitable for unbounded output. Shelly writes the buffered input to each stage, closes stdin—even for empty input—and waits for EOF deterministically.

Pipefail semantics apply: any non-zero stage fails the whole pipeline, even if a later stage succeeds. The error identifies the stage and command and retains captured stdout and stderr in structured process-error details for handlers and reporting.

```shelly
pipeline -> $result
    command sh -c "printf partial; exit 7"
    command cat
end
```

This reports stage 1's status and captured output; replacing `command` with `shell` is not allowed.

## Timeouts and dry-run

A timeout after a stage applies to that stage. An enclosing [timeout](timeouts.md) supplies an inherited deadline, and deferred blocks still run while the error unwinds. There is no streaming or unbounded-output exception to these limits.

With `--dry-run`, Shelly validates the direct-argv stages and records planned process actions without spawning them. The destination variable is unavailable because no output exists to capture; the audit log describes the plan, not a completed pipeline. See [Dry-run and audit logs](dry-run.md).

## Related guides

- [Commands](commands.md) — direct process invocation.
- [Timeouts](timeouts.md) — command and block deadlines.
- [Error handling](errors.md) — structured process failures.
