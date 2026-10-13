# Conditionals

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Use `if`, optional `elif`, optional `else`, and `end` to choose which block runs.

```shelly
$enabled = true
$count = 3

if $enabled
    shellout "enabled"
elif $count >= 10
    shellout "large"
else
    shellout "fallback"
end
```

## Rules

- `if condition` starts a conditional block.
- `elif condition` adds an optional middle branch.
- `else` adds one optional fallback branch.
- `end` closes the block.
- Conditions use truthiness or simple comparisons.
- Blocks do not create a new scope.
- Nested `if`, `repeat`, `while`, and `for` blocks are supported.
- Logical operators `and`, `or`, and `not` are not implemented yet.
- Parenthesized condition expressions are not implemented yet.

## Comparisons

```shelly
if $count >= 3
    shellout "enough"
end
```

Supported operators: `==`, `!=`, `>`, `>=`, `<`, and `<=`.

## Truthiness

- `true` is true.
- `false` is false.
- `""` is false.
- `0` is false.
- Non-empty lists are true.
- Empty lists are false, including empty selector/filter results.
- Everything else is true.

```shelly
$numbers = [1, 3, 5]

if $numbers[>10]
    shellout "has big numbers"
else
    shellout "no big numbers"
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
