# Comments

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Comments are notes for readers. Shelly ignores them when they use matching `#` markers outside strings.

```shelly
# Say hello #
shellout "hello"
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
