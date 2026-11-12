# Repeat a block a fixed number of times

Use `repeat` for a non-negative integer count. The optional binding exposes a temporary 1-based iteration number.

## 🚀 Quick path

```shelly
repeat 3
    shellout "hello"
end
```

## 📖 Syntax and rules

```shelly
repeat COUNT
    ...
end

repeat COUNT -> $index
    ...
end
```

`COUNT` may be a numeric variable, constant, or arithmetic expression. It must resolve to a non-negative integer; `repeat 0` skips the body. Assignments to ordinary variables persist after the loop. Nested `if`, `repeat`, `while`, and `for` blocks are supported.

The count must be complete and valid. Negative, fractional, missing, and malformed counts are rejected:

```text
repeat -1       # negative count
repeat 1.5      # fractional count
repeat $missing # unresolved variable
repeat          # missing count
repeat 2 +      # malformed expression
repeat 2 words  # trailing text
```

## 🔢 Index binding

The index is 1-based. Shelly saves the destination before the loop, sets it for each iteration, and restores it afterward—including zero iterations, `break`, or an error. If it did not exist before the loop, it is removed. An assignment in the body affects only the remainder of that iteration.

```shelly
$turn = "outside"
repeat 3 -> $turn
    shellout "turn $turn"
end
shellout "after=$turn"
```

This prints `turn 1`, `turn 2`, `turn 3`, then `after=outside`. Nested loops can use distinct bindings and restore each independently.

```shelly
repeat 2 -> $row
    repeat 2 -> $column
        shellout "$row,$column"
    end
end
```

## 🛠️ Control flow and errors

`break` exits the nearest loop; `continue` skips to its next iteration. In nested loops, each affects only the nearest loop. Either statement outside a loop is an error.

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

## Related guides

- [While loops](while.md) and [For loops](for.md) — condition- and collection-based loops.
- [Conditionals](conditionals.md) — branch inside a loop.
- [Lists](lists.md) — values for collection-based iteration.
