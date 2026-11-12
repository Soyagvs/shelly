# Trace a script block with logs

Use a `log` block to record structured lifecycle and action markers while normal terminal output remains visible.

## Quick path

```shelly
log "deploy"
    shellout "starting"
    command printf running
end
```

The identifier is required and is evaluated like other expressions. Nested blocks are allowed, and `shelly check` validates structure without running it.

## Log files

Shelly creates `logs/` beside the running script and writes one UTC-dated file per identifier:

```text
logs/<sanitized-identifier>-YYYY-MM-DD.log
```

Sanitization keeps ASCII letters, digits, `_`, and `-`; every other character becomes `-`. An empty sanitized name becomes `log`. Thus `team/release!*` becomes `team-release---YYYY-MM-DD.log`, while `log ""` uses `log`.

## Recorded markers

Each line begins with a UTC timestamp. Entries include lifecycle lines such as `ENTER` and `EXIT ok`, plus selected actions:

- `shellout: ...`
- `command: PROGRAM [ARGS]`
- file actions: `create`, `copy`, `move`, `rename`, `read`, `write`, path `remove`, and file `append`
- nested `log: ...` entry markers

List and dictionary mutations do not add action markers. Nested logs write their own files. Ordinary errors record `ERROR <message>` and still fail normally; control signals such as `exit`, `break`, `continue`, and `return` are not swallowed.

## Errors and limits

A missing identifier expression (`log`) is rejected; an explicit empty string is accepted. Use [try/catch](errors.md) to log recovery separately:

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

Logging does not capture complete stdout or stderr from external commands. Levels are not configurable, and recorded markers are fixed. See [Secrets](secrets.md) for redaction behavior when secret values appear in messages.
