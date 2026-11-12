# Review planned script effects without running them

Use `--dry-run` to validate a Shelly script and review its planned actions in execution order without performing script effects. Add an audit log when you need machine-readable JSON Lines for review.

## Quick path

```sh
shelly --dry-run examples/deploy.shelly staging
shelly --dry-run --audit-log review.jsonl examples/deploy.shelly staging
```

The audit option must appear immediately after `--dry-run`, takes exactly one path, and must precede the script path. Dry-run accepts a direct script path; top-level commands such as `init`, `fmt`, `update`, `test`, and project `run` are rejected.

## What dry-run does

- Validates syntax, paths, command arguments, URLs, and other safe observations.
- Prints planned actions in execution order.
- Plans or blocks mutations, archive creation, temporary paths, process and shell execution, HTTP requests, logging destinations, interactive input, `wait`, and `watch` without performing them.
- Keeps parallel planning serial and marks execution-only results, such as pipeline output and parallel results, unavailable.
- Runs retry bodies once for planning; it does not repeat effects or claim repeated successes.

Normal reads needed to parse and validate a script remain allowed. Dry-run does not follow command output or create a mock environment. Extensions can still perform unsupported behavior, so review the explicit action list and audit file.

## Audit log contract

`--audit-log PATH` creates a JSON Lines file with create-new semantics. An existing path is never overwritten. The file is the sole dry-run filesystem write and is created during preflight, so a later script error can leave a partial log; a failure before preflight creates no log.

Each record has `schema: "shelly.audit.v1"`, `version: 1`, and a monotonic `seq`. Action records use `event: "action"` and `status: "planned"`; a successful run ends with `event: "summary"` and `status: "complete"`. Failed runs write an `error` terminal record when possible and never a `complete` record.

```json
{"schema":"shelly.audit.v1","version":1,"seq":1,"event":"action","action":"write","status":"planned","data":{"path":"/tmp/out"}}
{"schema":"shelly.audit.v1","version":1,"seq":2,"event":"summary","action":"summary","status":"complete","data":{"counts":{"write":1}}}
```

Action data is deliberately bounded: command records identify the program and argument count, not raw arguments. HTTP method and redacted URL, paths, capture names, and blocking reasons may be recorded. Payloads, source, HTTP bodies and headers, credentials, environment values, and captured output are not audit data.

## Durability and errors

Terminal records are written first, then Shelly attempts `sync_all()`. A synchronization failure is an error even if a `complete` record is already present. Successful synchronization is not a guarantee against OS, filesystem, storage-device, or hardware failure; the exact durability depends on the platform and filesystem. Open and write failures are reported as errors.

> **Note:** An audit log proves what Shelly recorded and synchronized; it does not prove that planned actions were safe on every external system.

## Related guides

- [Pipelines](pipeline.md) — direct-argv process stages and dry-run output.
- [Parallel tasks](parallel.md) — bounded concurrency and planned task records.
- [Testing with mocks](mocks.md) — test-only virtual reads, separate from dry-run.
- [Error handling](errors.md) — catchable runtime failures.
