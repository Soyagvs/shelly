# Lists

Shelly has one collection type: `list`.

```shelly
$empty = []
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2 + 3, 6]
$mixed = ["Ana", 3, true]
$name = "Ada"
$$AGE = 37
$refs = [$name, $$AGE, false]
```

## List values

List elements may be:

- strings
- numbers and numeric expressions
- booleans
- variables
- constants

Nested lists are not supported yet.

## Indexing and selectors

List indexing is 1-based.

```shelly
shellout $names[1]
shellout $names[first]
shellout $names[last]
shellout $names[$i + 1]
shellout "Second: $names[2]"
```

Scalar selectors return one value.

Multi-selectors return a new list:

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

Filters return a new list.

```shelly
shellout $numbers[>5]
shellout $numbers[<=5]
shellout $names[=="Ana"]
shellout $names[!="Ana"]
shellout $mixed[!=3]
```

Rules:

- Numeric filters support `==`, `!=`, `>`, `>=`, `<`, and `<=`.
- String and boolean filters support `==` and `!=`.
- Mixed lists support exact `==` and `!=` matching.
- Ordered filters require numeric comparison values and numeric list items.
- Ordered string filters such as `$names[>"Ana"]` are not supported yet.

## Mutations

```shelly
$names = ["Ana", "Luis"]
append $names, "Mora"
prepend $names, "Iris"
$names[2] = "Ada"
remove $names[last]
remove $names[first]
shellout $names[..]
```

Rules:

- Mutation targets must be variable lists.
- Constants, undefined values, non-lists, and temporary selector/filter results are rejected.
- Mutation indexes are 1-based and must select exactly one item.
- `first`, `last`, and numeric indexes are allowed for mutation.
- Multi-indexes, ranges/full-copy selectors, and filters are not mutation targets yet.
- Out-of-range mutation indexes are errors.
- `first`/`last` mutations on an empty list are errors.
- Mutation values use the same scalar element forms as list literals.
