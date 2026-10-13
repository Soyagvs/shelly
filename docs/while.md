# While loops

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`while` repeats a block while a condition is true.

```shelly
$count = 0

while $count < 3
    shellout "Count $count"
    $count = $count + 1
end

shellout "Done at $count"
```

## Rules

- `while condition` starts the loop and `end` closes it.
- Conditions use the same rules as [conditionals](conditionals.md).
- If the condition is false before the first iteration, the body is skipped.
- Assignments inside the loop persist after the loop.
- `break` exits the nearest loop.
- `continue` skips to the next iteration.
- A safety limit stops accidental infinite loops after 10,000 iterations.
- Nested `if`, `repeat`, `while`, and `for` blocks are supported.

## Truthy loop

```shelly
$enabled = true

while $enabled
    shellout "once"
    $enabled = false
end
```

## List filter condition

```shelly
$numbers = [1, 3, 5]
$limit = 2

while $numbers[>$limit]
    shellout "has values above $limit"
    $limit = $limit + 10
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
