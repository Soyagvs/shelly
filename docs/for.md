# For loops

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`for` visits each value in an iterable list, in list order. It is useful for lists held in variables or constants, ranges and selectors, filters, and lists returned by [`files`](files.md).

```shelly
$names = ["Ana", "Luis", "Mora"]

for $name in $names
    shellout "Hello $name"
end
```

## Syntax and iterable rules

```text
for $item in ITERABLE
    ...
end
```

- The destination after `for` must be a mutable variable name (lowercase, such as `$item`).
- `ITERABLE` must be a variable or constant whose value is a list, or a range, selector, filter, or `files` result that evaluates to a list.
- A list literal is **not** accepted directly as the iterable. Bind it first:

  ```shelly
  $colors = ["red", "blue"]
  for $color in $colors
      shellout $color
  end
  ```

- The iterable is evaluated when the loop starts. Values are copied into the loop sequence; changing a source variable in the body does not change which values are visited.
- An empty list skips the body and performs no iterations.

A scalar, dictionary, undefined variable, or other non-list value is an error. A malformed selector or filter is also reported before iteration begins.

## Ranges, selectors, and filters

Selectors use one-based indexes. A range is inclusive and may have an open endpoint. A multi-selector, range, or filter produces a new list and can therefore be iterated:

```shelly
$numbers = [1, 3, 5, 8]

for $number in $numbers[2..3]
    shellout $number       # 3, then 5
end

for $number in $numbers[>3]
    shellout $number       # 5, then 8
end

for $number in $numbers[first..last]
    shellout $number
end
```

A single selector such as `$numbers[2]` returns one value, not a list, so it is not an iterable. Indexes must be positive, existing integers; ranges must be in order and within the list. Filters support `==` and `!=` for equality, plus `>`, `>=`, `<`, and `<=` for ordered numeric comparisons. Equality filters exclude non-scalar items from the result; ordered filters require numeric items and error when they encounter non-numeric or nested items. See [Lists](lists.md) for the complete selector and filter rules.

## Scope and persistence

The loop variable is an ordinary mutable variable, not a temporary index binding. A non-empty loop leaves it set to the last value visited, and assignments to it in the body can affect the current iteration:

```shelly
$values = ["one", "two"]
$item = "before"
for $item in $values
    shellout $item
end
# $item is "two"
```

The list must be bound before the loop; a literal cannot appear directly after `in`. Variables assigned in the body otherwise persist after the loop, just as they do in [`repeat`](repeat.md) and [`while`](while.md). A zero-iteration loop leaves an existing destination unchanged; it does not create a useful last value. In a function, the destination follows normal function scope rules: it is local unless the function explicitly uses `global` for an outer variable. See [Variables](variables.md) and [Functions](functions.md).

Constants can be iterated but cannot be used as the destination or reassigned in the body:

```shelly
$COLORS = ["red", "blue"]
for $color in $COLORS
    shellout $color
end
```

## Nested values

Lists may contain dictionaries and other lists. `for` preserves each value; it does not flatten or convert nested values:

```shelly
$users = [{ name: "Ada" }, { name: "Luis" }]
for $user in $users
    shellout $user.name
end

$rows = [[1, 2], [3, 4]]
for $row in $rows
    shellout $row[1]
end
```

Use [Dictionaries](dictionaries.md) for nested keys and [Lists](lists.md) for access chains and mutation rules.

## Files composition

`files` writes a sorted list of matching file paths into a variable. No matches produce an empty list, so the loop simply does nothing:

```shelly
files "**/*.txt" -> $items
for $file in $items
    shellout "found $file"
end
```

Patterns and paths are resolved relative to the script file. `files` returns files only, never directories; see [Files](files.md).

## Control flow and nesting

`break` exits the nearest loop. `continue` skips the remainder of the current iteration and advances the nearest loop. Both are errors outside a loop. Nested `if`, `repeat`, `while`, and `for` blocks are supported; control flow applies only to the nearest enclosing loop.

```shelly
$groups = ["A", "B"]
$turns = [1, 2]

for $group in $groups
    for $turn in $turns
        if $turn == 1
            continue
        end
        shellout "$group:$turn"
    end
end
```

## Errors and recovery

Iterable evaluation, selector validation, and the loop body can all produce ordinary runtime errors. Use [`try`/`catch`](errors.md) when the failure is expected and recoverable. `break` and `continue` are control signals, not ordinary errors, and are not caught by `try`; [`defer`](defer.md) still runs cleanup when a loop leaves a scope through an error or control signal.

Common errors include an undefined or non-list iterable, a literal used directly after `in`, an invalid index or range, an incompatible filter, and `break` or `continue` outside a loop.

## Related guides

- [Lists](lists.md) — list values, selectors, ranges, filters, and nested values.
- [Files](files.md) — collecting sorted script-relative paths.
- [Variables](variables.md) and [Functions](functions.md) — assignment and scope.
- [Repeat loops](repeat.md) and [While loops](while.md) — other loop forms.
- [Error handling](errors.md) and [Deferred cleanup](defer.md) — failures and cleanup.
- [Language Guide](language.md) — syntax overview and reference map.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
