# Define immutable values with constants

Constants name values that a Shelly script may read but not replace or mutate. Use them for fixed configuration and shared limits; use [Variables](variables.md) for mutable lowercase names.

## 🚀 At a glance

```shelly
$APP = "Shelly"
$MAX_RETRIES = 3

shellout "running $APP"
```

- A constant uses one `$` followed by an uppercase-only identifier.
- It must receive a value on its declaration line.
- It can be read, indexed, iterated, and passed as an argument.
- It cannot be reassigned, mutated, or used as a capture destination.

## Names and declaration rules

A constant name starts with `$`, then an ASCII letter, followed by ASCII letters, numbers, or `_`. The identifier must be uppercase-only. Valid names include `$APP`, `$MAX_RETRIES`, and `$VERSION2`.

These are invalid examples, not runnable code:

```text
$App
$userName
$$APP
$2APP
$APP-NAME
$APP
$APP = "first"  # followed later by another assignment
$APP =
```

The first is missing its initial value; the others use invalid naming or attempt reassignment or a missing value. Reading an undeclared constant is an undefined-constant error.

## Values and use

Constants can hold strings, numbers, booleans, lists, and dictionaries. They can be interpolated, used in expressions, iterated, and passed to functions:

```shelly
$GREETING = "Hello"
$LIMIT = 10
$VERBOSE = true
$COLORS = ["red", "blue"]
$SETTINGS = {"mode": "safe", "retries": 2}

$total = $LIMIT * 2
shellout "$GREETING; total: $total; verbose: $VERBOSE"

for $color in $COLORS
    shellout $color
end

fn show($value) {
    shellout $value
}

call show($SETTINGS.mode)
```

## Immutability and captures

Immutability includes collections stored in a constant. Reading and indexing are valid, but `append`, `set`, `delete`, and other collection mutations are not:

```shelly
$COLORS = ["red", "blue"]
$FIRST = $COLORS[1]
shellout $FIRST
```

```text
append $COLORS "green"
set $SETTINGS.mode "fast"
command pwd -> $APP
```

An operation's `->` destination must be a mutable variable. Store a collection in a lowercase variable when it needs to change. See [Lists](lists.md) and [Dictionaries](dictionaries.md).

## Functions and modules

Top-level constants are not implicitly visible inside functions. Use `global` to explicitly read a selected outer constant; it does not make the constant mutable:

```shelly
$APP = "Shelly"

fn identify() {
    global $APP
    shellout $APP
}

call identify()
```

Constants and functions can be exported and imported; normal variables cannot. See [Functions](functions.md) and [Modules](modules.md).

## Related topics

- [Variables](variables.md) — mutable names, assignment, and scope.
- [Strings](strings.md) — interpolation rules.
- [Lists](lists.md) and [Dictionaries](dictionaries.md) — immutable collection access.
