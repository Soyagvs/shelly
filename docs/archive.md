# Archives

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

Shelly archive v1 creates and extracts deterministic ZIP and TAR.GZ archives, validates archives without keeping extracted files, and computes SHA-256 checksums. Paths are interpreted relative to the current Shelly script file directory, like the commands in [Files](files.md).

## Quick reference

```shelly
archive create "source-dir" from "backup.zip" -> $result
archive create "source-dir" from "backup.tar.gz" -> $result
archive extract "backup.zip" into "restored" -> $result
archive checksum "backup.zip" -> $sha256
archive check "backup.zip" -> $result
```

These are the complete one-line forms. `create` takes a source directory and archive output path; `extract` takes an archive and a new destination directory; `checksum` takes any readable file; and `check` takes an archive. The destination after `->` must be a variable for every form.

## Create

`archive create "source" from "archive.zip" -> $result` or `archive create "source" from "archive.tar.gz" -> $result` recursively archives regular files below `source`. `source` must be a directory. ZIP is selected by `.zip`; TAR.GZ is selected by `.tar.gz` or `.tgz` (see [Suffix inference](#suffix-inference)).

```shelly
archive create "project" from "dist/project.zip" -> $created
shellout $created
```

Files are stored with normalized `/` separators and deterministic directory order. Empty directories are not archive members. Symlinks and special files are rejected. The output is built in staging and then published only when the destination does not already exist; archive creation never overwrites an existing file.

## Extract

`archive extract "archive" into "destination" -> $result` validates the complete archive before creating a staging directory. The destination must not exist, including as a file. On success the staging directory is renamed to the destination, preserving nested paths:

```shelly
archive extract "project.zip" into "restored" -> $extracted
# A member named lib/config.json becomes restored/lib/config.json.
```

Extraction refuses unsafe member names, including absolute paths, backslashes, traversal that escapes the archive root, duplicate names, and file/directory path collisions. Only regular file members are accepted; archive directories, links, and other special members are rejected. It never overwrites an existing destination.

## Check

`archive check "archive" -> $result` performs the same format, member-name, member-type, and size checks as extraction without leaving an extracted destination. It may use an internal staging path while validating, but removes that staging path and does not create or overwrite a user destination or alter the archive. A failed check also cleans up its staging path.

```shelly
archive check "incoming.tar.gz" -> $checked
if $checked
    shellout "archive is valid"
end
```

## Checksum

`archive checksum "path" -> $sha256` returns the lowercase, 64-character SHA-256 digest as a string. It hashes the file bytes exactly; it does not inspect or unpack an archive.

```shelly
archive checksum "backup.zip" -> $digest
shellout $digest
# For the bytes "abc": ba7816bf8f01cfea414140de5dae2223...
```

## Result dictionaries

`create`, `extract`, and `check` return a dictionary with string values:

```text
{
  operation: "create" | "extract" | "check",
  format: "zip" | "tar.gz",
  entries: "number of regular file members",
  bytes: "total uncompressed bytes",
  archive: "archive path"
}
```

For `extract`, the path key is `destination` instead of `archive`. `check` reports `operation: "check"` and the checked path in `archive`. `checksum` is the exception: it returns only the SHA-256 string.

## Suffix inference

The output path selects the format for `create`, and the input path selects the format for `extract` and `check`:

| Suffix | Format |
| --- | --- |
| `.zip` | ZIP |
| `.tar.gz` | TAR.GZ |
| `.tgz` | TAR.GZ |

Suffix matching is case-sensitive. Other or missing suffixes fail; content sniffing is not performed. Examples of invalid paths include `backup.tar`, `backup.gz`, and `backup.ZIP`.

## Limits and safety policy

The fixed v1 limits are:

- At most **100,000 entries**.
- At most **256 MiB per extracted file**.
- At most **1 GiB total uncompressed extracted data**.
- At most a **1000:1 uncompressed-to-compressed ratio** for an extracted member.

Create also rejects a source whose total regular-file bytes exceed 1 GiB. These limits are checked before extraction writes files. A late invalid member therefore leaves no destination behind. Create rejects nonexistent sources, non-directories, symlinks, and special files. Extract and check reject malformed archives, nonexistent or unreadable archive paths, unsafe names, duplicate/colliding names, non-regular members, and limit violations.

## Errors and non-goals

Archive failures are ordinary Shelly runtime errors and can be handled with [`try`](errors.md):

```shelly
try -> $error
    archive extract "download.zip" into "out" -> $result
catch
    shellout $error.message
end
```

The error dictionary has the usual `kind` and `message` fields. Exact messages describe the failed policy or underlying I/O/format error; callers should branch on success or catch the error rather than depend on message wording.

Archive v1 does not support incremental updates, appending, encryption/passwords, archive comments, preserving empty directories or filesystem metadata, symlink/hardlink members, broad format detection, overwriting destinations, or partial extraction. It is a regular-file archive format, validation, and checksum API—not a general-purpose archive editor.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
