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
- Counts must resolve to non-negative integers.
- `repeat 0` skips the block.
- `repeat count -> $var` binds the current 1-based iteration number.
- The optional index binding is local/protected inside the loop.
- Other assignments inside the loop persist after the loop.
- Nested `if`, `repeat`, `while`, and `for` blocks are supported.

## Index binding

```shelly
repeat 3 -> $turn
    shellout "turn $turn"
end
```

## Numeric counts

```shelly
$extra = 1
$$BASE = 2

repeat $$BASE + $extra
    shellout "again"
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
