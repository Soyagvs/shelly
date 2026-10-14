# Variables

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Variables store mutable values and use one dollar sign with lowercase-only names.

```shelly
$name = "world"
$count = 42
$enabled = true

shellout "hello $name"
```

## Rules

- Variables start with `$`.
- Variable names are lowercase-only: `$name` is valid, while uppercase or mixed-case names are rejected.
- Names must start with an ASCII letter.
- After the first letter, names may contain ASCII letters, numbers, and `_`.
- A bare variable declaration creates an empty value.
- Variables can store strings, numbers, booleans, lists, empty values, and captured command output.

## Assignment

```shelly
$name = "Ada"
$count = 1 + 2 * 3
$names = ["Ana", "Luis"]
command "pwd" -> $cwd
```

`=` assigns a value directly. `->` binds the result of an operation into a variable.

## Empty value

```shelly
$note

if not $note
    shellout "note is empty"
end
```

A bare variable declaration creates an empty value. Empty values are falsey in conditions.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Dictionary variables

Variables can hold dictionaries:

```shelly
$user = { name: "Ada", age: 32 }
shellout $user.name
$user.name = "Grace"
```

Use bracket access for quoted or dynamic keys: `$user["first-name"]` or `$user[$key]`.
