# Standard helpers

[Documentation index](index.md) · [Time and date](index.md#use-the-standard-library)

Shelly standard helpers transform values and capture the result with `-> $variable`.

<img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />

## Rules and practical limits

- Capture is universal: helpers use `HELPER arguments -> $variable`; missing, malformed, or wrong-arity arguments fail before assignment. The destination is unchanged on any failure.
- Results are scalar strings unless shown as a list or dictionary. Transformations such as `split`, `sort`, `reverse`, `unique`, `keys`, and `values` produce new collections; they do not mutate the input.
- Lists and dictionaries can be nested where JSON creates them, but scalar-only helpers reject collections, and ordinary list mutation/assignment has limited nesting support. Keep helper inputs to scalar lists unless a section says otherwise.
- Strings and widths use Unicode characters. Integer arguments must fit Shelly's signed integer range; counts and widths are non-negative where stated.
- `len` counts Unicode characters, list items, or dictionary entries.
- `join`, `sort`, `reverse`, and `unique` require scalar lists and fail for scalars or nested items. `timestamp` is an alias for `now`.

Path helpers parse path text only; they do not check the filesystem. `commandExists` and `which` are the exception: bare command names search the process `PATH`, while path-qualified names are checked as filesystem paths (`which` returns an empty string when absent).

<img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />

## Text

```shelly
trim "  hello  " -> $text
lower "HELLO" -> $text
upper "hello" -> $text
replace "one two" "two" "three" -> $text
split "a,b,c" "," -> $items
startsWith "hello" "he" -> $ok
endsWith "hello" "lo" -> $ok
substring "héllo" 2 3 -> $part
padLeft "42" 5 "0" -> $padded
padRight "ok" 4 "·" -> $padded
```

| Helper | Behavior |
| --- | --- |
| `substring` | Uses one-based Unicode character positions and a positive length; out-of-range starts produce an empty or shortened result. Other forms fail with an argument error. |
| `padLeft`, `padRight` | Require a non-negative width and exactly one Unicode fill character; they only extend strings narrower than the requested width and never truncate. |
| `split` | Rejects an empty separator (`split separator cannot be empty`). |
| `formatDate` | Formats strict UTC timestamps; see the [date formatting guide](date-formatting.md). |

For the complete `formatDate` token reference, input rules, and failure cases, see [date-formatting.md](date-formatting.md).

## Lists

```shelly
$items = ["b", "a", "b"]
len $items -> $count
join $items ", " -> $text
sort $items -> $sorted
reverse $items -> $reversed
unique $items -> $unique
```

## Conversion and validation

```shelly
toNumber "007.50" -> $number
toString $number -> $text
isEmpty "" -> $ok
isNumber $number -> $ok
isText "hello" -> $ok
isList ["a"] -> $ok
```

`toNumber` fails on invalid numeric input. `isText` is true for scalar values that are not numbers and not `true` / `false`.

## Paths

Path helpers operate on strings only; they do not check whether a file exists. `pathJoin` requires at least two parts. `basename` accepts one or two arguments and removes the optional extension only when it matches the filename suffix; `dirname`, `extension`, and `withoutExtension` each accept one path. Non-string path arguments and wrong arity are rejected.

```shelly
pathJoin "dist" "app.zip" -> $path
basename "dir/archive.tar.gz" -> $name
basename "dir/archive.tar.gz" ".tar.gz" -> $name
dirname "dir/archive.tar.gz" -> $dir
extension "dir/archive.tar.gz" -> $ext
withoutExtension "dir/archive.tar.gz" -> $name
```

## Time

```shelly
formatDate "2025-01-31T12:34:56Z" "YYYY/MM/DD HH:mm:ss" -> $formatted
```

`formatDate` accepts strict UTC timestamps in `YYYY-MM-DDTHH:mm:ssZ` form. Patterns replace only `YYYY`, `MM`, `DD`, `HH`, `mm`, and `ss`; unsupported pattern letters produce an actionable error.

```shelly
now -> $timestamp
timestamp -> $timestamp
today -> $date
```

`now` and `timestamp` return a UTC timestamp like `2025-01-31T12:34:56Z`. `today` returns a UTC date like `2025-01-31`. See the [Time and date route](index.md#use-the-standard-library) for the complete reference.

## Commands

```shelly
commandExists "git" -> $hasgit
which "git" -> $gitpath
```

Use `commandExists` before running optional tools. Use `which` when the script needs to show or reuse the resolved executable path. Missing commands return `false` from `commandExists` and an empty string from `which`.

## Random

```shelly
random -> $value
randomInt 1 6 -> $roll
randomInt 5 5 -> $five
```

`random` returns an unsigned integer string. `randomInt` returns an inclusive signed-integer value; both bounds must be integers in range, and `min` must be less than or equal to `max` (otherwise it fails).

## JSON and dictionary helpers

```shelly
jsonParse $text -> $value
jsonStringify $value -> $text
keys $dict -> $keys
values $dict -> $values
isDict $value -> $ok
```

`len` works for strings, lists, and dictionaries. `join`, `sort`, `reverse`, and `unique` require scalar lists. `jsonParse` accepts JSON values supported by Shelly; JSON `null` becomes an empty scalar. JSON details are in [`json.md`](json.md).
