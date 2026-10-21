# Conditionals

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Use `if`, optional `elif`, optional `else`, and `end` to choose which block runs. Use `case` when one value should dispatch to exact matching branches.

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

## Rules and scope

- `if condition` starts a block; `elif` and `else` are optional; `end` closes it.
- Conditions support comparisons, predicates, `not`, `and`, `or`, and parentheses.
- Blocks do not create a scope. Assignments made in an `if`, `case`, or nested loop remain available afterward.
- Nested `if`, `repeat`, `while`, `for`, and `try` blocks are supported.

<p align="center"><img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" /></p>

## Evaluation and comparisons

Supported operators are `==`, `!=`, `>`, `>=`, `<`, and `<=`. Equality and inequality compare values exactly: there is no implicit case folding or substring matching. Ordered comparisons use numeric ordering when both values are integer numbers; otherwise they use lexicographic text ordering. Use `contains` for containment rather than `==`.

`not` binds first, then `and`, then `or`. Operators short-circuit: `and` skips its right side after false, and `or` skips it after true. This can safely guard a later predicate or expression.

```shelly
if $count > 0 and $count < 10
    shellout "small positive number"
end

if ($mode == "dev" or $mode == "test") and not $disabled
    shellout "enabled non-production"
end
```

Parentheses group expressions and may nest. **Invalid:** an unmatched `(` or `)`, an empty group, or malformed text inside a group. These forms produce an error rather than a condition result.

## Predicates

- `exists "path"` is true when the file or directory exists relative to the current script; missing paths are false.
- `contains $value "thing"` checks a substring in a string, an exact rendered item in a list, or a key in a dictionary.
- `matches $value /pattern/` performs a Rust-regex substring match. A list matches when any rendered element matches; dictionary values are unsupported. Use `^` and `$` for whole-value matching.

```shelly
if matches $name /(?i)^[a-z]+$/ and contains $names "Luis"
    shellout "valid"
end
```

A predicate may be captured with `-> $variable`; see [predicates.md](predicates.md). **Invalid:** missing predicate arguments, an unterminated `/pattern/`, or an invalid regular expression.

<p align="center"><img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" /></p>

## Truthiness

Scalar values are false only when they are exactly `false`, exactly `0`, or the empty string `""`. Every other scalar string is true (including `"False"`, `"0"`, and whitespace). Non-empty lists and dictionaries are true; empty lists and dictionaries are false, including empty selector/filter results.

```shelly
$numbers = [1, 3, 5]
if $numbers[>10]
    shellout "has big numbers"
else
    shellout "no big numbers"
end
```

## Case blocks

`case VALUE` evaluates its value once. The first `when` whose value is exactly equal to it runs, with no fallthrough. `when VALUE or VALUE` supplies alternatives; each alternative is evaluated and compared exactly. `else` runs only when no `when` matches.

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

`case` requires at least one `when`; `when` requires at least one value; `else` is optional, may appear once, and must be last. **Invalid:** `when` outside `case`, a missing `end`, `case` without `when`, duplicate `else`, `when` after `else`, comma-separated alternatives, or malformed values.

<p align="center"><img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" /></p>

## `check` versus runtime

`shelly check` validates conditional syntax, grouping, predicate forms, and block structure without running commands or evaluating filesystem state. It can report malformed expressions and structural errors, but it cannot prove a runtime path exists, a variable is defined at execution time, a regex matches, or a branch will be selected. `shelly run` evaluates conditions, short-circuits them, and reports runtime errors and results.
