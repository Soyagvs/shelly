# Conditionals

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Use `if`, optional `elif`, optional `else`, and `end` to choose which block runs. Use `case` when one value should dispatch to exact matching branches.

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
- Conditions use truthiness, simple comparisons, predicates, and logical operators.
- `not` binds before `and`; `and` binds before `or`.
- Blocks do not create a new scope.
- Nested `if`, `repeat`, `while`, `for`, and `try` blocks are supported.
- Parentheses group condition expressions and can override logical precedence.
- `case VALUE` compares one evaluated value against one or more exact `when` values.
- `case` requires at least one `when`; `else` is optional and must be last.

## Case blocks

Use `case` for exact value dispatch without fallthrough. The first matching `when` branch runs; no `break` is needed. If no `when` matches and there is no `else`, nothing runs.

```shelly
$mode = "prod"

case $mode
    when "dev", "test"
        shellout "non-production"
    when "prod"
        shellout "production"
    else
        shellout "unknown mode"
end
```

`case` values and `when` values can be strings, numbers, booleans, constants, variables, or numeric expressions. Blocks do not create a new scope, and nested blocks work inside branches.

```shelly
$$LIMIT = 3
$count = 3

case $count
    when 1
        shellout "one"
    when $$LIMIT
        if true
            shellout "at limit"
        end
end
```

Rules:

- `case VALUE` requires a value.
- `when VALUE` requires at least one value.
- `when VALUE, VALUE` matches if any listed value equals the `case` value.
- `else` may appear once and must come after all `when` branches.
- `when` outside `case`, duplicate `else`, `when` after `else`, missing `end`, and `case` with no `when` are errors.

## Comparisons

```shelly
if $count >= 3
    shellout "enough"
end
```

Supported comparison operators: `==`, `!=`, `>`, `>=`, `<`, and `<=`.

## Logical operators

Use `not`, `and`, and `or` to compose conditions. Priority is:

1. `not`
2. `and`
3. `or`

Use parentheses to group condition expressions when you need a different order or clearer logic. Groups can nest, and `not (condition)` is supported.

### `not`

```shelly
if not false
    shellout "true"
end
```

### `and`

```shelly
if $count > 0 and $count < 10
    shellout "small positive number"
end
```

### `or`

```shelly
if $mode == "dev" or $mode == "test"
    shellout "non-production"
end
```

### Grouping

```shelly
if ($mode == "dev" or $mode == "test") and not $disabled
    shellout "enabled non-production"
end

if not (exists "dist/app" and contains $mode "prod")
    shellout "needs attention"
end
```

### Combined predicates

```shelly
$names = ["Ana", "Luis"]

if exists "README.md" and contains $names "Luis"
    shellout "ready"
end

if not exists "dist/app" or contains $mode "dev"
    shellout "build needed"
end
```

## Predicates

Use `exists "path"` to check for a file or directory relative to the current script file. Missing paths return false instead of erroring.

Use `contains $value "thing"` for substring checks on strings and exact item checks on lists.

See [predicates.md](predicates.md) for capture forms, composition examples, and predicate-specific notes.

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
