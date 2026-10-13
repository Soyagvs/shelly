# Predicates

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Predicates are built-in boolean checks. Use them directly in `if` / `elif` conditions, combine them with logical operators, or capture the result with `->`.

## Quick reference

| Form | Result |
| --- | --- |
| `exists "path"` | `true` when the file or directory exists. |
| `not exists "path"` | `true` when the file or directory is missing. |
| `contains $list "item"` | `true` when the list has an exact item match. |
| `contains $string "substring"` | `true` when the string includes the substring. |
| `exists "path" -> $var` | Stores `true` or `false` in `$var`. |
| `contains $value "thing" -> $var` | Stores `true` or `false` in `$var`. |

## File and directory checks

`exists` checks paths relative to the Shelly script file, not the directory where `shelly` was launched.

```shelly
if exists "README.md"
    shellout "README found"
end

if not exists "tmp/output.txt"
    shellout "output is missing"
end
```

Missing paths return `false`; they do not stop the script.

## Containment checks

Use `contains` with lists for exact item checks.

```shelly
$names = ["Ana", "Luis", "Mora"]

if contains $names "Luis"
    shellout "Luis is present"
end
```

Use `contains` with strings for substring checks.

```shelly
$message = "hello shelly"

if contains $message "shell"
    shellout "matched substring"
end
```

## Capture boolean results

Predicate capture stores a boolean value in a variable.

```shelly
exists "README.md" -> $hasReadme
contains $message "shell" -> $mentionsShell

if $hasReadme and $mentionsShell
    shellout "ready"
end
```

## Compose predicates

Logical priority is `not`, then `and`, then `or`.

```shelly
if exists "README.md" and contains $names "Luis"
    shellout "both checks passed"
end

if not exists "dist/app" or contains $message "rebuild"
    shellout "build needed"
end
```

Use parentheses when predicate logic needs grouping or clearer precedence.

```shelly
$force = false

if (exists "README.md" and contains $names "Luis") or $force
    shellout "clear condition"
end
```

## Include is not containment

`include` loads and runs another Shelly file. It does not check whether a string or list contains a value.

```shelly
include "shared.shelly"
```

Use `contains` for containment checks and `exists` when you only need to know whether a path is present.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
