# Conditionals

Use `if`, optional `elif`, optional `else`, and `end`.

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

## Syntax

- `if condition` starts a conditional block.
- `elif condition` adds optional middle branches.
- `else` adds one optional fallback branch.
- `end` closes the block.
- Shelly uses no parentheses, no `then`, and no braces.

## Conditions

Direct truthy values are allowed:

```shelly
if $enabled
    shellout "enabled"
end

if "text"
    shellout "text is truthy"
end
```

Comparisons support:

- `==`
- `!=`
- `>`
- `>=`
- `<`
- `<=`

Condition values use normal value forms: strings, numbers, booleans, numeric expressions, variables, constants, and string interpolation.

## Truthiness

- `true` is true.
- `false` is false.
- `""` is false.
- `0` is false.
- Everything else is true.

## Scope and limitations

- Conditional blocks do not create a new scope.
- Assignments inside the selected branch affect the current scope.
- Complex expressions are not implemented yet: `and`, `or`, `not`, and parenthesized conditions are rejected.
