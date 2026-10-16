# Date formatting

[Documentation index](index.md) · [Time and date helpers](helpers.md#time)

`formatDate` turns a Shelly UTC timestamp into readable text. It replaces a small, predictable set of tokens in the pattern you provide.

## Basic usage

```shelly
formatDate "2025-01-31T12:34:56Z" "YYYY/MM/DD HH:mm:ss" -> $formatted
```

The input must use this strict form:

```text
YYYY-MM-DDTHH:mm:ssZ
```

It is a UTC timestamp with two digits for each date and time component, a literal `T`, and a literal `Z`. The supported pattern tokens are:

| Token | Meaning |
| --- | --- |
| `YYYY` | Four-digit year |
| `MM` | Two-digit month |
| `DD` | Two-digit day |
| `HH` | Two-digit hour (UTC, 00–23) |
| `mm` | Two-digit minute |
| `ss` | Two-digit second |

For example, `YYYY-MM-DD` produces `2025-01-31`, while `HH:mm:ss` produces `12:34:56`.

## Formatting `now`

Use `now` to obtain a current UTC timestamp, then format it:

```shelly
now -> $timestamp
formatDate $timestamp "YYYY-MM-DD HH:mm:ss" -> $readable
```

`timestamp` is an alias for `now`. `today` returns the UTC date without a time component. `formatDate` does not convert to local time: every result uses the UTC values in the input timestamp. See the [Time and date route](index.md#time-and-date) for the related helpers.

## Failures

`formatDate` fails clearly when:

- the input is not in the strict `YYYY-MM-DDTHH:mm:ssZ` form;
- the timestamp contains invalid date or time values; or
- the pattern contains unsupported token letters or malformed tokens.

Use only the six supported tokens listed above. Literal separators such as `/`, `-`, spaces, and `:` are allowed in patterns.
