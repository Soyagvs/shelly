# Shelly language guide

Shelly scripts use the `.ly` extension. The interpreter is line-oriented: one instruction per line, no semicolons.

## Guided mental model

Start with values, then commands, then blocks. `$name` stores a mutable variable, `$$NAME` stores an immutable constant, and most commands read values from ordinary comma-separated arguments. The arrow `->` is special: it means the command produces or binds a result into a variable, as in `ask "Name? " -> $name`, `command "pwd" -> $cwd`, and `repeat 3 -> $i`. Use commas for normal argument lists; use `->` only for result binding.

Run a script with:

```sh
cargo run -- path/to/script.ly
```

## Comments

Comments must start and end with `#` markers outside strings.

Line comments start and end on the same line:

```ly
# explain the next command #
shellout "hello"
```

Block comments start with `#` and end with a later `#`:

```ly
#
This whole block is ignored.
Use it for notes that span lines.
#
shellout "after the comment"
```

A comment that reaches the end of the file without a closing `#` is an error with the opening line number. `#` characters inside strings are literal text, so `shellout "# not a comment #"` prints the string.

## Variables

```ly
$name = "world"
$empty
$count = 42
$enabled = true
shellout "hello $NAME"
```

- Variables use `$name` syntax.
- Variable names are case-insensitive.
- A bare variable declaration, such as `$empty`, creates an empty value.
- Values may be strings, numbers, booleans (`true` / `false`), or numeric expressions. Command output can be captured into variables with `command "..." -> $name`.
- Identifiers must start with an ASCII letter, followed by ASCII letters, numbers, or `_`.

## Constants

```ly
$$APP = "Shelly"
shellout "running $$APP"
```

- Constants use `$$NAME` syntax.
- Constant names must be uppercase.
- Constants require an initial value and cannot be reassigned.
- Constants follow the same identifier character rules as variables.

## Lists

```ly
$empty = []
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2, 3]
$mixed = ["Ana", 3, true]
$name = "Ada"
$$AGE = 37
$refs = [$name, $$AGE, false]

shellout $names[1]
shellout $names[first]
shellout $names[last]
shellout $names[1,3]
shellout $names[1..2]
shellout $names[..]
shellout $numbers[>1]
shellout $names[=="Ana"]
shellout "Second: $names[2]"
shellout $refs[1]
shellout $refs[2]
```

- Shelly has one collection type: list.
- Lists can be assigned to variables with `[]` or comma-separated values inside brackets.
- First-version list elements may be strings, numbers or numeric expressions, booleans, variables, and constants.
- Nested lists are not supported yet.
- Index and selector access uses `$list[...]` and is 1-based to match `repeat ... -> $i`.
- Scalar selectors are `$list[1]`, numeric expressions such as `$list[$i + 1]`, `$list[first]`, and `$list[last]`.
- Multi-selectors return a new list formatted as `[item, item]`: `$list[1,3]`, inclusive ranges `$list[1..3]`, `$list[..3]`, `$list[3..]`, `$list[..]`, and `$list[first..last]`.
- Filters return a new list. Numeric filters support `==`, `!=`, `>`, `>=`, `<`, and `<=`; string and boolean filters support `==` and `!=`; mixed lists support exact `==` and `!=` matching.
- Ordered filters require numeric comparison values and numeric list items. Ordered string comparisons such as `$names[>"Ana"]` are not supported yet.
- Indexing, selectors, and filters work in `shellout` and interpolation like `shellout "Selected: $names[1,3]"`.
- Index `0`, negative indexes, decimal indexes, malformed selector syntax, reverse ranges, out-of-range access, undefined lists, and indexing non-list values are errors.
- Mutate variable lists with `append $list, value`, `prepend $list, value`, `remove $list[first]`, `remove $list[last]`, `remove $list[3]`, and `$list[2] = value`.
- Mutation targets must be variable lists. Constants, undefined values, non-lists, and temporary selector/filter results are rejected.
- Mutation indexes are 1-based and must select exactly one item; `first` and `last` are allowed. Multi-indexes, ranges/full-copy selectors, and filters are not mutation targets yet.
- Out-of-range mutation indexes are errors, and `first`/`last` mutations on an empty list are errors.
- Mutation values may be the same scalar element forms as list literals: strings, numbers/numeric expressions, booleans, variables, and constants. Nested lists remain unsupported.

```ly
$names = ["Ana", "Luis"]
append $names, "Mora"
prepend $names, "Iris"
$names[2] = "Ada"
remove $names[last]
remove $names[first]
shellout $names[..]
```

## Numeric expressions

```ly
$a = 1 + 2 * 3
$b = ($a + 4) / 2
$c = 10.5 + 2.25
$d = -3 * 2
$$BASE = 10
$total = $$BASE * $c
shellout $total
```

- Numeric expressions support integers and floating-point numbers.
- Operators are `+`, `-`, `*`, and `/`; multiplication and division bind before addition and subtraction.
- Parentheses group subexpressions.
- Negative numbers and unary `+` / `-` are supported.
- Variables and constants can participate when their current value is numeric.
- Division by zero is an error.
- Strings and booleans are not valid numeric operands.
- Malformed expressions fail with a line-numbered diagnostic.
- Integer-looking results print without `.0`, so `6 / 2` prints `3`; decimal results print as decimals, such as `2.5`.

## Commands and command capture

```ly
shellout "hello"
shellout $name
shellin $answer
ask "Name? " -> $name
command "printf streamed"
command "printf captured" -> $value
```

- `shellout` prints the evaluated value and adds a newline.
- `shellin $name` reads one line from stdin and stores it without the line ending.
- `ask "Question?" -> $name` prints the prompt, flushes stdout, reads one line from stdin, trims the line ending, and stores it in a variable.
- The prompt before `->` must be a string.
- The destination after `->` must be a variable (`$name`). Strings, constants, and other values are rejected.
- The old comma form is rejected with migration guidance to the arrow form.
- `ask` is not print-only; use `shellout` when you only need to print.
- `command "command"` executes through `sh -c` and streams output directly. Non-zero status fails the script.
- `command "command" -> $name` executes through `sh -c`, captures stdout into a variable, trims trailing newlines, and does not print captured stdout automatically. Non-zero status fails the script.
- Capture destinations must be variables (`$name`), not constants (`$$NAME`) or other values.
- The old `capture` syntax is removed; use `command "..." -> $name` instead.
- Shell commands run through the operating system shell, so system-shell quoting rules apply inside the command string.

## Exit

```ly
if $answer != "yes"
    shellout "Cancelled"
    exit 1
end

$code = 0
exit $code
```

- `exit` terminates the whole Shelly process immediately with status code `0`.
- `exit code` terminates the whole process with the resolved code.
- Codes may be integer literals, numeric expressions, variables, or constants, and must resolve to an integer from `0` to `255`.
- `exit` works at top level, inside `if`, `repeat`, `while`, functions, included files, and imported/exported functions when they run.
- `exit` exits the whole script, not just the current block, loop, or function.
- Strings, booleans, lists, decimal values, negative values, values greater than `255`, and malformed trailing text are line-numbered errors and do not pretend to exit successfully.

## Functions

```ly
$outside = "outer"
$$APP = "Shelly"

fn greet $name, $age, $prefix = "Hola"
    global $outside, $$APP
    shellout "$prefix $name age=$age from $$APP with $outside"
    $outside = "local only"
end

greet $outside, $$APP, 27
shellout $outside
```

- Function definitions start with `fn` and end with `end`.
- Functions are called by name: `greet "Agustin", 27` or `greet $name, $$APP`.
- Definitions use comma-separated parameters: `fn person $name, $age`.
- Calls use comma-separated arguments: `person "Agustin", 27`. Arguments may be literal strings, numbers, booleans, variables (`$name`), or constants (`$$APP`).
- Parameters use `$` variable syntax and obey the normal variable identifier rules.
- Parameters can have defaults: `fn greet $name, $prefix = "Hola"`.
- Defaulted parameters must come after required parameters. A required parameter after a default is an error.
- Function scope is local by default. Assignments inside a function do not mutate outer variables.
- Outer variables and constants require `global` declarations before use inside a function.
- `global` supports comma-separated identifiers: `global $name, $age` and `global $$NAME, $$AGE`.
- Initial `global` behavior is read-only for outer values: globals can be read inside a function, but assignments inside the function remain local and do not mutate the outer value.
- Constants remain immutable.
- Functions do not return values yet.

## Conditionals

```ly
$enabled = true
$count = 3

if $enabled
    shellout "enabled"
elif $count >= 10
    shellout "large"
else
    shellout "fallback"
end
```

- `if condition` starts a conditional block and `end` closes it.
- `elif condition` adds optional middle branches. `else` adds one optional fallback branch.
- Syntax uses no parentheses, no `then`, and no braces.
- Conditions can be direct truthy values such as `if $enabled`, `if "text"`, or `if 1`.
- Comparisons support `==`, `!=`, `>`, `>=`, `<`, and `<=`.
- Condition values use the normal value forms: strings, numbers, booleans, numeric expressions, variables, constants, and string interpolation.
- Truthiness: `true` is true; `false`, an empty string, and number `0` are false; everything else is true.
- Conditional blocks do not create a new scope. Assignments inside the taken branch affect the current scope.
- Complex expressions are not implemented yet: `and`, `or`, `not`, and parenthesized conditions are rejected.

## While loops

```ly
$count = 0
while $count < 3
    shellout "Count $count"
    $count = $count + 1
end
shellout "Done at $count"

$enabled = true
while $enabled
    shellout "once"
    $enabled = false
end
```

- `while condition` repeats its block while the condition is true and closes with `end`.
- `while` uses the same condition rules as `if`: truthiness and comparisons with `==`, `!=`, `>`, `>=`, `<`, and `<=`.
- Condition values use normal value forms, including variables, constants, and numeric expressions where condition handling supports them.
- If the condition is false before the first iteration, the body is skipped.
- While blocks do not create a new scope. Assignments inside the loop affect the current scope and persist after the loop.
- To guard against accidental infinite loops, a while loop errors after 10,000 iterations.
- Nested `if`, `repeat`, and `while` blocks are supported.

## Repeat loops

```ly
repeat 3
    shellout "Hola"
end

repeat 3 -> $turn
    shellout "Vuelta $turn"
end

$count = 0
repeat 3
    $count = $count + 1
end
shellout "count=$count"
```

- `repeat count` runs its block exactly `count` times and closes with `end`.
- The count may be a numeric expression using numbers, variables, and constants.
- Counts must be integer values. Negative and decimal counts are errors.
- `repeat 0` skips the block.
- `repeat count -> $name` binds a one-based iteration variable inside the block.
- The index destination must be a normal variable (`$name`), not a constant or bare name.
- The optional index variable is protected/local to the repeat block and restores any outer variable with the same name after the loop.
- Other assignments inside repeat blocks use the current scope and persist after the loop.

## Imports, includes, and exports

Use exports to make a file's public constants and functions available to importers:

```ly
# module.ly #
export $$GREETING = "Hola"

export fn greet $name
    global $$GREETING
    shellout "$$GREETING $name"
end

shellout "not run by import"
```

```ly
# main.ly #
import "module.ly"
greet "Ada"
shellout $$GREETING
```

- `export $$NAME = value` exports a constant. Exported constants follow normal constant rules: uppercase names, initial values, and no reassignment.
- `export fn name ... end` exports a function.
- Normal variables are not exportable yet. `export $name = "x"` is rejected.
- `import "file.ly"` loads only exported constants and functions from the target file.
- `import` does not execute non-exported top-level code in the target file.
- Private variables, private constants, and private functions in an imported file are unavailable unless exported.
- Duplicate imports of the same file are ignored after the first successful import.
- `include "file.ly"` executes the whole target file in the current scope, like copying that file's contents at the include line.
- Included files can set variables and constants or define functions for later lines in the including file.
- Import and include paths are resolved relative to the file that declares them, not necessarily the process current working directory.
- Missing files and import/include cycles are errors with line numbers.

## Strings and interpolation

Strings use double quotes:

```ly
$message = "hello"
shellout "$message from Shelly"
```

Single quotes are normal characters inside Shelly strings. Use them when you want quoted text in the output:

```ly
shellout "texto 'mas texto'"
```

Supported escapes:

| Escape | Meaning | Example |
| --- | --- | --- |
| `\n` | newline | `shellout "linea 1\nlinea 2"` |
| `\t` | tab | `shellout "nombre\tvalor"` |
| `\"` | double quote | `shellout "dijo \"hola\""` |
| `\\` | backslash | `shellout "C:\\tmp"` |

Variable and constant interpolation works inside strings with `$name` and `$$CONST`. Undefined variables and constants are errors.

```ly
$name = "Agustin"
shellout "hola $name\nesto sale en otra linea"
```

## Identifier rules

| Kind | Prefix | Case rule | Valid examples | Invalid examples |
| --- | --- | --- | --- | --- |
| Variable | `$` | Case-insensitive | `$name`, `$Name`, `$user_1` | `$1name`, `$_name`, `$-name` |
| Constant | `$$` | Must be uppercase | `$$NAME`, `$$API_URL`, `$$URL2` | `$$name`, `$$1NAME`, `$$_NAME` |

Identifiers must start with a letter. After the first letter, they may contain letters, numbers, or `_`. They cannot start with numbers, `_`, or other symbols. They cannot contain spaces.

## Not implemented yet

- HTTP/fetch helpers.
- Test syntax.
- Inline comments after code.
- Multi-line strings or commands.
