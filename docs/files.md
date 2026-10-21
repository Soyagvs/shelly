# Files

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly has first-class file commands for common script automation, plus `files` for collecting matching paths.

All relative paths, including `./`, resolve from the current Shelly script file directory, not from the process current working directory. `import`, `include`, `exists`, and `files` use the same script-relative model. This is not a sandbox: `..` is allowed and can escape the script directory, for example `write "../shared/out.txt" "hello"` writes in its parent directory.

## Path commands

```shelly
create "out/"
create "out/report.txt"
write "out/report.txt" "hello"
append "out/report.txt" "next line"
read "out/report.txt" -> $text
size "out/report.txt" -> $bytes
isFile "out/report.txt" -> $isfile
isDir "out/" -> $isdir
tempFile -> $scratchfile
tempDir -> $scratchdir
copy "out/report.txt" "copy/report.txt"
move "copy/report.txt" "copy/moved.txt"
rename "copy/moved.txt" "final.txt"
remove "copy/final.txt"
remove "out/"
```

## Path command rules

- `create "path"` creates an empty file and parent directories. If `path` ends with `/`, it creates a directory recursively. Existing targets succeed and existing file content is preserved.
- `copy "from" "to"` copies a file or directory recursively, creates destination parents, and fails if `to` already exists.
- `move "from" "to"` moves a file or directory, creates destination parents, and fails if `to` already exists.
- `rename "from" "new-name"` renames within the same directory. `new-name` must be a plain name, not a path, and the destination must not already exist.
- `remove "path"` succeeds when the path is missing. It deletes files. It deletes directories recursively only when the input path ends with `/`; a directory path without trailing `/` fails with a safety error.
- `read "file" -> $text` reads UTF-8 text from a file into a variable. Missing paths and directories fail.
- `write "file" $text` writes a string expression to a file, creates parent directories, and overwrites existing file content. Directories fail.
- `append "file" $text` creates parent directories when needed, appends the evaluated text, and adds one trailing newline for log-friendly output. It does not overwrite existing content. Directories fail. File append also accepts a variable path like `append $path $text`. List append uses the same whitespace-separated form: `append $list $value`.
- `isFile "path" -> $ok` stores `true` when the path exists and is a file, otherwise `false`. Missing paths do not fail.
- `isDir "path" -> $ok` stores `true` when the path exists and is a directory, otherwise `false`. Missing paths do not fail.
- `size "file" -> $bytes` stores the file size in bytes. Missing paths and directories fail.
- `tempFile -> $path` creates an empty temporary file and stores its absolute path.
- `tempDir -> $path` creates a temporary directory and stores its absolute path.

## Mutation and failure examples

Path commands mutate the filesystem immediately. `write` overwrites, `append` preserves existing content and adds a trailing newline, `copy` and `move` create destination parents, and `remove "dir/"` deletes a directory recursively. Destination conflicts fail rather than overwrite:

```shelly
copy "report.txt" "archive/report.txt" # fails if the destination exists
move "report.txt" "archive/report.txt" # fails if the destination exists
rename "report.txt" "archive/report.txt" # fails: rename takes a plain name
remove "archive"                    # fails: recursive removal needs "archive/"
read "missing.txt" -> $text          # fails: the path is missing
```

`create`, `write`, and `append` create missing parent directories. `isFile` and `isDir` are the exceptions: missing paths simply produce `false`.

## Commas in the language

Commas are reserved for structural contexts such as data literals, function signatures, selectors, and multi-capture destinations (`-> $out, $err, $code`). They are not command argument separators or `when` alternative separators.

## Script-relative paths

```shelly
create "./build/"
write "./build/message.txt" "from this script directory"
append "./build/message.txt" "another line"
read "./build/message.txt" -> $message
shellout $message
```

Running that script from another terminal directory still writes under the script's own directory.

## Collect matching files

`files` collects matching file paths into a Shelly list.

```shelly
files "*.txt" -> $items
```

## `files` rules

- Syntax: `files "pattern" -> $items`.
- The destination after `->` must be a variable.
- Matches are resolved relative to the current script file directory.
- The result is a list of matching paths as strings.
- `files` only assigns the list. It does not print or iterate by itself.
- No matches returns an empty list: `[]`.
- Results are sorted deterministically.

## Patterns

Patterns support literal segments, single-segment `*`, and recursive `**`:

```shelly
files "*.txt" -> $textfiles
files "dir/*.txt" -> $dirtextfiles
files "**/*.shelly" -> $all_shelly_files
files "src/**/test_*.shelly" -> $tests
files "README.md" -> $readme
```

`*` matches only within one path segment. `**` matches zero or more directory segments, so `**/*.shelly` includes files in the current directory and every descendant. Recursive traversal does not follow symlinked directories; a symlink to a directory is not traversed or returned. A symlink to a file may match a non-recursive pattern, but matches are files only and directories are never returned. Unreadable or absent paths produce no match. Results are sorted by normalized relative path: each returned string is normalized relative to the script directory (with `.` and `..` resolved and `/` separators), not an absolute path.

## Print the list

```shelly
files "*.txt" -> $items
shellout $items
```

This prints the list using Shelly's normal list formatting, for example:

```text
[a.txt, b.txt]
```

## Iterate over matches

Use [`for`](for.md) to visit each path.

```shelly
files "*.txt" -> $items
for $file in $items
    shellout "found $file"
end
```

Empty lists are falsey, so you can branch on whether any files matched:

```shelly
files "*.txt" -> $items
if $items
    shellout "found files"
else
    shellout "no matches"
end
```

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

## Helper command examples

```shelly
isFile "data/input.txt" -> $hasinput
isDir "data" -> $hasdatadir

write "logs/run.log" "started"
append "logs/run.log" "loaded input=$hasinput"
size "logs/run.log" -> $logbytes

tempFile -> $scratchfile
tempDir -> $scratchdir
write $scratchfile "temporary content"
isFile $scratchfile -> $scratchisfile
isDir $scratchdir -> $scratchisdir
```

`tempFile` and `tempDir` paths are unique enough for repeated calls in one script and are returned as absolute strings. Single-path helpers such as `create`, `write`, `append`, `read`, `size`, `isFile`, and `remove` accept those variables as path expressions, while literal paths remain script-relative.
