# Work with ordered lists

Lists are ordered, one-based collections of values. This guide covers construction, selectors, filters, iteration, copying, and mutation.

## 🚀 At a glance

```shelly
$names = ["Ana", "Luis", "Mora"]
shellout $names[1]
for $name in $names
    shellout $name
end
```

- Values may be scalars, dictionaries, lists, variables, or constants.
- Indexes start at `1`; `first` and `last` are also available.
- Selectors and filters return values or new lists; they do not mutate the source.
- Only a mutable lowercase variable can be a mutation root.

## Create and copy lists

```shelly
$numbers = [1, 2 + 3, 6]
$mixed = ["Ana", 3, true]
$nested = [[1, 2], { name: "Ada" }]
```

An empty list is `[]`. Assigning a list copies its value rather than creating an alias:

```shelly
$first = ["a"]
$second = $first
append $first "b"
# $second is still ["a"]
```

Lists can be assigned, passed to functions, printed, tested, and iterated. See [Variables](variables.md), [Constants](constants.md), [Dictionaries](dictionaries.md), [Conditionals](conditionals.md), and [For loops](for.md).

## Indexes and selectors

Indexes are positive, existing, one-based integers. A single selector returns one value; a multi-selector or inclusive range returns a new list. Open range endpoints use the list boundary.

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

Zero, negative, fractional, nonnumeric, and out-of-range indexes fail. Reversed or invalid ranges fail. Selectors are read-only; they cannot be assignment targets.

## Filters and iteration

Filters compare scalar list values. Numeric lists support `==`, `!=`, `>`, `>=`, `<`, `<=`; string and boolean lists support `==` and `!=`. The right side is one comparison value.

```shelly
$numbers = [1, 3, 5, 8]
shellout $numbers[>5]
shellout $numbers[<=5]
shellout $names[=="Ana"]
shellout $names[!="Ana"]
```

Filters reject dictionaries, lists, mixed incomparable values, and nested-item filtering. Use [`for`](for.md) to visit values in order; an empty list skips the body:

```shelly
for $name in $names[1..2]
    shellout $name
end
```

## Mutation rules

Only a mutable lowercase variable can be mutated. Constants and expression results cannot be targets. Intermediate dictionary keys and list indexes must already exist; lists never auto-expand.

| Form | Value support | Rule |
| --- | --- | --- |
| `set $dict.key VALUE` | Scalar, list, dictionary | Creates or replaces a terminal dictionary value. |
| `set $list[INDEX] VALUE` | Scalar, list, dictionary | Replaces an existing item. |
| `append $list VALUE` | Scalar or dictionary | Root append rejects lists. |
| `prepend $list VALUE` | Scalar | Root-only list prepend. |
| `append $path[INDEX] VALUE` | Scalar, list, dictionary | Appends to an existing nested list. |
| `remove PATH[SELECTOR]` | Selected item | Removes and compacts the list. |
| `delete PATH[INDEX]` | Existing index | Deletes and compacts the list. |
| `PATH[INDEX] = VALUE` | Scalar | Legacy list-item replacement. |

```shelly
$queue = ["Luis"]
append $queue "Mora"
prepend $queue "Ana"
$queue[2] = "Lee"
remove $queue[first]
set $queue[1] "Ada"
delete $queue[1]
```

Nested dictionary and list paths can be combined:

```shelly
$data = {"items, now": [["a"]]}
set $data["items, now"][1][1] "b"
append $data["items, now"][1] "c"
delete $data["items, now"][1][1]
shellout $data["items, now"][1]
```

Quoted bracket keys preserve commas and spaces. Nested `prepend` is rejected. These are invalid: `append $CONSTANT "x"`, `prepend $data.items "x"`, `set $items[4] "x"`, `append $items ["x"]`, and `append $items "x", "y"`.

## Related topics

- [Dictionaries](dictionaries.md) — keyed values and mixed paths.
- [JSON helpers](json.md) — convert lists and dictionaries to and from JSON.
- [Variables](variables.md) and [Constants](constants.md) — mutable and immutable collection roots.
