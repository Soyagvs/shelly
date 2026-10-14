# Constants

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Constants store immutable values and use one dollar sign with uppercase-only names.

```shelly
$APP = "Shelly"
$BASE = 10

shellout "running $APP"
```

## Rules

- Constants start with `$` and use uppercase-only names.
- Constants require an initial value.
- Constants cannot be reassigned.
- Constants follow the same identifier character rules as variables.

## Use in expressions

```shelly
$BASE = 10
$total = $BASE * 2

shellout $total
```

## Use in lists and loops

```shelly
$NAMES = ["Ana", "Luis"]

for $name in $NAMES
    shellout $name
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
