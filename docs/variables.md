# Variables

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Variables are named values that a script can change. They start with `$` and use lowercase names.

```shelly
$name = "world"
$count = 42
$enabled = true

shellout "hello $name"
```

## Naming

- Every mutable variable starts with `$`.
- Names start with an ASCII letter, followed by ASCII letters, numbers, or `_`.
- Mutable variable names are lowercase-only: `$name`, `$item2`, and `$user_name` are valid.
- `$Name`, `$userName`, `$2items`, and `$user-name` are rejected.
- Uppercase identifiers are constants, not mutable variables; see [Constants](constants.md).

## Declaration and assignment

A bare declaration creates an empty string:

```shelly
$note
```

Use `=` to assign or replace the whole value. Reassignment is supported:

```shelly
$name = "Ada"
$name = "Grace"
```

The left side of `=` is the variable name, so `$name =` is invalid. Constants cannot be reassigned; see [Constants](constants.md).

## Empty values and undefined variables

A declared empty variable contains `""`. It is different from reading `$missing` when no such variable has been declared: the latter is an undefined-variable error. Empty strings, `false`, and numeric `0` are falsey in conditions; this guide does not treat every collection as falsey or truthy.

```shelly
$note
if not $note
    shellout "note is empty"
end
```

## Assignment forms

Declare or assign one variable per line. Comma-separated declarations and assignments are rejected:

```shelly
$first
$second
$first = "Ada"
$second = "Lin"
```

These forms are invalid:

```shelly
$first, $second
$first = "Ada", $second = "Lin"
```

A multi-destination capture is the exception because the operation defines several result destinations:

```shelly
command pwd -> $out, $err, $code
```

`->` binds an operation's result to a variable. It is also used by non-command operations:

```shelly
len "hello" -> $length
```

## Values and copies

Variables can hold strings, numbers, booleans, lists, and dictionaries. Operations return values that can be assigned with `->`; the empty value is the empty string `""`, not a null or undefined type.

Assigning a list or dictionary copies its current value. Later mutation of one variable's collection does not change the other:

```shelly
$first = ["a"]
$second = $first
append $first "b"
```

See [Lists](lists.md), [Dictionaries](dictionaries.md), and [JSON](json.md) for collection and JSON details.

## Collection mutation

Only collections stored in mutable variables may be changed. For example:

```shelly
$items = ["a"]
append $items "b"
```

Use [Lists](lists.md) and [Dictionaries](dictionaries.md) for selectors and the complete mutation syntax.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Scope

Top-level variables are available at top level. Variables created inside a function are local to that function by default. Inside a function, use `global` when you explicitly need to read a selected outer variable:

```shelly
$prefix = "Hello"

fn greet($name) {
    global $prefix
    $message = "$prefix $name"
    shellout $message
}

call greet("Ada")
```

See [Functions](functions.md) for full scope rules.
