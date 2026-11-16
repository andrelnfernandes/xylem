# Copy files from another machine over SSH

This shows `deploy@webserver` beside this machine and copies three backup
files from it into a local directory.

## Steps

1. Press F8. The screen splits in two windows, both on the current
   directory. The left one has the focus.
2. Press `L`, type `deploy@webserver:` and press Enter. The left window now
   shows the home directory of that user on that machine.
3. Put the cursor on `backups`, press → to read it, and Enter to go to its
   files.
4. Press `T` on each file to copy.
5. Press Ctrl+C. The prompt suggests the directory of the other window.
6. Press Enter.

![The six steps: F8, Log to deploy@webserver, the files of backups, three files tagged, Ctrl+C suggesting the local window, and three files copied](../screenshots/howto-ssh-copy.gif)

The footer shows `Copied 3 of 3 files`.

## The address

| Address | Opens |
|---|---|
| `user@host:` | the home directory |
| `user@host:/srv/www` | that directory |
| `ssh://user@host:2222/srv/www` | the same, on another port |
| `docker://container/path` | a running container, through `docker exec` |

Xylem runs your own `ssh`. Host aliases, keys, the agent and `known_hosts`
from `~/.ssh/config` apply, and nothing is installed on the other machine.

## Passwords

If ssh asks for a password or a passphrase, the question shows in the footer
and what you type is hidden. Xylem passes it to ssh and does not store it.
For a host seen for the first time, the footer shows the fingerprint of its
key and waits for `yes`.

## In the other direction

To copy to the remote machine, go to the local window with Tab, tag the
files and press Ctrl+C: the prompt suggests the remote window's directory.
Move, Rename, Delete, Makedir, Attributes and New date also work on the
remote side.

## When it does not do what you expect

- **The file window says `Not logged`.** The directory was not read. Go back
  with Esc and press → on it. Step 3 does this.
- **Ctrl+C asks for a destination with nothing suggested.** The screen is
  not split. Type a local path, or press F8 first.
- **Edit and eXecute are not in the menu.** They do not work on a remote
  machine. Alt+X opens a shell there.
- **There is no trash.** Delete on a remote machine is permanent, and the
  question says so.

To come back later without typing the address, save the directory as a
bookmark with Ctrl+F11.

See also: [remote machines](remote.md), [split screen](split.md),
[bookmarks](bookmarks.md).
