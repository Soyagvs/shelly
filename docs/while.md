# While loops

`while` repeats a block while a condition is true.

```ly
$count = 0
while $count < 3
    shellout "Count $count"
    $count = $count + 1
end
shellout "Done at $count"
```

## Conditions

`while` uses the same condition rules as [`if`](conditionals.md):

- truthiness
- comparisons with `==`, `!=`, `>`, `>=`, `<`, and `<=`
- normal value forms including variables, constants, strings, booleans, and numeric expressions where condition handling supports them

```ly
$enabled = true
while $enabled
    shellout "once"
    $enabled = false
end
```

## Rules

- If the condition is false before the first iteration, the body is skipped.
- While blocks do not create a new scope.
- Assignments inside the loop affect the current scope and persist after the loop.
- To guard against accidental infinite loops, a while loop errors after 10,000 iterations.
- Nested `if`, `repeat`, and `while` blocks are supported.
