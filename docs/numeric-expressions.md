# Numeric expressions

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

A numeric expression is a scalar value evaluated with floating-point arithmetic. It is not a user-visible integer or decimal type: a value can be written as an integer or decimal, while a particular context may require an integer.

```shelly
$items = 3
$price = 10.5
$total = $items * $price

shellout $total
```

## Literals

Accepted decimal forms include integer-looking numbers and numbers with digits on both sides of the decimal point: `0`, `12`, `12.5`, and `0.25`. Exponential notation, hexadecimal, comma-separated numbers, and a leading-dot form are not numeric literals.

```shelly
$whole = 12
$decimal = 12.50
$small = 0.25
```

These are invalid numeric expressions (shown as text, not runnable Shelly):

```text
$bad = 1e3       # exponent notation
$bad = 0x10      # hexadecimal
$bad = 1,000     # comma-separated number
$bad = .5        # leading-dot form
```

## Operators and evaluation

The supported binary operators are `+`, `-`, `*`, and `/`. Multiplication and division have precedence over addition and subtraction. Operators at the same precedence level evaluate from left to right. Parentheses override precedence. A unary `+` or `-` may prefix an operand, including a parenthesized operand.

```shelly
$subtotal = 3 * (12 + 2)
$change = 50 - $subtotal
$temperature = -(2 + 3)
```

For example, a $50 grocery budget with three $12 items and a $2 fee is evaluated as `50 - (3 * 12 + 2)`, leaving `$12`:

```shelly
$remaining = 50 - (3 * 12 + 2)
shellout $remaining
```

These forms are invalid: unsupported operators, incomplete or adjacent operands, unmatched parentheses, and trailing text all fail.

```text
$bad = 2 ^ 3       # unsupported operator
$bad = 2 +         # incomplete expression
$bad = 2 3         # adjacent operands
$bad = (2 + 3      # unmatched parenthesis
$bad = 2 + 3 later # trailing text
```

## Variables and constants

Numeric variables and constants can participate in expressions. A nonnumeric value (such as a string, boolean, list, or dictionary), a collection, or an undefined reference fails when used as an operand. See [variables](variables.md) and [constants](constants.md) for naming, values, and lookup rules.

```shelly
$tax = 2.5
$BASE = 10
$total = $BASE + $tax
```

## Contexts and limits

Expressions are accepted wherever Shelly needs a numeric value, including:

- assignments, such as `$total = $price * $count`;
- [`repeat`](repeat.md) counts, which must be non-negative integers;
- [`while`](while.md) comparisons, such as `while ($count < 3)`;
- [list indexes](lists.md), which are one-based integers;
- [`arg`](commands.md) argument indexes, which are one-based positions;
- [`exit`](exit.md) codes, which must be integers from `0` through `255`.

A context can therefore reject an otherwise valid fractional or negative result. Division by zero always fails.

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

Integral results print without `.0`; non-integral results print as decimals. Ordinary floating-point precision limits apply, so calculations involving fractional values can have the usual small rounding differences of floating-point arithmetic.

```shelly
$result = 6 / 2
shellout $result
```

This prints `3`, not `3.0`. Division by zero is invalid:

```text
$bad = 10 / 0       # division by zero
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
