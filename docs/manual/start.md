# Install and first five minutes

## Install

Xylem runs on Linux (x86_64 and ARM64). The Windows build passes the tests
but has not been used day to day; see [Windows](windows.md).

**Binary.** Download the archive for your system from
`https://github.com/andrelnfernandes/xylem/releases`, extract it and put `xy`
in a directory of your `PATH`:

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

The Linux binaries are static and run on any distribution. Each archive has
a `.sha256` file to check with `sha256sum -c`.

**Package.** The same page has `.deb` and `.rpm` packages, which also install
the menu entry, the icons and the man page:

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

The package does not come from a repository, so it is not updated
automatically. Install the next version the same way.

**Cargo.** Needs Rust and a C compiler (for SQLite and zstd). The crate is
`xylem-fm`; the command it installs is `xy`:

```bash
cargo install xylem-fm
```

## Start

```bash
xy
```

opens the current directory. `xy ~/projects` opens another one, and
`xy user@host:` opens a [remote machine](remote.md). [The command
line](cli.md) has the other forms.

The first time, Xylem asks whether it may look for a new version once a
day: **Y** or **N** (see [New versions](configuration.md#new-versions)).

## Move in the tree

The tree shows the start directory and its subdirectories. Only the start
directory has been read.

| Key | What it does |
|---|---|
| ↑ ↓ | move the cursor |
| → | read the directory under the cursor and show what is in it |
| ← | close it |
| `*` | read the directory and everything below it |

The lower window lists the files of the directory under the cursor. On a
directory that has not been read it shows `Not logged`.

## Go to the files

**Enter** moves to the file window, which then takes the height of the
screen. Enter or Esc goes back to the tree.

| Key | What it does |
|---|---|
| `V` | view the file |
| `E` | open it in your editor |
| `C`, `M`, `D`, `R` | copy, move, delete, rename it |

## Tag files and copy them

1. In the file window, press `T` on each file you want. A `■` appears before
   the name and the cursor moves down.
2. Press Ctrl+C. The prompt asks where to copy.
3. Type a directory and press Enter.

The tagged files are copied. `U` untags a file, and Ctrl+U untags all of
them.

## Quit

`Q` quits, after asking. Alt+Q quits and leaves the shell in the directory
you were in, if the shell function from [Quitting](quit.md) is installed.

## Next

- [The screen](screen.md) names each part of what you see.
- [The menus](menus.md) explains what a letter does alone, with Ctrl and
  with Alt.
- [Vocabulary](glossary.md) defines log, branch, tag and the other terms.
- **F1** opens the help for what is on the screen.
