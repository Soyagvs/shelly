# Create and validate regular-file archives

Shelly archive v1 creates and extracts deterministic ZIP or TAR.GZ archives, validates them, and computes SHA-256 checksums. Paths are relative to the current Shelly script directory, like [file commands](files.md).

## Quick path

```shelly
archive create "source-dir" from "backup.zip" -> $created
archive check "backup.zip" -> $valid
archive extract "backup.zip" into "restored" -> $extracted
archive checksum "backup.zip" -> $sha256
```

Each command is one line and requires exactly one variable capture.

## Operations

| Command | Result and constraints |
| --- | --- |
| `create` | Recursively archives regular files from a directory. Output suffix selects `.zip`, `.tar.gz`, or `.tgz`. It never overwrites an existing output. |
| `extract` | Validates the complete archive, then atomically publishes a new destination. The destination must not already exist. |
| `check` | Performs archive/member/size validation without creating a user destination. Temporary validation state is cleaned up. |
| `checksum` | Returns the lowercase 64-character SHA-256 digest of file bytes; it does not inspect or unpack the file. |

Creation normalizes `/` separators and uses deterministic directory order. Empty directories are not members. Suffix matching is case-sensitive; other or missing suffixes fail.

## Safety and limits

Extraction rejects absolute paths, backslashes, traversal escaping the root, duplicate names, file/directory collisions, directories, links, and other non-regular members. Creation rejects symlinks and special files. All archive operations reject malformed archives and never overwrite an existing destination.

Fixed v1 limits:

- 100,000 entries during creation, extraction, or checking.
- 256 MiB per regular file during creation and extraction.
- 1 GiB total uncompressed regular-file data during creation or extraction.
- 1000:1 uncompressed-to-compressed ratio for an extracted member.

Creation streams source files. Extraction validates all members and limits before writing to staging, then renames staging to the destination; a late invalid member leaves no destination behind. These checks do not make archive contents trusted or provide encryption.

## Results and errors

`create`, `extract`, and `check` return a dictionary of string values:

```text
{ operation, format, entries, bytes, archive }
```

`extract` uses `destination` instead of `archive`; `checksum` returns only the digest. Archive failures are ordinary errors and can be handled with [try/catch](errors.md). Branch on success or `error.kind`; exact messages are not a stable API.

Archive v1 does not support updates, appending, encryption/passwords, comments, empty-directory preservation, filesystem metadata, links, broad format detection, overwriting, or partial extraction.

## Related topics

See [Error handling](errors.md) for recovery and [Files](files.md) for path behavior.
