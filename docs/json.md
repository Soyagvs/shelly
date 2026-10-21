# JSON helpers

Shelly can convert between JSON text and Shelly values. Both helpers take exactly one argument; the result is captured with `->`:

```shelly
$text = "{\"users\":[{\"name\":\"Ada\"}],\"empty\":null}"
jsonParse $text -> $data
shellout $data.users[1].name
jsonStringify $data -> $json
```

## Mapping and access

- JSON objects become dictionaries, and JSON arrays become lists. Both may be nested recursively.
- JSON strings, numbers, and booleans become scalar text (`true` and `false` are lowercase).
- JSON `null` becomes an empty scalar.
- List indexes are **one-based**, so `$data.users[1].name` selects the first user. Use dot or bracket dictionary access for nested keys; see [Dictionaries](dictionaries.md) and [Lists](lists.md).

`jsonStringify` recursively converts dictionaries and lists. For scalar values, the text `true` or `false` becomes a JSON boolean, numeric text becomes a JSON number, and empty text becomes JSON `null`; other text becomes a JSON string. This coercion means scalar-looking text is not always preserved as a JSON string.

**Limitation: null and empty text are lossy.** After parsing, JSON `null` and an empty JSON string are both empty Shelly scalars; stringifying either emits `null`.

Conversions recursively copy their values. They do not create aliases to an existing nested dictionary or list. After parsing or assignment, normal mutation rules apply: mutate only through a mutable lowercase variable root, and see [Dictionaries](dictionaries.md#mutation) and [Lists](lists.md#mutation).

Output is compact JSON. There is no pretty-print or formatting option.

## Arguments, destinations, and errors

The forms are exactly `jsonParse TEXT -> $variable` and `jsonStringify VALUE -> $variable`. The destination must be a mutable variable, not a path, constant, or expression. Invalid argument counts are reported as `jsonParse expects 1 argument(s)` or `jsonStringify expects 1 argument(s)`; an invalid destination reports `<helper> destination must be a variable`. Invalid JSON reports `jsonParse failed` followed by the parser message. See [Variables](variables.md) for mutable variables and [Helpers](helpers.md) for command conventions.
