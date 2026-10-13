# Lists

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Lists are Shelly's single collection type.

```shelly
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2 + 3, 6]
$mixed = ["Ana", 3, true]
```

## Rules

- Empty list: `[]`.
- Lists can contain strings, numbers, booleans, variables, and constants.
- Nested lists are not supported yet.
- Indexing starts at `1`.
- `first` and `last` are supported selectors.
- Selectors can return one scalar or a new list.
- Mutations only work on variable lists.

## Indexing

```shelly
shellout $names[1]
shellout $names[first]
shellout $names[last]
shellout "second: $names[2]"
```

## Selectors

```shelly
shellout $names[1,3]
shellout $names[1..3]
shellout $names[..3]
shellout $names[3..]
shellout $names[..]
shellout $names[first..last]
```

Ranges are inclusive.

## Filters

```shelly
shellout $numbers[>5]
shellout $numbers[<=5]
shellout $names[=="Ana"]
shellout $names[!="Ana"]
```

Numeric filters support `==`, `!=`, `>`, `>=`, `<`, and `<=`. String and boolean filters support `==` and `!=`.

## Iteration

Use [`for`](for.md) to visit list values, constant lists, ranges, multi-selectors, and filters.

```shelly
for $name in $names[1..2]
    shellout $name
end
```

## Mutations

```shelly
$queue = []
append $queue, "Luis"
prepend $queue, "Ana"
$queue[2] = "Mora"
remove $queue[first]

shellout $queue[..]
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
