# Xylem File Manager

<!-- Sample of the text only. The hero block, the buttons and the demo keep
     the HTML and the classes of site/pages/en/index.md. -->

**Your whole disk, one tree**

A file manager for the terminal, with the screen and the keys of XTreeGold
and ZTreeWin. It reads directories into memory, so one list can hold files
from many folders, and one command acts on all the files you tagged in it.

[Install](start.md) · [Source on GitHub](https://github.com/andrelnfernandes/xylem)

Free and open source (MIT). Linux x86_64 and ARM64. Windows is experimental.

## What it does

**Lists files across folders.** Branch lists a directory with everything
below it, and Showall lists everything read so far. Tag files there, then
copy, move, delete or rename them together.

**Shows data files as tables.** SQLite, spreadsheets, CSV, JSON Lines, DBF
and Parquet open in the viewer as rows and columns, filtered with a SQL
`WHERE` condition. SQLite files also take full queries with `JOIN`.

**Filters the file list with SQL.** `ext = 'jpg' AND size > 10000000` keeps
the files that meet the condition.

**Works on other machines.** Servers over SSH and containers through
`docker exec` appear as a tree, with copy, move, rename and delete. Google
Drive, S3 and other storage come through rclone.

**Finds what takes space.** Usage sorts the folders and files of a branch by
size. Duplicates finds files and folders with the same contents; on rclone
storage it compares the hashes the service keeps and downloads nothing.

**Has a terminal inside.** The tree follows the shell's `cd`, and Alt+O types
the tagged files into the command line.

**Reads files as hostile.** Viewing a file runs nothing from it, and every
reader has a size limit. The [security statement](security.md) lists them.

Twelve themes, keys set in a configuration file, and help in English,
Portuguese, Spanish, French, German and Chinese.
