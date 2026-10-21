# Dictionaries

A dictionary stores keyed values. Values may be scalars, lists, or other dictionaries. Lists can also contain dictionaries and nested lists; see [Lists](lists.md) for list indexing and list-specific mutation rules.

```shelly
$user = {
    name: "Ada"
    age: 32
    admin: true
    "first-name": "Ada"
    tags: ["language", "design"]
    address: { city: "Buenos Aires" }
}

shellout $user.name
shellout $user["first-name"]
shellout $user.address.city
shellout $user.tags[1]
```

Assigning a dictionary or list copies its value; it does not create an implicit alias. Changing one variable does not change the other. Empty dictionaries are false in conditions; non-empty dictionaries are true.

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

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Keys and paths

Use dot access for simple identifier keys. Use brackets for quoted keys or dynamic keys. A bracket key is a string or an expression whose value is used as the key. Paths can mix dots, dictionary brackets, and list indexes, and can reach nested values.

```shelly
$key = "first-name"
$country = "argentina"
$data = { pais: { argentina: { city: "Buenos Aires" } } }

shellout $user[$key]
shellout $data.pais[$country].city
shellout $data["pais"]["argentina"].city
```

Dot keys must be simple identifiers. Use brackets for spaces, punctuation, reserved words, or dynamic keys. A path's intermediate segments must already exist and have the required collection type: a dictionary key for a dictionary step, or an existing one-based list index for a list step. Lists do not auto-expand.

Comma-separated command arguments are rejected. Commas remain valid where the language requires them, such as list literals, dictionary literals, selectors, and multi-capture destinations. For example, `set $user.name "Ada", $user.age 32` is invalid; use one command per line.

## Mutation

Only a mutable, lowercase variable can be the root of a mutation. A path may traverse nested dictionaries and lists, but every traversed intermediate value must already exist. Mutation through a constant or an expression is rejected.

`set PATH VALUE` creates the terminal dictionary key when it is missing, or replaces it when it already exists. It does not create missing intermediate dictionaries, list indexes, or list entries. Assignment with a path (`$dict.key = VALUE`) replaces an existing terminal key; it does not create a missing key. Both forms replace the value at that terminal location rather than merging dictionaries. `delete PATH` removes an existing terminal dictionary key and errors when the key is missing. List insertion, replacement, and deletion follow the rules in [Lists](lists.md).

```shelly
$data = { profile: {} }
set $data.profile.name "Ada"       # creates name
$data.profile.name = "Grace"       # replaces name
set $data.profile {
    name: "Ada"
    age: 32
} # replaces the whole value
# set $data.missing.name "x"        # error: missing intermediate key
# $data.profile.missing = "x"       # error: assignment cannot create a key
delete $data.profile.name
```

Dictionary mutation changes only the targeted mutable root and its reachable value. It does not mutate a separately assigned copy, and it does not change an unrelated dictionary or list. `append` can mutate a list reached through a dictionary path, subject to the scalar-value and root rules in [Lists](lists.md); dictionary paths do not make list `prepend`, filters, or scalar-list helpers applicable.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Helpers

- `keys $dict -> $keys` returns dictionary keys in deterministic order.
- `values $dict -> $values` returns values in that same order; nested values remain dictionaries or lists.
- `isDict $value -> $ok` reports whether a value is a dictionary.
- `len $dict` returns the number of keys. (`len` also supports strings and lists; see [helpers](helpers.md).)

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

For conditions and other value behavior, see [Conditionals](conditionals.md) and [Variables](variables.md). For JSON conversion of nested dictionaries and lists, see [JSON](json.md). For command argument and mutation syntax, see [Commands](commands.md).
