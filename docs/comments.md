# Explain Shelly code without changing it

Comments help people understand a Shelly script. Shelly ignores them, so they document intent without changing runtime behavior.

## 🚀 Quick path

```shelly
# Greet the person running this script #
shellout "hello"
```

## 📖 Syntax and rules

| Form | Meaning |
| --- | --- |
| `# note #` | One-line comment |
| `#` … `#` | Block comment spanning multiple lines |
| `#` inside a string | Ordinary string text |

- Comments open with `#` and close with `#`.
- A one-line comment opens and closes on the same line.
- A block comment may span multiple lines.
- An unclosed comment is an error.
- Use comments for decisions, limitations, or surprising requirements; do not repeat clear code.

```shelly
# The API accepts dates only in UTC. #
$DATE_FORMAT = "UTC"

# Avoid comments that merely repeat the instruction. #
shellout "ready"
```

## 🧩 Block comments and strings

```shelly
#
This whole block is ignored.
Use it for a longer note.
#
shellout "after the comment"
```

A `#` in a quoted string is printed as text:

```shelly
shellout "# this prints as text #"
```

## Related guides

- [Language overview](language.md) — learn Shelly syntax in context.
- [Strings](strings.md) — work with quoted text.
- [Documentation index](index.md) — browse the complete reference.
