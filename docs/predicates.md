# Check values with predicates

Predicates are built-in boolean checks. Use them in conditions, combine them with logical operators, or capture their result with `->`.

## 🚀 Quick reference

| Form | Result |
| --- | --- |
| `exists "path"` | File or directory exists. |
| `not exists "path"` | File or directory is missing. |
| `contains $value "thing"` | Type-specific containment check. |
| `matches $value /pattern/` | Rust-regex substring match. |
| Any form followed by `-> $var` | Captures `true` or `false`. |

## 📁 Check files

`exists` accepts one quoted path, resolved relative to the Shelly script file rather than the launch directory. Missing paths return `false`; globs are not expanded.

```shelly
if exists "README.md"
    shellout "README found"
end

if not exists "tmp/output.txt"
    shellout "output is missing"
end
```

An unquoted path, variable, or extra argument is rejected.

## 🔎 Check containment

`contains` takes a value and a search value. The search value is evaluated normally.

- **Strings:** substring containment (`contains "hello" "ell"` is true).
- **Lists:** exact containment of a rendered item, not a substring.
- **Dictionaries:** key containment; values are not searched.

```shelly
$names = ["Ana", "Luis", "Mora"]
$labels = {"ready": "yes", "owner": "Ana"}

if contains $names "Luis" and contains $labels "owner"
    shellout "found"
end
```

A missing search item, dictionary key, or matching list item returns `false`. An undefined value used in either argument is an evaluation error. Missing or malformed arguments are rejected.

## 🧾 Capture a result

Append `-> $variable` to store a boolean. The destination must be a plain mutable variable, and the arrow must be at the end.

```shelly
exists "README.md" -> $hasreadme
contains $message "shell" -> $mentionsshell
matches $name /(?i)^[a-z]+$/ -> $validname

if $hasreadme and $mentionsshell
    shellout "ready"
end
```

Capture does not store matched text or dictionary values. Do not put `->` inside an argument or use more than one destination.

## 🔤 Match regular expressions

`matches` accepts a scalar or list and a slash-delimited Rust `regex` pattern. Matching is by substring unless anchored with `^` and `$`; a list succeeds when any rendered item matches. Dictionary values are rejected.

```shelly
if matches $name /(?i)^[a-z]+$/
    shellout "letters only"
end
```

The slashes delimit the pattern; escape a literal delimiter as `\/`. Regex escaping and Shelly string escaping differ: `\\d` in a regex literal means the digit class. Character classes, ranges, quantifiers, and inline flags such as `(?i)` are supported. Backreferences and look-around are not supported by Rust regex. A quoted pattern is not accepted. Unterminated or invalid regex syntax is rejected.

## 🧩 Compose predicates

Logical priority is `not`, then `and`, then `or`; use parentheses for clarity. Predicate operators short-circuit.

```shelly
if exists "README.md" and contains $names "Luis"
    shellout "both checks passed"
end

if not exists "dist/app" or contains $message "rebuild"
    shellout "build needed"
end
```

`include` loads and runs another Shelly file; it is not containment. Use `contains` for values and `exists` for quoted script-relative paths. See [Files](files.md).

## Related guides

- [Conditionals](conditionals.md) — use predicates in control flow.
- [Lists](lists.md), [Dictionaries](dictionaries.md), and [Strings](strings.md) — values being checked.
