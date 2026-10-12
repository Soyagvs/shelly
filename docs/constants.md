# Constants

Constants store immutable values and use `$$NAME` syntax.

```shelly
$$APP = "Shelly"
$$BASE = 10
shellout "running $$APP"
```

## Rules

- Constants start with two dollar signs.
- Constant names must be uppercase.
- Constants require an initial value.
- Constants cannot be reassigned.
- Constants use the same identifier character rules as variables: start with an ASCII letter, then ASCII letters, numbers, or `_`.

## Use in expressions

```shelly
$$BASE = 10
$total = $$BASE * 2
shellout $total
```

Constants can participate in numeric expressions when their value is numeric.
