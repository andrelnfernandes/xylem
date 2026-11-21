# Security

Xylem treats as hostile every file it did not create: files you download,
archives you open, names a remote server sends, repositories you clone. A
file's contents or name should never make Xylem crash, hang, run code, write
outside the destination or use up the memory. If one does, that is a bug to
report.

## What Xylem does

| Risk | Measure |
|---|---|
| A crafted file exploits the reader | Text, hex, encodings, compressed files, archives, Markdown, spreadsheets, CSV, JSON Lines and DBF are read by parsers written in Rust. SQLite and Parquet are read by libraries written in C, and only when you open the file with `V`, never in Autoview. |
| A file runs a program | No external program runs on a file unless you ask with `E`, `O` or `X`. |
| A small file expands to fill memory or disk | Each reader has a limit on how much it reads, how much it decompresses, how long it takes and how much memory the decompressor uses. One level of compression is undone. Showing a file writes nothing to the disk. |
| A name hides what it is | Control characters and bidirectional controls (Trojan Source) are not sent to the terminal. A name with hidden direction characters is marked in red, and `O` asks before opening it. |
| An archive writes outside the destination | Paths with `..` and absolute paths are refused, symbolic links in archives are not extracted, and setuid bits are dropped. The same checks apply to names sent by an SFTP server. |
| A Git repository runs its own commands | Looking at a repository runs no hooks, no `core.fsmonitor`, no external diff and no text conversion, unless its folder is listed in `git_trusted`. |
| A dependency has a known flaw | The dependencies are checked against the RustSec database in CI. |

## The network

Xylem connects to the network on its own for one thing only: if you said yes
at the first start, it asks the website once a day for the number of the
latest version ([New versions](configuration.md#new-versions)). Everything
else that uses the network is something you asked for: a remote machine
(`ssh`), the cloud (`rclone`), a container (`docker`), a network share, and
the programs you run yourself (`X`, the built-in terminal).

## What you can do

- Turn off decompression in the viewer (`viewer_decompress = false`, F12
  page 2) if you browse untrusted folders with Autoview, or keep Autoview
  closed there.
- Leave Git off (`git = false`, the default) in repositories you did not
  create.
- Say no to looking for new versions, or turn it off on page 3 of F12, if
  Xylem should never connect by itself.
- Do not run Xylem as root.
- `O` passes the file to another program. Xylem's limits do not apply to
  that program.

## What is not covered

- The programs that `E`, `O` and `X` start.
- `rclone`, `ssh`, `docker` and `git`, which Xylem runs as they are
  installed.
- A terminal that itself mishandles the text it receives.

## Reporting a problem

Report it in private, through GitHub Security Advisories:
`https://github.com/andrelnfernandes/xylem/security/advisories/new`. Do not
open a public issue for it.

See also: [the viewer](viewer.md), [archives](archives.md), [Git](git.md).
