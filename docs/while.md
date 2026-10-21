# While loops

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`while` repeats a block while its condition is true.

```shelly
$count = 0

while ($count < 3)
    shellout "Count $count"
    $count = $count + 1
end

shellout "Done at $count"
```

## Conditions

A `while` condition follows the same rules as an [if condition](conditionals.md):

- A value can be tested for truthiness, or compared with `==`, `!=`, `>`, `>=`, `<`, or `<=`.
- `not`, `and`, and `or` are supported and short-circuit in that order.
- Parentheses group expressions and may be nested.
- Predicates include `exists`, `contains`, and `matches`; selectors can also produce a truthy or falsey result.

```shelly
while ($enabled and not contains $blocked $name)
    if matches $name /(?i)^[a-z]+$/
        shellout "valid: $name"
    end
    $enabled = false
end
```

An empty condition, empty `()`, unmatched parentheses, malformed text, missing predicate arguments, unterminated regex, or invalid regex is rejected.

## Rules and scope

- `while condition` starts the loop; `end` closes it.
- The condition is checked before every iteration. If it is false initially, the body is skipped.
- A `while` loop has no loop-variable binding and creates no scope. Assignments made in its body persist after the loop.
- `break` exits the nearest loop; `continue` skips to the next iteration of the nearest loop.
- `if`, `repeat`, `while`, and `for` blocks may be nested. A nested loop's control statements affect only that nested loop.

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

A loop may execute at most 10,000 iterations. The 10,000th iteration is allowed; if the condition would start an 10,001st iteration, execution stops with an iteration-limit error rather than running that body. `break` can end the loop before the limit.

## Examples

### Truthy loop

```shelly
$enabled = true

while $enabled
    shellout "once"
    $enabled = false
end
```

### Predicate and selector condition

```shelly
$numbers = [1, 3, 5]
$limit = 2

while $numbers[>$limit]
    shellout "has values above $limit"
    $limit = $limit + 10
end
```

## Invalid forms

These forms are rejected:

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

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
