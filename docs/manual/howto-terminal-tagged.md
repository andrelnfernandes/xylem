# Use the tagged files in a shell command

This runs `wc -c` on the files tagged in Xylem, from the built-in terminal.
The same steps work with any command that takes file names.

## Steps

1. In the tree, put the cursor on the directory, press → to read it, and
   Enter to go to its files.
2. Tag the files: `T` on each one, or Ctrl+T for the whole list.
3. Press Alt+X. A terminal opens in the lower part of the screen, with the
   shell in the directory shown above it.
4. Type the command and a space, `wc -c `, and press Alt+O. Xylem types the
   names of the tagged files at the cursor, each in quotes.
5. Press Enter.

![The five steps: the files of notes, Ctrl+T, Alt+X opening the terminal, wc -c with Alt+O typing the two file names, and the output](../screenshots/howto-terminal-tagged.gif)

Alt+X goes back to Xylem. The shell keeps running, with its history, until
`exit`.

## What Alt+O types

The names are relative to the shell's directory and quoted, so names with
spaces are passed as one argument each:

```
wc -c 'ideas.md' 'todo.md'
```

In a Branch or a Showall the tagged files are in several directories, and
each name has its path from the shell's directory.

## The variable

On entering the terminal Xylem also sets `XY_TAGGED` with the tagged files.

| Shell | How to use it |
|---|---|
| bash | `cp "${XY_TAGGED[@]}" ~/backup` |
| zsh, fish | `cp $XY_TAGGED ~/backup` |
| sh, dash | not set |

The variable has the files tagged when you entered the terminal. Tag more
files and it changes the next time you enter.

## One command with eXecute

`X` (eXecute) asks for one command line and runs it. In it `%t` stands for
the tagged files and `%f` for the file under the cursor:

```
tar czf photos.tar.gz %t
```

[eXecute](execute.md) has the other placeholders.

## When it does not do what you expect

- **The first line in the terminal is ` . '…/sync.sh'`.** Xylem types it to
  move the shell to the tree's directory and set `XY_TAGGED`. The leading
  space keeps it out of the history in bash and zsh.
- **Alt+O types nothing.** No file is tagged.
- **The shell did not change directory.** A program was running in the
  terminal. Xylem types the `cd` only when the shell is at its prompt.
- **F1 does not open the help.** In the terminal every key goes to the
  shell, except Alt+X, Alt+Z and Alt+O.

See also: [the built-in terminal](terminal.md), [eXecute](execute.md),
[tags](tagging.md).
