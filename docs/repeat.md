# Repeat loops

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`repeat` runs a block a fixed number of times.

```shelly
repeat 3
    shellout "hello"
end
```

## Rules

- `repeat count` starts the loop and `end` closes it.
- The count is a numeric expression. A non-negative integer literal, mutable variable, constant, or arithmetic expression is accepted; for example, `repeat 3`, `repeat $BASE`, and `repeat $BASE + 1`.
- Counts must resolve to non-negative integers. `repeat 0` is valid and skips the body.
- `repeat count -> $var` optionally binds the current iteration number.
- Assignments to ordinary variables inside the loop persist after the loop.
- `break` exits the nearest enclosing loop; `continue` skips to the next iteration of the nearest enclosing loop.
- Nested `if`, `repeat`, `while`, and `for` blocks are supported.

## Index binding

The index is **1-based**: it is set to `1` at the start of the first iteration and increases by one at the start of each later iteration. The binding is temporary: the loop saves the variable before entering, sets it for each iteration, and restores the saved value after the loop. An assignment to that variable in the body can affect the remainder of the current iteration, but is replaced by the next iteration's index and never escapes the loop. Restoration also occurs for a zero-iteration loop, `break`, or an error; if the variable did not exist before the loop, it is removed afterward.

```shelly
$turn = "outside"
repeat 3 -> $turn
    shellout "turn $turn"
end
shellout "after=$turn"
```

This prints `turn 1`, `turn 2`, `turn 3`, then `after=outside`. Nested loops can use distinct bindings; each binding is restored independently:

```shelly
repeat 2 -> $row
    repeat 2 -> $column
        shellout "$row,$column"
    end
end
```

## Numeric counts

Counts may use numeric variables, constants, and arithmetic expressions:

```shelly
$extra = 1
$BASE = 2

repeat $BASE + $extra
    shellout "again"
end
```

The expression must be complete and must evaluate to an integer. Negative, fractional, missing, and malformed counts are rejected. These are invalid examples (shown as text, not runnable Shelly):

```text
repeat -1       # negative count
repeat 1.5       # fractional count
repeat $missing  # unresolved count variable
repeat           # missing count
repeat 2 +       # malformed expression
repeat 2 words   # trailing text
```

## Control flow and errors

`break` and `continue` affect only the nearest loop, including when loops are nested. A `break` in an inner loop leaves that inner loop and execution continues in the outer loop; a `continue` advances only the inner loop. Using either statement when no loop is active is an error (`break outside loop` or `continue outside loop`).

```shelly
repeat 2 -> $outer
    repeat 3 -> $inner
        if $inner == 2
            continue
        end
        if $outer == 2
            break
        end
        shellout "$outer,$inner"
    end
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
