# Manage mutable values with variables

Variables give a Shelly script named, changeable values. This guide covers declaration, assignment, scope, collection copies, and mutation targets.

## 🚀 At a glance

```shelly
$name = "world"
$count = 42
$enabled = true

shellout "hello $name"
```

- Mutable names start with `$` and use lowercase ASCII identifiers.
- A bare declaration creates the empty string (`""`).
- Use `=` for assignment and `->` to capture an operation result.
- Only mutable variables can receive captures or collection mutations.

## Names and declarations

Every mutable variable starts with `$`. Its name starts with an ASCII letter and continues with ASCII letters, numbers, or `_`. Mutable names are lowercase-only: `$name`, `$item2`, and `$user_name` are valid. `$Name`, `$userName`, `$2items`, and `$user-name` are rejected. Uppercase identifiers are constants; see [Constants](constants.md).

A bare declaration creates an empty string:

```shelly
$note
```

It is different from reading `$missing`, which is an undefined-variable error.

## Assignment and captures

Use `=` to assign or replace the whole value. Reassignment is supported:

```shelly
$name = "Ada"
$name = "Grace"
```

The left side of `=` is the variable name, so `$name =` is invalid. Declare or assign one variable per line; comma-separated forms are rejected:

```shelly
$first
$second
$first = "Ada"
$second = "Lin"
```

```text
$first, $second
$first = "Ada", $second = "Lin"
```

A multi-destination capture is the exception because the operation defines several destinations:

```shelly
command pwd -> $out, $err, $code
len "hello" -> $length
```

## Values and copies

Variables can hold strings, numbers, booleans, lists, and dictionaries. The empty value is `""`, not a null or undefined type. Empty strings, `false`, and numeric `0` are falsey in conditions.

Assigning a list or dictionary copies its current value. Later mutation of one collection does not change the other:

```shelly
$first = ["a"]
$second = $first
append $first "b"
```

A declared empty variable can be tested normally:

```shelly
$note
if not $note
    shellout "note is empty"
end
```

## Collection mutation

Only collections stored in mutable variables may be changed:

```shelly
$items = ["a"]
append $items "b"
```

See [Lists](lists.md), [Dictionaries](dictionaries.md), and [JSON helpers](json.md) for selectors, paths, and complete mutation rules.

## Scope

Top-level variables are available at top level. Variables created inside a function are local by default. Inside a function, use `global` when you explicitly need to read a selected outer variable:

```shelly
$prefix = "Hello"

fn greet($name) {
    global $prefix
    $message = "$prefix $name"
    shellout $message
}

call greet("Ada")
```

See [Functions](functions.md) for full scope rules and [Strings](strings.md) for interpolation.

## Related topics

- [Constants](constants.md) — uppercase, immutable names.
- [Lists](lists.md) and [Dictionaries](dictionaries.md) — collection access and mutation.
- [Numeric expressions](numeric-expressions.md) — numeric values and operators.
