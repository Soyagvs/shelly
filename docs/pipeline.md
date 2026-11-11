# Native pipelines

Shelly supports block-form native pipelines:

```shelly
pipeline -> $result
    command printf "hello\n"
    command tr a-z A-Z
end
shellout $result
```

Each stage must be a direct-argv `command` line. Shelly passes only the stage's
stdout to the next stage's stdin; stderr is never chained. The final stdout is
captured in the destination variable. Pipeline execution uses pipefail
semantics: a non-zero stage fails the whole pipeline, including when a later
stage would otherwise succeed.

## Errors and safety

A failure identifies the pipeline stage and command, and retains captured
stdout/stderr in structured process-error details for error handlers and
reporting. Timeouts apply to each stage and inherit an enclosing `timeout`.
Deferred blocks still run during unwinding. `shell` is intentionally forbidden
inside a pipeline: shell parsing, expansion, and redirection are not part of
this feature. Use explicit argv arguments instead.

Pipeline arguments use Shelly's normal command-atom rules; they are not shell
source. No shell operators, redirections, or implicit environment expansion are
supported. stdout is buffered between stages, so this v1 implementation is not
streaming and is unsuitable for unbounded output. Each stage's stdin is closed
immediately after its buffered input is written, including when that input is
empty, so stages receive EOF deterministically.

## Dry run and audit

`--dry-run` records the planned direct-argv process stages and does not spawn
processes. The result variable is marked unavailable, because no output exists
to capture. Audit output therefore describes the plan rather than pretending a
pipeline ran.

## Examples

```shelly
pipeline -> $lines
    command printf "one\ntwo\n"
    command wc -l
end
shellout $lines
```

A failing stage is reported instead of being hidden:

```shelly
pipeline -> $result
    command sh -c "printf partial; exit 7"
    command cat
end
```

This reports stage 1's status and captured output. It does not permit replacing
`command` with `shell`.
