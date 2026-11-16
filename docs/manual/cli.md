# The command line

```
xy [--cwd-file FILE] [WHERE]
```

## What to open

| `WHERE` | Opens |
|---|---|
| nothing | the current directory |
| `DIRECTORY` | that directory, as the root of the tree |
| `user@host:` or `user@host:PATH` | a machine over SSH, at the home directory or at `PATH` |
| `ssh://[user@]host[:port][/PATH]` | the same, with a port |
| `docker://[user@]container[/PATH]` | a running container, through `docker exec` |

The same addresses work in `L` (Log) inside Xylem. See
[remote machines](remote.md).

## Options

| Option | What it does |
|---|---|
| `--cwd-file FILE` | On Alt+Q, writes the current directory to `FILE`. A shell function reads it and changes directory. [Quitting](quit.md) has the function. |
| `--default-config` | Prints a configuration file with every option, its default and a comment. |
| `--man` | Shows the manual page. With the output redirected, prints its source. |
| `-h`, `--help` | Shows the options and the path of the configuration file. |
| `-V`, `--version` | Shows the version. |

## Installing the manual page

The `.deb` and `.rpm` packages install it. From a binary or from Cargo:

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Starting from a commented configuration

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

This overwrites an existing `config.toml`.

## Environment variables

| Variable | Effect |
|---|---|
| `XDG_CONFIG_HOME` | The configuration is read from `$XDG_CONFIG_HOME/xylem/` instead of `~/.config/xylem/`. |
| `SHELL` | The shell of the built-in terminal. |
| `VISUAL`, `EDITOR` | The editor for `E`, when none is set in the configuration. |
| `XY_DOCKER` | The program used for `docker://` addresses. `XY_DOCKER=podman` uses Podman. |

Inside the built-in terminal Xylem sets `XY_TERMINAL=1` and, in bash, zsh
and fish, `XY_TAGGED` with the tagged files. See
[the built-in terminal](terminal.md).

See also: [configuration](configuration.md).
