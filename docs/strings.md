# Write text with strings and interpolation

Strings hold text for commands, files, functions, and other values. This guide shows single-line and multiline syntax, escapes, interpolation, and the boundaries between text and values.

## 🚀 At a glance

````shelly
$name = "Ada"
shellout "hello $name"

$banner = ```
Welcome
  to Shelly
```
shellout $banner
````

- Double quotes create a single-line string.
- Triple backticks create a multiline string and preserve body newlines and indentation.
- Strings interpolate variables, constants, and selectors.
- Lists and dictionaries are not automatically converted to text.

## Single-line strings

Double quotes delimit a single-line string. The closing quote must be on the same physical line. Single quotes are ordinary characters, not string delimiters.

```shelly
$name = "Ada"
$APP = "Shelly"
shellout "hello $name from $APP"
shellout "It's valid text"
```

Supported escapes are:

| Escape | Meaning |
| --- | --- |
| `\\n` | newline character |
| `\\t` | tab character |
| `\\"` | double quote |
| `\\\\` | backslash |

```shelly
$message = "line one\nline two\t(indented)"
shellout "She said \"hello\""
```

An unescaped newline, unsupported escape such as `\\q`, dangling backslash, or non-whitespace after the closing quote is invalid. `'Ada'` is text containing quote characters, not a separate string form.

## Multiline strings

A multiline string opens with unquoted triple backticks followed only by whitespace. Its closing delimiter is a line whose trimmed text is exactly triple backticks. Delimiter lines are excluded; all other body newlines and indentation are preserved.

````shelly
$banner = ```
Welcome
  to Shelly
```
shellout $banner
````

The body is text, not Shelly source. Comments, braces, and command-looking lines are not executed. Multiline strings still interpolate:

````shelly
$name = "Ada"
shellout ```
Hello $name
shellout this is still literal body text
```
````

A missing closing delimiter is invalid. A body line that trims to triple backticks closes the string and cannot be included as content.

## Interpolation

Inside a string, `$name` reads a variable or [constant](constants.md). Selectors can read list elements, list selections, or dictionary entries. Dictionary keys may be quoted.

```shelly
$names = ["Ana", "Luis", "Mora"]
$user = {"display name": "Ana"}

shellout "first: $names[1]"
shellout "selected: $names[1,3]"
shellout "user: $user[\"display name\"]"
shellout "# is text, not a comment"
```

Interpolation produces text; it does not execute the surrounding string. These are rejected:

```text
shellout "$$name"           # legacy $$ syntax
shellout "$missing"         # undefined name
shellout "$names[unknown]"  # invalid selector
shellout "$names"           # whole list is not stringified
shellout "$user"             # whole dictionary is not stringified
```

See [Variables](variables.md), [Constants](constants.md), [Lists](lists.md), and [Dictionaries](dictionaries.md) for name and selector contracts.

## String versus value

Use a string when you need surrounding text. Pass a bare variable or selector when the receiving command accepts the value directly:

```shelly
$name = "Ada"
shellout "Hello $name"
shellout $name
write "greeting.txt" "Hi $name\n"
```

See [Comments](comments.md) for comment syntax; `#` inside a string is text.

## Limits and related topics

- There is no custom or heredoc delimiter for multiline strings.
- Multiline content cannot contain a line whose trimmed text is exactly triple backticks.
- See [Files](files.md), [Commands](commands.md), [Functions](functions.md), and [JSON helpers](json.md) for string consumers and conversion.
