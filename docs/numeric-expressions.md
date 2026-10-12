# Numeric expressions

Shelly supports integer and floating-point numeric expressions.

```ly
$a = 1 + 2 * 3
$b = ($a + 4) / 2
$c = 10.5 + 2.25
$d = -3 * 2
$$BASE = 10
$total = $$BASE * $c
shellout $total
```

## Operators

- `+` addition
- `-` subtraction
- `*` multiplication
- `/` division

Multiplication and division bind before addition and subtraction. Parentheses group subexpressions.

## Rules

- Negative numbers and unary `+` / `-` are supported.
- Variables and constants can participate when their current value is numeric.
- Division by zero is an error.
- Strings, booleans, lists, and empty values are not numeric operands.
- Malformed expressions fail with a line-numbered diagnostic.
- Integer-looking results print without `.0`, so `6 / 2` prints `3`.
- Decimal results print as decimals, such as `2.5`.
