# Dictionaries

Nested collection mutation uses `set PATH VALUE`, `append PATH VALUE`, and `delete PATH`. Dot paths address simple keys (`$config.server.port`); bracket-string paths address arbitrary keys (`$config["server-name"]`). Intermediate segments must already exist and have the required collection type. `set` may create or replace only the terminal dictionary key; `delete` is strict and requires an existing key. Collection assignment copies values, with no implicit aliases. `insert`, `prepend`, slices, and range replacement remain unsupported.

Dictionaries store keyed values.

```shelly
$user = {
    name: "Ada"
    age: 32
    admin: true
    "first-name": "Ada"
}

shellout $user.name
shellout $user["first-name"]

$key = "first-name"
shellout $user[$key]
```

Use dot access for simple identifier keys. Use brackets for quoted or dynamic keys. Access can be chained through nested dictionaries, and dot/bracket forms can be mixed.

```shelly
$user = {
    pais: {
        argentina: {
            city: "Buenos Aires"
        }
    }
}

$country = "argentina"
shellout $user.pais.argentina.city
shellout $user["pais"]["argentina"].city
shellout $user.pais[$country].city
```

Chained paths also work in assignment, removal, and string interpolation.

```shelly
$user.pais.argentina.city = "Córdoba"
$user["pais"]["argentina"].city = "Rosario"
shellout "city=$user.pais.argentina.city"
remove $user.pais.argentina.city
```

`remove $dict.path.key` removes the final key from its parent dictionary. Missing keys are runtime errors. Assigning or removing through dot or brackets requires each traversed value to be a dictionary. Dot keys must be simple identifiers; use brackets for keys with spaces, punctuation, or dynamic values.

Empty dictionaries are false in conditions; non-empty dictionaries are true.

```shelly
$empty = {}
if not $empty
    shellout "empty"
end
```

`contains $dict "key"` checks dictionary keys. `shellout $dict` prints keys deterministically.

Limitations: lists remain scalar-only in this release, so list items cannot be dictionaries or nested lists. Dictionary values may be scalars, lists, or dictionaries.

## Dictionary helpers and nested values

Use `keys $dict -> $keys` for deterministic key order and `values $dict -> $values` for values in that same order. Values can be scalars, lists, or dictionaries.

```shelly
$data = { b: [2, 1], a: { name: "Ada" } }
keys $data -> $keys
values $data -> $values
shellout $keys
shellout $values[1].name
```
