# Retry

`retry <attempts>` runs a block again after a known transient failure:

```shelly
retry 3
    http GET "https://example.invalid/health" -> $response
end
```

`attempts` is the total number of executions, including the first. Shelly retries
only known timeout failures. Command exit failures, parse errors, and ordinary runtime errors
are not retried. `break`, `continue`, `return`, and other control-flow signals
always escape immediately. The final failure is preserved unchanged.

Retry does not cancel work already started. Parent `timeout` deadlines still
bound every attempt, and each attempt has the normal deferred-scope behavior.
In `--dry-run`, the body is inspected and audited once; repeated effects are
never executed or recorded as if they happened. Audit output therefore describes
planned work, not successful repeated side effects.

The retry implementation receives the propagated error text, so it recognizes only the exact canonical timeout messages (`timeout` and `command timed out`); it does not inspect arbitrary error text or the structured error dictionary used by `try -> $error`.

Limitations: there is no custom retry predicate, jitter, or backoff syntax yet;
retrying command exit status requires a future explicit condition. Use small
attempt counts and parent timeouts to keep automation bounded.
