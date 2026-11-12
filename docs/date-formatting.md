# Format UTC timestamps as text

Use `formatDate` to turn a strict Shelly UTC timestamp into readable text with six predictable tokens. Formatting is numeric and locale-independent; it does not convert time zones.

## Quick path

```shelly
formatDate "2025-01-31T12:34:56Z" "YYYY/MM/DD HH:mm:ss" -> $formatted
```

The input must be exactly `YYYY-MM-DDTHH:mm:ssZ`: fixed-width numeric fields, a literal `T`, and a literal `Z`.

## Supported tokens

| Token | Meaning |
| --- | --- |
| `YYYY` | Four-digit year (`0000`–`9999`) |
| `MM` | Two-digit month |
| `DD` | Two-digit day |
| `HH` | Two-digit UTC hour (`00`–`23`) |
| `mm` | Two-digit minute |
| `ss` | Two-digit second |

For example, `YYYY-MM-DD` produces `2025-01-31`, and `HH:mm:ss` produces `12:34:56`. Literal punctuation, spaces, and non-ASCII text are copied. ASCII letters are reserved: any ASCII alphabetic text other than the six tokens fails. There is no quoting or escaping syntax.

## Format the current time

```shelly
now -> $timestamp
formatDate $timestamp "YYYY-MM-DD HH:mm:ss" -> $readable
```

`timestamp` is an alias for `now`; `today` returns the current UTC date without a time component. See [Time helpers](helpers.md#time-random-and-json) for the related standard helpers.

## Failures

`formatDate` raises an ordinary runtime error when:

- the input is not exactly the required UTC form (offsets and fractional seconds are not accepted);
- date or time values are invalid; or
- the pattern contains unsupported ASCII letters.

Months are `01`–`12`, hours `00`–`23`, and minutes and seconds `00`–`59`; the date must be valid. Use [Error handling](errors.md) when formatting is expected to fail.
