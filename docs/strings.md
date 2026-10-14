# Strings and interpolation

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Strings use double quotes for one-line text. Use triple backticks for multiline text.

```shelly
$name = "Ada"
$APP = "Shelly"

shellout "hello $name from $APP"
```

````shelly
shellout ```
line one
line two
```
````

## Rules

- Double-quoted strings are one-line text.
- Triple-backtick strings use exactly ``` as the opening and closing delimiter.
- Triple-backtick delimiter lines are not part of the value.
- Triple-backtick strings preserve internal newlines and indentation exactly.
- Single quotes are ordinary text inside a string.
- Interpolation works for variables, constants, and list selectors in string values.
- `#` inside a string is text, not a comment.
- Use escapes for special characters in double-quoted strings.

## Escapes

| Escape | Meaning |
| --- | --- |
| `\n` | newline |
| `\t` | tab |
| `\"` | double quote |
| `\\` | backslash |

## Multiline examples

Triple-backtick strings work where Shelly accepts string/value expressions, including `shellout`, assignment, `write`, `append`, and function call arguments.

````shelly
$banner = ```
Welcome
  to Shelly
```
shellout $banner
````

````shelly
write "message.txt" ```
first line
second line
```
append "message.txt" ```
third line
```
````

````shelly
fn echo($text) {
    shellout $text
}

call echo(```
hello
from a call
```
)
````

Content inside triple backticks is not parsed as Shelly syntax, comments, or block markers.

````shelly
shellout ```
end
}
fn not_a_function() {
# not a comment
```
````

## Interpolation examples

```shelly
$names = ["Ana", "Luis", "Mora"]

shellout "first: $names[1]"
shellout "selected: $names[1,3]"
shellout "# not a comment #"
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
