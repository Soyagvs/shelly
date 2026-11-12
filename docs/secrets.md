# Read non-empty secrets from the environment

Use `secret get` to capture one non-empty text value from the Shelly process environment. Shelly does not create, persist, export, or source secrets from files or configuration.

## Quick path

```shelly
secret get "DEPLOY_TOKEN" -> $token
command deploy --token $token
```

The command requires a quoted environment-name literal and exactly one variable capture. Names are non-empty, case-sensitive, and contain only ASCII letters, digits, and underscores; the first character is an ASCII letter or `_`.

## Failures and safe transport

A missing or empty variable fails with `environment secret is missing or empty`; non-text values fail with `environment secret is not valid text`; invalid names fail with `invalid environment variable name`. These are ordinary errors handled with [try/catch](errors.md).

`command` builds an argv array without invoking a shell, so pass the secret as a separate argument. HTTP headers accept scalar text values:

```shelly
secret get "API_TOKEN" -> $token
http GET "https://example.test/status" headers {"authorization": "Bearer $token"} -> $response
```

Shell source is deliberately rejected when it interpolates a captured secret. Do not place secrets in URLs, bodies, logs, or user-facing output unless the destination and redaction behavior have been reviewed.

## Redaction and audit

Successfully loaded secret values are redacted as `[REDACTED]` in formatted values, normal output, process errors, HTTP failures, log messages, dry-run output, and JSONL audit records. The original value is still passed to an allowed direct command or HTTP request; it is not written to the audit record.

Redaction is exact-value replacement, not protection against a program or server that deliberately exfiltrates argv or headers. Avoid short or guessable values and untrusted destinations.

`--dry-run` validates and reports planned actions without executing direct commands, shell commands, or HTTP requests. It is not a secret-delivery mechanism. With `--audit-log`, command arguments, payloads, headers, environment values, and captured output are omitted.

## Limits and non-goals

Shelly provides no encrypted storage, rotation, child-process masking, provider integration, file/stdin/prompt/keychain/config sources, shell interpolation, automatic URL/body encoding, automatic HTTP authentication, or guarantee that a called program or server keeps the value private. Provision and rotate environment values using your operating system, deployment platform, or secret manager.

For test guidance, use a process-local fixture and assert `[REDACTED]` without printing the sentinel. Cover invalid names, missing/empty/non-text values, shell rejection, safe command arguments, and HTTP headers; never commit real credentials.
