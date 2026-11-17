# Measure elapsed time

`measure -> $milliseconds` runs a block and records its elapsed time in monotonic milliseconds:

```shelly
measure -> $milliseconds
    command sleep 0.01
end
shellout "elapsed: $milliseconds ms"
```

The clock is monotonic, so elapsed values do not move backward when the system
clock changes. The destination is assigned only after the body succeeds. If a
command or nested block fails, the failure propagates and the destination is
not assigned. `measure` does not capture or suppress visual output: `shellout`,
tables, and other visual producers remain visible, while silent operations
remain silent. The elapsed-time result remains the measure block's own result.

## Any block or function

A measure block can contain ordinary Shelly statements, control flow, tables,
and other supported blocks. It also works inside a function:

```shelly
fn report($items) {
    measure -> $elapsed
        table border ascii
            column "Item"
            for $item in $items
                if $item != "skip"
                    row $item
                end
            end
        end
    end
    shellout "built in $elapsed ms"
}

call report(["one", "skip", "two"])
```

Syntax is exactly `measure -> $destination`, followed by a body and `end`.
The destination must be a mutable variable; missing, malformed, constant, or
non-variable destinations are errors. The value is an integer number of
milliseconds. There is no precision or duration guarantee beyond the
monotonic clock and the practical limits of the host clock and integer value.

## Related guides

- [Terminal tables](tables.md) — tables, dynamic rows, and alignment.
- [Functions](functions.md) — function scope and returns.
- [Error handling](errors.md) — ordinary runtime failures.
- [Formatting](formatting.md) — formatting nested blocks.
- [LSP reference](lsp.md) — editor diagnostics and formatting.
