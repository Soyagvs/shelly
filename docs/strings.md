# Strings and interpolation

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

A **string** is text. A value is the expression that supplies it: a quoted string is one value, while a variable, selector, or function result can supply another. See [variables](variables.md), [constants](constants.md), [lists](lists.md), and [dictionaries](dictionaries.md) for those values' own rules. Strings can be passed to [commands](commands.md), [files](files.md), and [functions](functions.md).

## Single-line strings (supported)

Double quotes delimit a single-line string. The closing quote must be on the same physical line; a quoted string cannot continue onto the next line. Single quotes have no delimiter meaning: they are ordinary characters inside the text.

```shelly
$name = "Ada"
$APP = "Shelly"
shellout "hello $name from $APP"
shellout "It's valid text"
```

Only these escapes are supported:

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

**Invalid forms (do not copy as runnable scripts):** an unescaped newline before the closing quote, an unsupported escape such as `\\q`, a dangling backslash at end of string, or any non-whitespace content after the closing quote. A single-quoted form such as `'Ada'` is not a separate string syntax; the quote characters are ordinary text.

## Multiline strings (supported)

A multiline string opens with unquoted triple backticks (` ``` `) followed only by whitespace. Its closing delimiter is a line whose trimmed text is exactly ` ``` `. Opening and closing delimiter lines are excluded from the value. Every other body newline and indentation is preserved exactly.

````shelly
$banner = ```
Welcome
  to Shelly
```
shellout $banner
````

Multiline values are accepted anywhere Shelly accepts a string/value expression, including assignment, [commands](commands.md), [files](files.md), and [function](functions.md) arguments:

````shelly
write "message.txt" ```
first line
second line
```
append "message.txt" ```
third line
```

fn echo($text) {
    shellout $text
}
call echo (```
hello
from a call
```
)
````

The body is text, not Shelly source: comments, braces, and Shelly-looking commands inside it are not executed or parsed. Multiline strings still perform [interpolation](#interpolation), however.

````shelly
$name = "Ada"
shellout ```
Hello $name
shellout this is still literal body text
```
````

A missing closing delimiter is invalid. A body line that is exactly a delimiter after trimming closes the string there; it therefore cannot be included as content.

## Interpolation (supported and rejected forms)

Inside a string, `$name` reads a variable or [constant](constants.md). A selector may read a list element, a list selection, or a dictionary entry. Dictionary keys may be quoted. Whitespace before a selector is allowed where the selector remains attached to the name (for example, `$names [1]`).

```shelly
$names = ["Ana", "Luis", "Mora"]
$user = {"display name": "Ana"}

shellout "first: $names[1]"
shellout "selected: $names[1,3]"
shellout "user: $user[\"display name\"]"
shellout "# is text, not a comment"
```

Interpolation also works in multiline strings. It produces text in the surrounding string; it does not turn the surrounding string into a command or otherwise execute its contents.

**Rejected interpolation examples (invalid; not runnable scripts):**

- `shellout "$$name"` — legacy `$$` syntax is unsupported.
- `shellout "$missing"` — the name must be defined.
- `shellout "$names[unknown]"` — selectors must be valid for the value and selector syntax.
- `shellout "$names"` or `shellout "$user"` — a whole list or dictionary is not automatically converted to a string.

For the complete contracts of names and selectors, link to [variables](variables.md), [constants](constants.md), [lists](lists.md), and [dictionaries](dictionaries.md) rather than treating interpolation as a separate data format.

## String versus value

A quoted or multiline string is text, with escapes decoded and interpolations substituted. A bare variable or selector is a value supplied directly to a command or function. Use a string when you need surrounding text; use the value when the receiving command accepts that value directly.

```shelly
$name = "Ada"
shellout "Hello $name"       # string with interpolation
shellout $name                # value supplied directly
write "greeting.txt" "Hi $name\n"  # escapes and interpolation in file text
```

See [comments](comments.md) for comment syntax; `#` inside a string is text.

## Limits

- Shelly has no custom or heredoc delimiter for multiline strings.
- Multiline content cannot contain a line whose trimmed text is exactly triple backticks.
- Lists and dictionaries are not automatically stringified; interpolate a selector or use the collection through the appropriate value contract.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
