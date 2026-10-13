# For loops

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

`for` iterates over values in a list, constant list, range, selector, filter, or list returned by [`files`](files.md).

```shelly
$names = ["Ana", "Luis", "Mora"]

for $name in $names
    shellout "Hello $name"
end
```

## Rules

- `for $item in iterable` starts the loop and `end` closes it.
- The destination must be a variable.
- The iterable must evaluate to a list.
- Values are visited in list order.
- Empty lists skip the block.
- After a non-empty loop, the loop variable remains set to the last iterated value.
- `break` exits the nearest loop.
- `continue` skips to the next iteration.
- Nested `if`, `repeat`, `while`, and `for` blocks are supported.

## Constants

```shelly
$$NAMES = ["Ana", "Luis"]

for $name in $$NAMES
    shellout $name
end
```

## Files

```shelly
files "*.txt" -> $items
for $file in $items
    shellout $file
end
```

`files` sorts matches and returns an empty list when nothing matches.

## Selectors and filters

```shelly
$numbers = [1, 3, 5, 8]

for $number in $numbers[2..3]
    shellout $number
end

for $number in $numbers[>3]
    shellout $number
end
```

## Nesting

```shelly
$groups = ["A", "B"]

for $group in $groups
    repeat 2 -> $turn
        shellout "$group:$turn"
    end
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
