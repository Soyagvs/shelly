# Numeric expressions

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Numeric expressions support integer and decimal arithmetic.

```shelly
$a = 1 + 2 * 3
$b = ($a + 4) / 2
$c = 10.5 + 2.25
$d = -3 * 2

shellout $b
```

## Rules

- Supported operators: `+`, `-`, `*`, and `/`.
- Multiplication and division run before addition and subtraction.
- Parentheses group subexpressions.
- Negative numbers and unary `+` / `-` are supported.
- Variables and constants may participate when their values are numeric.
- Division by zero is an error.
- Strings, booleans, lists, and empty values are not numeric operands.

## Constants

```shelly
$$BASE = 10
$total = $$BASE * 2

shellout $total
```

Integer-looking results print without `.0`. Decimal results print as decimals.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
