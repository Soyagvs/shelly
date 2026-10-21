# Constants

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

A constant is a named value that a script is not allowed to replace or mutate. Shelly writes a constant with **one** dollar sign and an uppercase identifier, such as `$APP` or `$MAX_RETRIES`. Use [Variables](variables.md) for mutable, lowercase names.

```shelly
$APP = "Shelly"
$MAX_RETRIES = 3

shellout "running $APP"
```

## Names and declarations

A constant name starts with `$`, then an ASCII letter, followed by ASCII letters, numbers, or `_`. The identifier must be uppercase-only. It must also receive a value on the declaration line.

Valid names include `$APP`, `$MAX_RETRIES`, and `$VERSION2`. Mixed-case names, a second `$`, names beginning with a number, hyphens, and other punctuation are rejected. These are **invalid examples**, not runnable code:

- `$App`
- `$userName`
- `$$APP`
- `$2APP`
- `$APP-NAME`

These forms are also rejected:

- `$APP`
- `$APP = "first"` followed later by `$APP = "second"`
- `$APP =`

The first is missing its initial value; the other two attempt reassignment or a missing value. A constant is declared once with `=` and cannot be assigned again.

## Values and everyday use

Constants can hold strings, numbers, booleans, lists, and dictionaries. They can be interpolated into text, used in expressions, iterated over, and passed as function arguments just like other readable values.

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

Reading a constant that was never declared is an undefined-constant error. This is different from an undefined variable: both are errors, but a declared variable may also intentionally contain an empty string; see [Variables](variables.md).

## Immutable values

Immutability includes collections stored in a constant. You may read, index, and iterate over a constant list or dictionary, but you may not change it with `append`, `set` item assignment, `delete`, or another collection mutation.

```shelly
$COLORS = ["red", "blue"]
$FIRST = $COLORS[1]
shellout $FIRST
```

The following is an **invalid mutation** (do not run it):

> `append $COLORS "green"`

Likewise, `set $SETTINGS.mode "fast"` would be an invalid mutation when `$SETTINGS` is a constant. Store a collection in a mutable lowercase variable when the script needs to change it; see [Lists](lists.md) and [Dictionaries](dictionaries.md).

## Capture destinations

An operation's `->` destination must be a mutable variable. A constant may be used as an input, but it cannot receive a result. This is a **rejected example**:

> `command pwd -> $APP`

Use a lowercase variable instead, as described in [Commands](commands.md): `command pwd -> $cwd`.

## Functions and scope

Constants declared at top level are not implicitly visible inside functions. A function can explicitly read one with `global`; this does not make the constant mutable.

```shelly
$APP = "Shelly"

fn identify() {
    global $APP
    shellout $APP
}

call identify()
```

`global $APP` permits reading the selected outer constant only. It does not permit reassignment, collection mutation, or capture into `$APP`. See [Functions](functions.md) for the full scope rules.

## Modules

Constants and functions can be exported and imported. Normal variables cannot be exported or imported. See [Modules](modules.md) for the complete module syntax and rules.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
