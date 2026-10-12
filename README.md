# Shelly

Shelly is a Bash 2.0 style shell and script runner for `.ly` scripts.

## Status

This repository contains an initial line-oriented `.ly` interpreter.

## Usage

```sh
cargo run -- path/to/script.ly
```

Running without a script prints a short usage hint and exits successfully.

## MVP syntax

```ly
$name = "world"
$empty
$count = 1 + 2 * 3
$enabled = true
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2 + 3, 6]
$$APP = "Shelly"
$$BASE = 10
$total = $$BASE * ($count + 2.5)
shellout "hello $name from $$APP"
shellout $total
append $names, "Iris"
prepend $names, "Ada"
$names[last] = "Mora"
remove $names[first]
shellout "Second name: $names[2]"
shellout $numbers[2]
command "printf captured" -> $value
command "printf running"
shellin $answer
ask "Continue? " -> $answer
exit 0

if $enabled
    shellout "enabled"
elif $count > 10
    shellout "large"
else
    shellout "disabled or small"
end

repeat 3 -> $turn
    shellout "turn $turn"
end

while $count < 10
    shellout "count $count"
    $count = $count + 1
end

fn greet $name, $prefix = "Hola"
    shellout "$prefix $name from $$APP"
end
greet $name

import "examples/module.ly"
include "examples/setup.ly"
```

Core rules:

- Blank lines are ignored.
- Comments use explicit closing markers: `# line comment #` for one line, or `#` ... `#` across multiple lines.
- A `#` inside a string is literal text, not a comment marker.
- Variables use `$name`, are case-insensitive, and may be declared empty with a bare `$name`.
- Constants use `$$NAME`, must be uppercase, require an initial value, and cannot be reassigned.
- Identifiers must start with an ASCII letter, followed by ASCII letters, numbers, or `_`.
- Assign strings, numbers, booleans, lists, or numeric expressions with `$name = "value"`, `$count = 42`, `$enabled = true`, or `$total = $$BASE * ($count + 2.5)`.
- Strings use double quotes, may contain single quotes as normal text, and support escapes such as `\n`, `\t`, `\"`, and `\\`.
- Numeric expressions support integers, floats, `+`, `-`, `*`, `/`, parentheses, negative numbers, variables, and constants; division by zero and non-numeric operands are errors.
- Integer-looking numeric results print without `.0`, while decimal results print as decimals.
- Lists use bracket literals such as `[]` or `["Ana", 3, true]`; elements can be strings, numbers/numeric expressions, booleans, variables, or constants. Nested lists are not supported yet.
- Select lists with one-based `$list[1]`, `$list[first]`, `$list[last]`, multi-index `$list[1,3]`, inclusive ranges like `$list[..3]`/`$list[3..]`, and full-copy `$list[..]` in `shellout` or interpolation.
- List filters return new lists: numeric filters support `==`, `!=`, `>`, `>=`, `<`, `<=`; string and boolean filters support `==` and `!=`; mixed lists support exact `==` and `!=` matching. Ordered string filters and ordered filters over non-numeric items are errors.
- Invalid, out-of-range, reverse-range, malformed, or non-list selection is an error. Mutate variable lists with `append $list, value`, `prepend $list, value`, `remove $list[first|last|index]`, and `$list[index|first|last] = value`; mutation indexes must select exactly one item, so multi-indexes, ranges/full-copy selectors, and filters are rejected for mutation targets.
- Capture command output with `command "printf value" -> $name`; captured stdout has trailing newlines trimmed and is not printed automatically.
- Print values with `shellout "hello $name $$CONST"` or `shellout $name`.
- Read one input line with `shellin $name`.
- Prompt and read one input line with arrow binding: `ask "Question?" -> $name`. The destination must be a variable. In Shelly, `->` means a command result is bound into a variable; commas remain for ordinary argument lists.
- Stream a shell command with `command "printf hello"`.
- Exit the whole script immediately with `exit` or `exit code`; codes must resolve to an integer from 0 to 255. Numeric expressions, variables, and constants are supported for codes.
- Branch with `if condition`, optional `elif condition`, optional `else`, and closing `end`.
- Repeat blocks with `repeat count` or `repeat count -> $index`, and closing `end`; counts must be non-negative integers, `repeat 0` skips, and the optional one-based index is local/protected.
- While blocks with `while condition` repeat until the condition is false, close with `end`, share `if` condition rules, do not create a new scope, and stop with an error after 10,000 iterations.
- Conditional comparisons support `==`, `!=`, `>`, `>=`, `<`, and `<=`.
- Direct conditions use truthiness: `true` is true; `false`, `""`, and `0` are false; everything else is true.
- `if` and `while` blocks do not create a new scope; assignments in the selected branch or loop body affect the current scope.
- Complex conditional expressions with `and`, `or`, `not`, or parentheses are not implemented yet.
- Define functions with `fn ... end`, call them by name, and pass comma-separated literal, variable, or constant arguments.
- Export public constants with `export $$NAME = value` and public functions with `export fn name ... end`.
- `import "file.ly"` loads only exported constants/functions and does not run private top-level code.
- `include "file.ly"` executes the whole file in the current scope, like copying it at that point.
- Import/include paths are relative to the file that declares them; missing files and cycles are line-numbered errors.
- Normal variables are not exportable yet; `export $name = "x"` is rejected.

See [docs/language.md](docs/language.md) for the full current language guide and the list of syntax ideas that are not implemented yet.
