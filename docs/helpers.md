# Standard helpers

Shelly standard helpers transform values and store the result with `-> $variable`. Helpers do not access the filesystem unless noted elsewhere; path helpers only parse path strings.

## Rules

- Every helper requires `-> $variable`.
- Lists remain lists, booleans are `true` / `false`, and numbers are normalized scalar strings.
- `split "" "," -> $items` returns an empty list `[]`.
- `len` works on strings and lists.
- `join`, `sort`, `reverse`, and `unique` require lists and fail clearly for scalars.
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
```

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
basename "dir/archive.tar.gz" -> $name
dirname "dir/archive.tar.gz" -> $dir
extension "dir/archive.tar.gz" -> $ext
withoutExtension "dir/archive.tar.gz" -> $name
```

## Time

```shelly
now -> $timestamp
today -> $date
```

`now` returns a UTC timestamp like `2025-01-31T12:34:56Z`. `today` returns a UTC date like `2025-01-31`.

## Random

```shelly
random -> $value
randomInt 1 6 -> $roll
randomInt 5 5 -> $five
```

`randomInt` arguments must be integers, and `min` must be less than or equal to `max`.
