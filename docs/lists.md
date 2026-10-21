# Lists

A list is an ordered, one-based collection of values. Values may be strings, numbers, booleans, variables, constants, dictionaries, or other lists. A list is written with brackets and comma-separated values; an empty list is `[]`.

```shelly
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2 + 3, 6]
$mixed = ["Ana", 3, true]
$nested = [[1, 2], { name: "Ada" }]
```

Assigning a list copies its value; it does not create an alias. Later changes to one variable do not change the other.

```shelly
$first = ["a"]
$second = $first
append $first "b"
# $second is still ["a"]
```

Lists are values: they can be assigned, passed to functions, printed, tested in conditions, and iterated. See [Variables](variables.md), [Constants](constants.md), [Dictionaries](dictionaries.md), [Conditionals](conditionals.md), [For loops](for.md), and [JSON helpers](json.md).

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Indexes and selectors

Indexes start at `1`. `first` and `last` select the first and last item. A single selector returns one value; a multi-selector or range returns a new list. Ranges are inclusive. Either endpoint may be open.

```shelly
shellout $names[1]
shellout $names[first]
shellout $names[last]
shellout $names[1,3]
shellout $names[1..3]
shellout $names[..3]
shellout $names[3..]
shellout $names[..]
shellout $names[first..last]
```

Indexes must be positive integers that already exist. Zero, negative, fractional, non-numeric, and out-of-range indexes are errors. A range whose endpoints are invalid, reversed, or outside the list is also an error; an open range uses the list boundary. Selectors are read-only expressions and do not support assignment.

**Invalid examples:** `$names[0]`, `$names[-1]`, `$names[1.5]`, and `$names[1..99]`.

## Filters

A filter keeps values matching a comparison. Numeric lists support `==`, `!=`, `>`, `>=`, `<`, and `<=`; string and boolean lists support `==` and `!=`. The right-hand side is one comparison value, not a selector or a list.

```shelly
$numbers = [1, 3, 5, 8]
shellout $numbers[>5]
shellout $numbers[<=5]
shellout $names[=="Ana"]
shellout $names[!="Ana"]
```

Filters are for scalar lists. Comparisons against dictionaries or lists, mixed incomparable values, and nested-item filtering are rejected. Filters return a new list and never mutate the source.

## Iteration

Use [`for`](for.md) to visit values in order. The iterable may be a list variable or constant, a range, selector, filter, or list returned by [`files`](files.md). An empty list skips the body.

```shelly
for $name in $names[1..2]
    shellout $name
end
```

## Mutation

Only a mutable, lowercase variable can be mutated. Constants and values produced directly by an expression cannot be mutation targets. Mutation commands take whitespace-separated arguments; comma-separated command syntax is invalid.

Inserted and replaced list values must be scalar (string, number, or boolean). Dictionaries and lists may be stored in a list literal and addressed through nested paths, but cannot be inserted or used as replacement values by list mutation. Paths may traverse nested dictionaries and lists; every intermediate segment must already exist and have the required collection type. Lists never auto-expand.

```shelly
$queue = ["Luis"]
append $queue "Mora"
prepend $queue "Ana"
$queue[2] = "Lee"
remove $queue[first]
set $queue[1] "Ada"
delete $queue[1]
```

### Mutation forms

| Form | Effect and limits |
| --- | --- |
| `append PATH VALUE` | Adds scalar `VALUE` to the end of the list at `PATH`. The path may be nested. |
| `prepend PATH VALUE` | Adds scalar `VALUE` to the beginning of a **top-level mutable scalar list**. Nested paths and nested-item lists are rejected. |
| `remove PATH[SELECTOR]` | Removes the selected item from a list and compacts it. The selected item must exist; use a one-item selector for one removal. |
| `delete PATH[INDEX]` | Deletes the indexed item from a list and compacts it. Nested paths are allowed; the index must exist. |
| `set PATH[INDEX] VALUE` | Replaces an existing list item with scalar `VALUE`; nested paths are allowed. It never creates an index. |
| `PATH[INDEX] = VALUE` | Bracket replacement, equivalent to `set`; the index must exist and `VALUE` must be scalar. |

Dictionary paths can be combined with list paths:

```shelly
$data = { items: [["a"]] }
set $data.items[1][1] "b"
append $data.items[1] "c"
delete $data.items[1][1]
shellout $data.items[1]
```

The following are **invalid examples**: `append $CONSTANT "x"` (constants are immutable), `prepend $data.items "x"` (prepend is top-level only), `set $items[4] "x"` (no auto-expansion), `set $items[1] ["x"]` (replacement must be scalar), and `append $items "x", "y"` (comma-separated mutation arguments are unsupported).

Nested paths are allowed for generic collection mutation, but mutation still requires a mutable root. A path through a constant cannot be changed.

## Rejected forms at a glance

| Rejected form | Why |
| --- | --- |
| `list[0]`, negative, fractional, or missing index | Indexes are one-based existing integers. |
| Reversed or out-of-range range | Range endpoints must resolve within the list. |
| Filter on a dictionary/list item or incompatible RHS | Filters compare scalar values only. |
| `prepend $nested.path VALUE` | `prepend` only targets a top-level mutable scalar list. |
| `set`/replacement with a list or dictionary value | Inserted and replaced list values must be scalar. |
| Mutation of a constant or expression | Only mutable variables are targets. |
| `append $list a, b` | Commands do not use comma-separated arguments. |
| Nested-item `prepend`, filtering, or scalar-list helpers | Nested items are not scalar-list inputs. |

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Lists can contain dictionaries and other lists, and access chains can mix brackets and dictionary dots:

```shelly
$users = [{ name: "Ada" }, { name: "Luis" }]
$matrix = [[1, 2], [3, 4]]
shellout $users[1].name
shellout $matrix[2][1]
```

Direct indexing, ranges, selectors, and `for` preserve nested values. Scalar-list filters and the scalar-only mutation value rules above still apply. Use [Dictionaries](dictionaries.md) for dictionary keys and nested dictionary mutation.
