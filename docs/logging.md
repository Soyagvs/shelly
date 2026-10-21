# Logging

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Use a `log` block when you want a structured trace of a script section without hiding normal output.

```shelly
log "deploy"
    shellout "starting"
    command printf running
end
```

## Syntax

```shelly
log "identifier"
    # statements
end
```

- An identifier expression is required; a missing expression (`log`) is rejected.
- The identifier is evaluated like other Shelly string/value expressions, so variables and interpolation can provide the name. An explicit empty string (`log ""`) is accepted and sanitizes to the fallback name `log`.
- Blocks close with `end` and can contain nested blocks, including nested `log` blocks.
- `shelly check` validates `log` structure without running the block.

## Storage

Shelly creates a `logs/` directory next to the running script.

Each identifier writes to one file per UTC day:

```text
logs/<sanitized-identifier>-YYYY-MM-DD.log
```

Sanitization keeps ASCII letters, digits, `_`, and `-`. Other characters become `-`. If the sanitized name is empty, Shelly uses `log`.

Examples:

| Identifier | Filename |
| --- | --- |
| `deploy` | `deploy-YYYY-MM-DD.log` |
| `team/release!*` | `team-release---YYYY-MM-DD.log` |
| `🔥` | `--YYYY-MM-DD.log` |

Here `🔥` sanitizes to `-`, then the date separator adds the second `-`. A missing identifier expression (`log`) is rejected before a filename is created; an explicit empty identifier (`log ""`) is accepted and uses the sanitizer's `log` fallback.

## Dynamic identifiers

```shelly
$target = "production"

log "deploy-$target"
    shellout "deploying $target"
end
```

The identifier can use variables and interpolation. This writes to a file like `logs/deploy-production-YYYY-MM-DD.log`.

## Recorded lines

Each line starts with a UTC timestamp. A block records lifecycle lines:

```text
2026-01-01T12:00:00Z --- deploy ---
2026-01-01T12:00:00Z ENTER
2026-01-01T12:00:00Z shellout: starting
2026-01-01T12:00:00Z command: printf ["running"]
2026-01-01T12:00:00Z EXIT ok
```

Shelly currently records selected action markers inside the block:

- `shellout: ...`
- `command: PROGRAM [ARGS]`
- file commands: `create`, `copy`, `move`, `rename`, `read`, `write`, path `remove`, and file `append`
- nested `log: ...` entry markers in the outer log

Collection mutations such as list or dictionary `append` and `remove` do not add action markers.

If an ordinary error happens, the log records `ERROR <message>` and the script still fails normally. Control signals such as `exit`, `break`, `continue`, and `return` are not swallowed; when practical, the log records an `EXIT control ...` line before propagating the signal.

## Nested log blocks

```shelly
log "deploy"
    shellout "deploy start"

    log "deploy-tests"
        command npm test
    end

    shellout "deploy end"
end
```

The outer log records a nested `log: deploy-tests` marker. The nested block writes its own `logs/deploy-tests-YYYY-MM-DD.log` file.

## Errors inside logs

```shelly
log "build"
    shellout "running tests"
    command false
end
```

The `build` log records an `ERROR ...` line, and the script still fails normally.

To handle the error and keep logging the recovery path, combine `log` with `try` / `catch`:

```shelly
try -> $error
    log "build"
        command false
    end
catch
    log "build-error"
        shellout "build failed: $error"
    end
end
```

## What is not captured

This first version does not capture full stdout or stderr from external commands. `shellout` and `command` output still goes to the terminal as usual. Logging levels are not configurable; the fixed markers above are recorded whenever their actions occur.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
