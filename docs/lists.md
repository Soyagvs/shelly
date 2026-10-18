# Lists

Nested collections can be mutated with `set PATH VALUE`, `append PATH VALUE`, and `delete PATH`; top-level scalar lists also support `prepend $list VALUE`. Arguments are whitespace-separated, so comma-separated mutation syntax is unsupported. List indexes are one-based and must already exist: `set $data.items[2] "x"` replaces an element, `append $data.items "x"` adds one, and `delete $data.items[1]` compacts the list. Lists never auto-expand; zero, negative, non-integer, and out-of-range indexes fail. Collection assignment copies values rather than creating aliases. Nested `prepend`, `insert`, slices, and range replacement remain deferred.

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
- Lists can contain strings, numbers, booleans, variables, constants, dictionaries, and other lists.
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

Use [`for`](for.md) to visit list values, constant lists, ranges, multi-selectors, filters, and lists returned by [`files`](files.md).

```shelly
for $name in $names[1..2]
    shellout $name
end
```

## Mutations

```shelly
$queue = []
append $queue "Luis"
prepend $queue "Ana"
$queue[2] = "Mora"
remove $queue[first]

shellout $queue[..]
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Dictionaries and lists

Dictionary values can contain lists, and lists can contain dictionaries:

```shelly
$user = { name: "Ada", tags: ["admin", "ops"] }
$users = [{ name: "Ada" }, { name: "Luis" }]

shellout $user.tags
shellout $users[1].name
```

## Nested list values

Lists can contain scalars, dictionaries, or other lists. Access chains can mix list brackets and dictionary dots.

```shelly
$users = [{ name: "Ada", role: "admin" }, { name: "Luis", role: "user" }]
$matrix = [[1, 2], [3, 4]]
shellout $users[1].name
shellout $matrix[2][1]
shellout "name=$users[2].name"
```

Selectors, ranges, `for`, and direct indexing preserve nested values. `join`, `sort`, `reverse`, `unique`, filters, and list mutations are scalar-list features; nested items are rejected with a clear scalar-list or mutation error.
