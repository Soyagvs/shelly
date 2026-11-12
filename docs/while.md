# Repeat while a condition is true

Use `while` when the next iteration depends on a condition evaluated before the loop body.

## 🚀 Quick path

```shelly
$count = 0

while ($count < 3)
    shellout "Count $count"
    $count = $count + 1
end

shellout "Done at $count"
```

## 📖 Conditions and rules

A `while` condition follows the same rules as an [if condition](conditionals.md): truthiness, comparisons, predicates, `not`, `and`, `or`, and nested parentheses. Conditions short-circuit. Selectors can also produce a truthy or falsey result.

```shelly
while ($enabled and not contains $blocked $name)
    if matches $name /(?i)^[a-z]+$/
        shellout "valid: $name"
    end
    $enabled = false
end
```

The condition is checked before every iteration; a false initial condition skips the body. The loop has no loop-variable binding and creates no scope, so assignments persist. Nested control-flow blocks are supported.

```shelly
$outer = 0
while $outer < 2
    $outer = $outer + 1
    while true
        break
    end
    continue
end
shellout "$outer" # 2
```

`break` exits the nearest loop and `continue` advances its next iteration. Both are errors outside a loop.

## ⏱️ Iteration limit

A loop may execute at most 10,000 iterations. The 10,000th is allowed; attempting an 10,001st iteration stops with an iteration-limit error before its body runs. `break` can end the loop earlier.

## ✅ More examples

```shelly
$enabled = true
while $enabled
    shellout "once"
    $enabled = false
end
```

```shelly
$numbers = [1, 3, 5]
$limit = 2
while $numbers[>$limit]
    shellout "has values above $limit"
    $limit = $limit + 10
end
```

## ⚠️ Errors

An empty condition, empty `()`, unmatched parentheses, malformed text, missing predicate arguments, unterminated regex, invalid regex, or missing `end` is rejected.

```text
while                 # empty condition
while ($count < 3    # unmatched group
while ($count < )    # malformed condition
while contains $x    # missing predicate argument
while $ready
    shellout "missing end"
# missing end

break                 # outside a loop
continue              # outside a loop
```

## Related guides

- [Conditionals](conditionals.md) and [Predicates](predicates.md) — condition syntax and checks.
- [Repeat loops](repeat.md) and [For loops](for.md) — other loop forms.
