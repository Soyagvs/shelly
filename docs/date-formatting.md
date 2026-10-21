# Date formatting

[Documentation index](index.md) · [Time and date helpers](helpers.md#time)

`formatDate` turns a Shelly UTC timestamp into readable text. It replaces a small, predictable set of tokens in the pattern you provide. Results are numeric, UTC-only, and not locale-formatted.

<img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />

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
| `YYYY` | Four-digit year (0000–9999) |
| `MM` | Two-digit month |
| `DD` | Two-digit day |
| `HH` | Two-digit hour (UTC, 00–23) |
| `mm` | Two-digit minute |
| `ss` | Two-digit second |

For example, `YYYY-MM-DD` produces `2025-01-31`, while `HH:mm:ss` produces `12:34:56`. `YYYY` parses and formats the input's four year digits; it does not add a calendar era or locale-specific year name.

ASCII letters are reserved for tokens: any ASCII alphabetic text other than the six supported tokens fails. Non-letter characters—including punctuation, spaces, and non-ASCII characters—are copied literally; there is no quoting or escaping syntax.

## Formatting `now`

Use `now` to obtain a current UTC timestamp, then format it:

```shelly
now -> $timestamp
formatDate $timestamp "YYYY-MM-DD HH:mm:ss" -> $readable
```

`timestamp` is an alias for `now`. `today` returns the UTC date without a time component. `formatDate` does not convert to local time: every result uses the UTC values in the input timestamp. See the [Time and date helpers](helpers.md#time) for the related helpers.

## Failures

`formatDate` fails clearly when:

- the input is not exactly `YYYY-MM-DDTHH:mm:ssZ` (no offsets, fractions, or other time zones);
- the timestamp contains invalid date or time values; or
- the pattern contains unsupported ASCII token letters.

The input year and all other components are numeric and fixed-width: months are `01`–`12`, hours `00`–`23`, minutes and seconds `00`–`59`, and dates must be valid. Use only the six supported tokens listed above. Literal separators such as `/`, `-`, spaces, `:`, and non-ASCII text are allowed in patterns; names, locale formatting, and timezone conversion are not supported.
