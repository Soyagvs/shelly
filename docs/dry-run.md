# Dry-run and audit logs

Use `shelly --dry-run SCRIPT` to validate a script and print its planned actions in execution order. No script effect is performed; normal reads used to parse and validate the script remain permitted. A final summary is printed after successful execution.

## Audit files

`shelly --dry-run --audit-log PATH SCRIPT [ARGS...]` additionally writes JSON Lines. `--audit-log` is accepted only immediately after `--dry-run`, requires one path, and cannot be repeated or placed after the script. The audit file is the sole dry-run filesystem write. It is opened with create-new semantics: an existing path is never overwritten. Open and write failures are reported as errors. The file is created during preflight, so a later script error can leave a partial log; a script that fails before preflight creates no log.

Each line is one serde-serialized object:

```json
{"schema":"shelly.audit.v1","version":1,"seq":1,"event":"action","action":"write","status":"planned","data":{"path":"/tmp/out"}}
{"schema":"shelly.audit.v1","version":1,"seq":2,"event":"summary","action":"summary","status":"complete","data":{"counts":{"write":1}}}
```

`seq` is monotonic and records match human output order. Action data identifies only the command program and argument count; raw command arguments are never recorded. HTTP method and redacted URL, paths, capture names, and blocking reasons may also be recorded. Payloads, shell source, HTTP bodies and headers, credentials, environment values, and captured output are never audit data. Captures unavailable in dry-run are reported as unavailable rather than fabricated, including values that would otherwise be stale.

## What is blocked

File mutation, archive creation, temporary paths, process execution, shell execution, HTTP requests, logging destinations, interactive input, `wait`, and `watch` are planned or blocked without performing effects. Syntax, paths, command arguments, URLs, and other safe observations are still validated. Read-only observations may inspect the script and required metadata, but dry-run never writes those observations or follows a command's output. Unsupported top-level dry-run commands (`init`, `fmt`, `update`, `test`, and project `run`) are rejected; dry-run is for a script path.

Errors and parser limits are reported normally. Dry-run does not promise that external state remains unchanged if a script invokes unsupported behavior through an extension; use the audit log and the explicit action list for review.

Examples:

```sh
shelly --dry-run examples/deploy.shelly staging
shelly --dry-run --audit-log review.jsonl examples/deploy.shelly staging
```
