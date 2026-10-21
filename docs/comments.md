# Comments

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Comments are notes for people reading a script. Shelly ignores them, so they help explain *why* something exists without changing what the script does.

```shelly
# Greet the person running this script #
shellout "hello"
```

## When to use comments

Use a comment when the intent is not obvious from the code: explain a decision, a limitation, or a surprising requirement. Do not repeat what a clear instruction already says.

```shelly
# The API accepts dates only in UTC. #
$DATE_FORMAT = "UTC"

# Avoid: this only repeats the instruction. #
shellout "ready"
```

## Rules

- Comments open with `#` and close with `#`.
- A one-line comment opens and closes on the same line.
- A block comment may span many lines.
- An unclosed comment is an error.
- `#` inside a string is normal text.

## Block comments

```shelly
#
This whole block is ignored.
Use it for longer notes.
#
shellout "after the comment"
```

## Strings are safe

```shelly
shellout "# this prints as text #"
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
