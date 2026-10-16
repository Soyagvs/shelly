# Standard helpers

[Documentation index](index.md) · [Time and date](index.md#time-and-date)

Shelly standard helpers transform values and store the result with `-> $variable`. Helpers do not access the filesystem unless noted elsewhere; path helpers only parse path strings.

## Rules

- Every helper requires `-> $variable`.
- Lists remain lists, booleans are `true` / `false`, and numbers are normalized scalar strings.
- `split "" "," -> $items` returns an empty list `[]`.
- `len` works on strings and lists.
- `join`, `sort`, `reverse`, and `unique` require lists and fail clearly for scalars.
- `pathJoin` requires at least two path parts.
- `basename PATH EXT` strips the optional extension only when it matches the file name suffix.
- `timestamp` is an alias for `now`.
- `commandExists` checks whether a command can be found on `PATH`; `which` returns its path or an empty string.
- `random` returns an unsigned integer string. `randomInt min max` is inclusive.

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
| `substring` | Uses one-based Unicode character positions and a positive length. |
| `padLeft`, `padRight` | Accept exactly one Unicode fill character and only extend strings narrower than the requested width; existing text is never truncated. |
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

Path helpers operate on strings only; they do not check whether a file exists.

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

`now` and `timestamp` return a UTC timestamp like `2025-01-31T12:34:56Z`. `today` returns a UTC date like `2025-01-31`. See the [Time and date route](index.md#time-and-date) for the complete reference.

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

`randomInt` arguments must be integers, and `min` must be less than or equal to `max`.

## JSON and dictionary helpers

```shelly
jsonParse $text -> $value
jsonStringify $value -> $text
keys $dict -> $keys
values $dict -> $values
isDict $value -> $ok
```

`len` works for strings, lists, and dictionaries. `join`, `sort`, `reverse`, and `unique` require scalar lists. JSON details are in [`json.md`](json.md).
