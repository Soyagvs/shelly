# JSON helpers

Shelly can convert between JSON text and Shelly values.

```shelly
$text = "{\"users\":[{\"name\":\"Ada\"}],\"empty\":null}"
jsonParse $text -> $data
shellout $data.users[1].name
jsonStringify $data -> $json
```

- `jsonParse $text -> $value` parses JSON text.
- `jsonStringify $value -> $text` emits valid JSON.
- JSON objects become dictionaries.
- JSON arrays become lists, including nested lists and dictionaries.
- JSON strings, numbers, and booleans become Shelly scalar values.
- JSON `null` becomes an empty Shelly string. When stringified, an empty scalar is emitted as `null`.

Runtime parse failures include `jsonParse failed` and the JSON parser message. The destination after `->` must be a variable.
