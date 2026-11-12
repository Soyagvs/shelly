# Iterate over a list with `for`

Use `for` to visit each value in an iterable list, in list order. It works with variables, constants, ranges, selectors, filters, and [`files`](files.md) results.

## 🚀 Quick path

```shelly
$names = ["Ana", "Luis", "Mora"]
for $name in $names
    shellout "Hello $name"
end
```

## 📖 Syntax and iterable rules

```text
for $item in ITERABLE
    ...
end
```

- The destination must be a mutable variable name.
- `ITERABLE` must evaluate to a list: a variable/constant, range, selector, filter, or `files` result.
- Bind list literals first; a literal directly after `in` is rejected.
- The iterable is evaluated at loop start and copied, so body changes do not change visited values.
- An empty list skips the body.

A scalar, dictionary, undefined variable, non-list value, malformed selector, or malformed filter is an error.

```shelly
$colors = ["red", "blue"]
for $color in $colors
    shellout $color
end
```

## 🔢 Ranges, selectors, and filters

Selectors are one-based. Ranges are inclusive and may have an open endpoint. Multi-selectors, ranges, and filters produce lists; a single selector such as `$numbers[2]` produces one value and cannot be iterated.

```shelly
$numbers = [1, 3, 5, 8]
for $number in $numbers[2..3]
    shellout $number
end
for $number in $numbers[>3]
    shellout $number
end
for $number in $numbers[first..last]
    shellout $number
end
```

Indexes must be positive, existing integers; ranges must be ordered and within the list. Filters support `==`, `!=`, `>`, `>=`, `<`, and `<=`. Equality filters exclude non-scalars; ordered filters require numeric items. See [Lists](lists.md) for complete selector and filter rules.

## 🧾 Scope and values

The loop variable is an ordinary mutable variable. A non-empty loop leaves it set to its last value, and body assignments can affect the current iteration. A zero-iteration loop leaves an existing destination unchanged. Constants can be iterated but cannot be destinations or reassigned.

```shelly
$values = ["one", "two"]
$item = "before"
for $item in $values
    shellout $item
end
# $item is "two"
```

Variables assigned in the body persist as in [`repeat`](repeat.md) and [`while`](while.md). In a function, the destination follows normal function scope rules: it is local unless `global` is used. See [Variables](variables.md) and [Functions](functions.md).

Nested lists and dictionaries are preserved, not flattened or converted:

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

## 📁 Iterate over files

`files` writes a sorted list of matching script-relative file paths. No matches produce an empty list; directories are never returned.

```shelly
files "**/*.txt" -> $items
for $file in $items
    shellout "found $file"
end
```

See [Files](files.md) for patterns and paths.

## 🛠️ Control flow and errors

`break` exits and `continue` skips within the nearest loop. Both are errors outside a loop. Nested `if`, `repeat`, `while`, and `for` blocks are supported.

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

Iterable evaluation, selector validation, and the body can produce runtime errors. Use [`try`/`catch`](errors.md) for expected recoverable failures. Control signals are not ordinary errors and are not caught by `try`; [`defer`](defer.md) still runs cleanup. Common errors include undefined/non-list iterables, direct literals, invalid indexes/ranges, incompatible filters, and control statements outside loops.

## Related guides

- [Lists](lists.md) — selectors, ranges, filters, and nested values.
- [Files](files.md) — sorted script-relative paths.
- [Variables](variables.md) and [Functions](functions.md) — assignment and scope.
- [Repeat loops](repeat.md) and [While loops](while.md) — other loop forms.
- [Error handling](errors.md) and [Deferred cleanup](defer.md) — failures and cleanup.
