# Environment-only secrets

Shelly's `secret get` reads one secret from the process environment. It does not create, persist, or export secrets. Keep the secret in the environment supplied to `shelly`, rather than writing it in a script.

## Exact syntax

```text
secret get "ENV_NAME" -> $VARIABLE
```

The command must contain a quoted string literal naming the environment variable and exactly one variable capture. The source is the environment of the Shelly process; `secret get` does not read files, prompt for input, consult a keychain, or search project configuration.

```shelly
secret get "DEPLOY_TOKEN" -> $token
```

### Environment names

The name must be non-empty and contain only ASCII letters, digits, and underscores. It must begin with an ASCII letter or underscore.

Valid names include `API_KEY_2` and `_TOKEN`. Invalid names include `2FAST`, `API-KEY`, an empty name, names containing spaces, and names containing dots or other punctuation. The name is case-sensitive according to the host environment.

The variable must exist and contain non-empty text. A missing variable or an empty variable fails with `environment secret is missing or empty`. An environment value that is not valid text fails with `environment secret is not valid text`. Invalid names fail with `invalid environment variable name`. These are ordinary errors and can be handled with `try` / `catch`; see [Error handling](errors.md).

## Safe use

After capture, a secret may be passed as an ordinary value to a direct command argument or to an HTTP request header:

```shelly
secret get "DEPLOY_TOKEN" -> $token
command deploy --token $token

secret get "API_TOKEN" -> $api_token
http GET "https://api.example.test/status" headers {"authorization": "Bearer $api_token"} -> $response
```

`command` builds an argv array and does not invoke a shell. HTTP request headers are scalar text values in the `headers` dictionary; use [HTTP requests](http.md) for the complete request grammar. Do not put secrets in URLs, request bodies, logs, or user-facing output unless you have independently reviewed the destination and redaction behavior.

Shell source is deliberately rejected when it would interpolate a captured secret:

```shelly
secret get "TOKEN" -> $token
shell "curl -H 'Authorization: Bearer $token' https://example.test"
```

Use `command` for an executable and separate argv arguments instead. `shell` is for shell text and cannot safely carry a `secret get` value.

## Opacity and redaction

Secret values are opaque to displayed values and diagnostics. Shelly redacts a successfully loaded secret as `[REDACTED]` in formatted values, normal output, process errors, HTTP failures, and log messages. Redaction also applies to dry-run output and JSONL audit records. The original value is still passed to an allowed direct command or HTTP request; it is not written into the audit record.

Redaction is exact-value replacement, not a guarantee against a program that deliberately exfiltrates its argv or headers. Do not pass a secret to an untrusted executable or endpoint. Avoid short or guessable values, which can also occur as ordinary text.

## Dry-run and audit

`--dry-run` validates the script and reports planned actions without executing direct commands, shell commands, or HTTP requests. Secret values are not printed. With `--audit-log`, action records omit command arguments, payloads, headers, environment values, and captured output; secret values must not appear in the JSONL file. See [Dry-run and audit logs](dry-run.md).

Dry-run is a validation and review aid, not a secret-delivery mechanism. A planned command is not evidence that an external program received the secret.

## Limits and non-goals

Environment-only secrets intentionally do not provide:

- encrypted storage, rotation, masking in a child process, or secret-provider integration;
- file, stdin, prompt, keychain, or project-config secret sources;
- interpolation into shell source, automatic URL/body encoding, or automatic HTTP authentication;
- a way to retrieve a secret without capturing it into a variable; or
- a promise that a called executable or HTTP server will keep the value private.

The supported operation is one environment lookup of one non-empty text value. Use your operating system, deployment platform, or secret manager to provision and rotate the environment before starting Shelly.

## Testing guidance

Tests should provision a dedicated environment variable only for the test process, assert successful capture through a safe direct-command fixture, and verify that stdout, stderr, and audit files contain `[REDACTED]` rather than the sentinel. Also cover invalid names, missing variables, empty variables, non-text values where the platform permits them, shell interpolation rejection, and HTTP header construction. Do not commit real credentials or print test secrets in failure messages.
