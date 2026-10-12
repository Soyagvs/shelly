# Repeat loops

`repeat` runs a block a fixed number of times.

```ly
repeat 3
    shellout "hello"
end
```

## Optional index binding

Use `->` to bind the current 1-based iteration index into a local protected variable.

```ly
repeat 3 -> $turn
    shellout "turn $turn"
end
```

## Rules

- `repeat count` starts the loop and `end` closes it.
- Counts must resolve to non-negative integers.
- `repeat 0` skips the block.
- The optional index starts at `1`.
- The optional index is local/protected inside the loop.
- `repeat` does not create a general new scope for other assignments.
- Nested `if`, `repeat`, and `while` blocks are supported.
