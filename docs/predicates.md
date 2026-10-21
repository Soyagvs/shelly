# Predicates

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Predicates are built-in boolean checks. Use them directly in `if` / `elif` conditions, combine them with logical operators, or capture the result with `->`. See [lists](lists.md), [dictionaries](dictionaries.md), and [strings](strings.md) for value types, and [conditionals](conditionals.md#predicates) for control-flow context.

## Quick reference

| Form | Result |
| --- | --- |
| `exists "path"` | `true` when the file or directory exists. |
| `not exists "path"` | `true` when the file or directory is missing. |
| `contains $value "thing"` | Type-specific containment check. |
| `matches $value /pattern/` | Rust-regex substring match. |
| `exists "path" -> $var` | Captures the boolean result in `$var`. |
| `contains $value "thing" -> $var` | Captures the boolean result in `$var`. |
| `matches $value /pattern/ -> $var` | Captures the boolean result in `$var`. |

## File and directory checks

`exists` accepts one **quoted string path**. The path is resolved relative to the Shelly script file, not the directory where `shelly` was launched.

```shelly
if exists "README.md"
    shellout "README found"
end

if not exists "tmp/output.txt"
    shellout "output is missing"
end
```

Missing paths return `false`; they do not stop the script. An unquoted path, a variable, or extra arguments are rejected. `exists` does not expand globs.

## Containment checks

`contains` takes a value and a search value. The search value is evaluated normally (so a quoted string or a defined variable can be used). If the evaluated search item or key is absent, or no item matches, the result is `false`; reading an undefined value while evaluating either argument is an error.

- **Strings:** substring containment; `contains "hello" "ell"` is true.
- **Lists:** exact item containment. Each list item is rendered and compared to the search text; it is not a substring check.
- **Dictionaries:** key containment. The search text is compared to dictionary keys; values are not searched.

```shelly
$names = ["Ana", "Luis", "Mora"]
$labels = {"ready": "yes", "owner": "Ana"}

if contains $names "Luis" and contains $labels "owner"
    shellout "found"
end
```

A missing search item, missing dictionary key, or non-matching list item returns `false`. An undefined variable used as the searched value or search expression is an evaluation error, not a `false` result. Missing arguments or malformed two-argument forms are rejected.

## Capture boolean results

Append `-> $variable` to any predicate to store its boolean result. The destination must be a plain variable, not a constant, indexed value, member expression, or literal. Capture stores a boolean value (`true` or `false`), and the arrow must be at the end of the predicate.

```shelly
exists "README.md" -> $hasreadme
contains $message "shell" -> $mentionsshell
matches $name /(?i)^[a-z]+$/ -> $validname

if $hasreadme and $mentionsshell
    shellout "ready"
end
```

Do not put `->` inside a predicate argument or use more than one destination. Predicate capture is distinct from assignments and does not capture matched text or a dictionary value.

## Regular-expression matching

`matches` accepts a scalar or list and a slash-delimited Rust `regex` pattern. Matching is by substring unless `^` and `$` anchors are used. A scalar is tested directly; a list succeeds when **any** rendered element matches. Dictionary values are rejected.

```shelly
if matches $name /(?i)^[a-z]+$/
    shellout "letters only"
end
```

The `/` characters delimit the regex; they are not part of the pattern. Escape a literal delimiter as `\/`. Regex escaping and Shelly string escaping are different: `\\d` in a regex literal means the regex digit class, while a backslash in a quoted Shelly string follows that string's escaping rules. `matches` does not accept a quoted pattern in place of `/pattern/`.

Character classes and ranges (`[a-z]`, `\\d`), quantifiers (`+`, `*`, `?`, `{2,4}`), and inline flags such as `(?i)` are supported. Invalid regex syntax is rejected. Rust regex intentionally does not support backreferences, look-around, or other non-regular constructs; numeric ranges are regex text, not numeric comparisons.

## Compose predicates

Logical priority is `not`, then `and`, then `or`. Use parentheses when grouping makes the condition clearer.

```shelly
if exists "README.md" and contains $names "Luis"
    shellout "both checks passed"
end

if not exists "dist/app" or contains $message "rebuild"
    shellout "build needed"
end
```

## Include is not containment

`include` loads and runs another Shelly file. It does not check whether a string, list, or dictionary contains a value. Use `contains` for containment and `exists` for a quoted, script-relative path. See [files](files.md) for file-loading behavior.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
