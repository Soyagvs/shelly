# Convert JSON text and Shelly values

Shelly's JSON helpers convert JSON text to Shelly dictionaries and lists, then serialize those values back to compact JSON. Both helpers take exactly one argument and capture their result with `->`.

## 🚀 At a glance

```shelly
$text = "{\"users\":[{\"name\":\"Ada\"}],\"empty\":null}"
jsonParse $text -> $data
shellout $data.users[1].name
jsonStringify $data -> $json
```

The first list item is `$data.users[1]` because Shelly indexes lists from `1`.

## Mapping and access

| JSON value | Shelly value |
| --- | --- |
| Object | Dictionary |
| Array | List |
| String, number, boolean | Scalar text; booleans are lowercase |
| `null` | Empty scalar |

Mappings recurse through nested objects and arrays. Use dot or bracket dictionary access and one-based list indexes; see [Dictionaries](dictionaries.md) and [Lists](lists.md).

`jsonStringify` recursively converts dictionaries and lists. Scalar coercion is:

| Shelly scalar text | JSON output |
| --- | --- |
| `true` or `false` | Boolean |
| Numeric text | Number |
| Empty text | `null` |
| Other text | JSON string |

> **Note:** JSON `null` and an empty JSON string both become empty Shelly scalars after parsing. Stringifying either emits `null`; this distinction is lossy.

Conversions recursively copy their values and do not create aliases to existing nested collections. After conversion, normal mutation rules apply: mutate only through a mutable lowercase variable root. See [Dictionaries](dictionaries.md#paths-and-mutation) and [Lists](lists.md#mutation-rules).

## Arguments, destinations, and errors

The exact forms are:

```shelly
jsonParse TEXT -> $variable
jsonStringify VALUE -> $variable
```

The destination must be a mutable variable, not a path, constant, or expression. Invalid argument counts report `jsonParse expects 1 argument(s)` or `jsonStringify expects 1 argument(s)`. An invalid destination reports `<helper> destination must be a variable`; invalid JSON reports `jsonParse failed` followed by the parser message.

Output is compact JSON. There is no pretty-print or formatting option.

## Related topics

- [Variables](variables.md) — mutable capture destinations.
- [Lists](lists.md) and [Dictionaries](dictionaries.md) — access and mutation rules.
- [Helpers](helpers.md) — command and result conventions.
