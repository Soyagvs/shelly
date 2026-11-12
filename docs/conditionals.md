# Choose a path with conditionals

Use `if`/`elif`/`else` for truth-based branching, or `case` when one value should dispatch to exact matches.

## 🚀 Quick path

```shelly
$enabled = true
$count = 3

if $enabled
    shellout "enabled"
elif $count >= 10
    shellout "large"
else
    shellout "fallback"
end
```

## 📖 Conditions and comparisons

Conditions support comparisons, predicates, `not`, `and`, `or`, and parentheses. Supported comparisons are `==`, `!=`, `>`, `>=`, `<`, and `<=`. Equality is exact: there is no implicit case folding or substring matching. Ordered comparisons are numeric when both values are integers; otherwise they are lexicographic text comparisons. Use `contains` for containment.

`not` binds first, then `and`, then `or`. Operators short-circuit: `and` skips its right side after false, and `or` skips it after true.

```shelly
if $count > 0 and $count < 10
    shellout "small positive number"
end

if ($mode == "dev" or $mode == "test") and not $disabled
    shellout "enabled non-production"
end
```

Parentheses may nest. An unmatched parenthesis, empty group, or malformed group is an error.

## 🔎 Predicates and truthiness

- `exists "path"` is true when a script-relative file or directory exists; missing paths are false.
- `contains $value "thing"` checks a string substring, an exact rendered list item, or a dictionary key.
- `matches $value /pattern/` performs a Rust-regex substring match. Lists match when any rendered element matches; dictionary values are unsupported. Use `^` and `$` for whole-value matching.

A predicate can capture its boolean result with `-> $variable`; see [Predicates](predicates.md).

Scalar values are false only when exactly `false`, exactly `0`, or `""`. Other scalar strings are true, including `"False"`, `"0"`, and whitespace. Non-empty lists and dictionaries are true; empty ones are false, including empty selector/filter results.

```shelly
$numbers = [1, 3, 5]
if $numbers[>10]
    shellout "has big numbers"
else
    shellout "no big numbers"
end
```

## 🧭 Case blocks

`case VALUE` evaluates its value once. The first exactly equal `when` branch runs, with no fallthrough. `when VALUE or VALUE` supplies alternatives; `else` is optional and runs only when no branch matches.

```shelly
$mode = "prod"
case $mode
    when "dev" or "test"
        shellout "non-production"
    when "prod"
        shellout "production"
    else
        shellout "unknown mode"
end
```

`case` needs at least one `when`; `when` needs at least one value. `else` may appear once and must be last. Invalid forms include `when` outside `case`, a missing `end`, duplicate `else`, `when` after `else`, comma-separated alternatives, or malformed values.

## 🛠️ Blocks, errors, and checks

`if` starts a block; `elif` and `else` are optional; `end` closes it. Blocks do not create scope, so assignments remain available afterward. Nested `if`, `repeat`, `while`, `for`, and `try` blocks are supported. Missing predicate arguments, an unterminated pattern, and an invalid regex are errors.

`shelly check` validates syntax, grouping, predicates, and block structure without running commands or evaluating filesystem state. `shelly run` evaluates conditions, short-circuits them, and reports runtime errors and results.

## Related guides

- [Predicates](predicates.md) — reusable boolean checks and captures.
- [While loops](while.md) — repeat while a condition is true.
- [Repeat loops](repeat.md) and [For loops](for.md) — other control-flow loops.
- [Choose menus](choose.md) — interactive value selection.
