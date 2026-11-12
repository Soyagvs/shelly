# Calculate values with numeric expressions

Numeric expressions evaluate scalar values with floating-point arithmetic. Shelly has no user-visible integer or decimal type: a literal may look integral or decimal, while the surrounding context can require an integer.

## 🚀 At a glance

```shelly
$items = 3
$price = 10.5
$total = $items * $price
shellout $total
```

- Supported binary operators: `+`, `-`, `*`, `/`.
- `*` and `/` bind before `+` and `-`; equal-precedence operators run left to right.
- Parentheses and unary `+` or `-` are supported.
- Division by zero and invalid operands fail.

## Numeric literals

Accepted decimal forms include `0`, `12`, `12.5`, and `0.25`. Exponential notation, hexadecimal, comma-separated numbers, and leading-dot forms are not numeric literals:

```shelly
$whole = 12
$decimal = 12.50
$small = 0.25
```

```text
$bad = 1e3
$bad = 0x10
$bad = 1,000
$bad = .5
```

## Operators and evaluation

```shelly
$subtotal = 3 * (12 + 2)
$change = 50 - $subtotal
$temperature = -(2 + 3)
$remaining = 50 - (3 * 12 + 2)
shellout $remaining
```

The last expression leaves `12`. Unsupported operators, incomplete or adjacent operands, unmatched parentheses, and trailing text fail:

```text
$bad = 2 ^ 3
$bad = 2 +
$bad = 2 3
$bad = (2 + 3
$bad = 2 + 3 later
```

## Variables and constants

Numeric variables and constants can be operands. Strings, booleans, lists, dictionaries, and undefined references cannot:

```shelly
$tax = 2.5
$BASE = 10
$total = $BASE + $tax
```

See [Variables](variables.md) and [Constants](constants.md) for naming and lookup rules.

## Contexts and limits

Expressions are accepted wherever Shelly needs a numeric value:

| Context | Additional requirement |
| --- | --- |
| Assignment | Any valid numeric result |
| [`repeat`](repeat.md) count | Non-negative integer |
| [`while`](while.md) comparison | Numeric comparison operands |
| [List index](lists.md) | Existing one-based integer |
| [`arg`](commands.md) index | One-based position |
| [`exit`](exit.md) code | Integer from `0` through `255` |

A valid expression can therefore be rejected by its context when its result is fractional or negative.

```shelly
$count = 1
while ($count < 3)
    $count = $count + 1
end

repeat 2 + 1
    shellout "working"
end
```

## Output and precision

Integral results print without `.0`; non-integral results print as decimals. Ordinary floating-point precision limits apply.

```shelly
$result = 6 / 2
shellout $result
```

This prints `3`, not `3.0`. Division by zero is invalid:

```text
$bad = 10 / 0
```

## Related topics

- [Variables](variables.md) and [Constants](constants.md) — numeric values and names.
- [Lists](lists.md) — one-based indexes and ranges.
- [Repeat loops](repeat.md), [While loops](while.md), and [Exit](exit.md) — numeric contexts.
