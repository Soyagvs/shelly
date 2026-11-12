# Transform values with standard helpers

Use Shelly's standard helpers to transform text, collections, paths, and runtime values. Helpers capture results with `-> $variable` and leave the destination unchanged when validation fails.

## Quick path

```shelly
trim "  hello  " -> $text
split "a,b,c" "," -> $items
join $items "," -> $csv
formatDate "2025-01-31T12:34:56Z" "YYYY/MM/DD HH:mm:ss" -> $formatted
```

Every helper uses `HELPER arguments -> $variable`. Missing, malformed, or wrong-arity arguments fail before assignment. Results are scalar strings unless the helper returns a list or dictionary. Transformations return new collections; they do not mutate the input.

## Text

```shelly
lower "HELLO" -> $lower
upper "hello" -> $upper
replace "one two" "two" "three" -> $text
startsWith "hello" "he" -> $ok
endsWith "hello" "lo" -> $ok
substring "héllo" 2 3 -> $part
padLeft "42" 5 "0" -> $padded
padRight "ok" 4 "·" -> $padded
```

| Helper | Contract |
| --- | --- |
| `trim`, `lower`, `upper`, `replace` | Return transformed text. |
| `substring` | Uses one-based Unicode character positions and a positive length; an out-of-range start produces an empty or shortened result. |
| `padLeft`, `padRight` | Require a non-negative width and exactly one Unicode fill character; never truncate. |
| `split` | Returns a list and rejects an empty separator. |

## Collections and conversion

```shelly
$items = ["b", "a", "b"]
len $items -> $count
sort $items -> $sorted
reverse $items -> $reversed
unique $items -> $unique
join $items ", " -> $text

toNumber "007.50" -> $number
toString $number -> $text
isEmpty "" -> $empty
isNumber $number -> $ok
isText "hello" -> $ok
isList ["a"] -> $ok
```

`len` counts Unicode characters, list items, or dictionary entries. `join`, `sort`, `reverse`, and `unique` require scalar lists and reject scalars or nested items. `toNumber` rejects invalid numeric input. `isText` is true for scalar values that are neither numbers nor booleans.

## Paths and commands

Path helpers parse path text only; they do not inspect the filesystem. `pathJoin` requires at least two parts. `basename` accepts one or two arguments and removes an optional extension only when it matches the filename suffix. `dirname`, `extension`, and `withoutExtension` each accept one path.

```shelly
pathJoin "dist" "app.zip" -> $path
basename "dir/archive.tar.gz" ".tar.gz" -> $name
dirname "dir/archive.tar.gz" -> $dir
extension "dir/archive.tar.gz" -> $ext
withoutExtension "dir/archive.tar.gz" -> $name
commandExists "git" -> $hasgit
which "git" -> $gitpath
```

`commandExists` and `which` are the exceptions: bare command names search `PATH`; path-qualified names are checked as filesystem paths. Missing commands return `false` or an empty string.

## Time, random, and JSON

```shelly
now -> $timestamp
timestamp -> $timestamp
today -> $date
random -> $value
randomInt 1 6 -> $roll
jsonParse $text -> $value
jsonStringify $value -> $json
keys $dict -> $keys
values $dict -> $values
isDict $value -> $ok
```

`now` and its alias `timestamp` return a UTC timestamp; `today` returns a UTC date. Integer bounds and widths must fit Shelly's signed range and be non-negative where stated. `randomInt` is inclusive and requires `min <= max`. JSON `null` becomes an empty scalar. See [JSON values](json.md) for JSON behavior.

`formatDate` is documented in full in [Date formatting](date-formatting.md); keeping its token and failure reference there avoids conflicting descriptions.

## Next step

For failures and recovery, see [Error handling](errors.md). For date tokens and strict UTC input, see [Date formatting](date-formatting.md).
