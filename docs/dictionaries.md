# Store keyed values in dictionaries

Dictionaries map keys to scalars, lists, and other dictionaries. This guide covers access paths, copying, mutation, and dictionary helpers.

## 🚀 At a glance

```shelly
$user = {
    name: "Ada"
    age: 32
    tags: ["language", "design"]
    address: { city: "Buenos Aires" }
}

shellout $user.name
shellout $user.address.city
```

- Use dot access for simple identifier keys.
- Use brackets for quoted, dynamic, spaced, or punctuated keys.
- Paths can mix dictionary steps and one-based list indexes.
- Only a mutable lowercase root can be mutated.

## Create, access, and copy

```shelly
$user = {
    name: "Ada"
    age: 32
    admin: true
    "first-name": "Ada"
    tags: ["language", "design"]
    address: { city: "Buenos Aires" }
}

shellout $user["first-name"]
shellout $user.tags[1]
```

Dot keys must be simple identifiers. Brackets accept quoted keys or an expression whose value is the key:

```shelly
$key = "first-name"
$country = "argentina"
$data = { pais: { argentina: { city: "Buenos Aires" } } }

shellout $user[$key]
shellout $data.pais[$country].city
shellout $data["pais"]["argentina"].city
```

Assigning a dictionary or list copies its value. Empty dictionaries are false in conditions; non-empty dictionaries are true:

```shelly
$empty = {}
if not $empty
    shellout "empty"
end

$first = { name: "Ada" }
$second = $first
set $first.name "Grace"
# $second.name is still "Ada"
```

## Paths and mutation

A path's intermediate segments must already exist and have the required collection type. Lists do not auto-expand. Mutation commands use whitespace-separated arguments; a top-level comma is rejected.

- `set PATH VALUE` creates or replaces a terminal dictionary key.
- Assignment with a path replaces an existing terminal key but cannot create one.
- `delete PATH` removes an existing terminal dictionary key.
- `append` can target a nested list and accepts scalar, list, or dictionary values.
- `prepend` is top-level-only and scalar-only; see [Lists](lists.md).

```shelly
$data = { profile: {}, "a, key": {items: ["old"]} }
set $data.profile.name "Ada"
$data.profile.name = "Grace"
set $data["a, key"].items[1] "x, y"
append $data["a, key"].items "z z"
delete $data["a, key"].items[1]
set $data.profile {
    name: "Ada"
    age: 32
}
delete $data.profile.name
```

The following fail because an intermediate key is missing or assignment cannot create a key:

```text
set $data.missing.name "x"
$data.profile.missing = "x"
```

Mutation through a constant or expression is rejected. Dictionary paths do not make list `prepend`, filters, or scalar-list helpers applicable.

## Helpers

| Helper | Result |
| --- | --- |
| `keys $dict -> $keys` | Dictionary keys in deterministic order. |
| `values $dict -> $values` | Values in the same order; nested values stay nested. |
| `isDict $value -> $ok` | Whether the value is a dictionary. |
| `len $dict -> $count` | Number of keys. |

```shelly
$data = { b: [2, 1], a: { name: "Ada" } }
keys $data -> $keys
values $data -> $values
isDict $values[1] -> $nested
shellout $keys
shellout $values[1].name
len $data -> $count
shellout $count
```

`len` also supports strings and lists; see [Helpers](helpers.md). For JSON conversion, see [JSON helpers](json.md). For conditions and variable behavior, see [Conditionals](conditionals.md) and [Variables](variables.md).

## Related topics

- [Lists](lists.md) — indexes, selectors, and collection mutation.
- [JSON helpers](json.md) — nested dictionary conversion.
- [Commands](commands.md) — command argument and capture syntax.
