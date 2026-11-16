# Xylem File Manager

<div class="xylem-hero">
<p class="xylem-hero-title">Your whole disk, <em>one tree</em></p>
<p class="lead">A file manager for the terminal, with the screen and the keys of XTreeGold and
ZTreeWin. It reads directories into memory, so one list can hold files from many folders, and
one command acts on all the files you tagged in it.</p>
<div class="buttons">
<a class="button primary" href="start.md">Install</a>
<a class="button" href="https://github.com/andrelnfernandes/xylem">
<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
Source on GitHub</a>
</div>
<p class="note">Free and open source (MIT) · Linux x86_64 and ARM64 · Windows (experimental)</p>
<div class="window demo">
<div class="bar"><i></i><i></i><i></i><span>xy ~/projects</span></div>
<img src="ASSETS/screenshots/demo-nord.gif" width="1104" height="772"
alt="Xylem in use: the built-in terminal with the tree following a cd, tagging files in a Branch and typing them into a command with Alt+O">
</div>
</div>

## What it does

<div class="features">
<div class="feature">
<h3>Lists files across folders</h3>
<p>Branch lists a directory with everything below it, and Showall lists everything read so far. Tag files there, then copy, move, delete or rename them together. <a href="overview.md">What Xylem is</a></p>
</div>
<div class="feature">
<h3>Shows data files as tables</h3>
<p>SQLite, spreadsheets, CSV, JSON Lines, DBF and Parquet open in the viewer as rows and columns, filtered with a SQL <code>WHERE</code> condition. SQLite files also take whole queries with <code>JOIN</code>. <a href="howto-table-sql.md">How to</a></p>
</div>
<div class="feature">
<h3>Filters the file list with SQL</h3>
<p><code>ext = 'jpg' AND size &gt; 10000000</code> keeps the files that meet the condition. <a href="howto-sql-filter.md">How to</a></p>
</div>
<div class="feature">
<h3>Works on other machines</h3>
<p>Servers over SSH and containers through <code>docker exec</code> appear as a tree, with copy, move, rename and delete. Google Drive, S3 and other storage come through rclone. <a href="howto-ssh-copy.md">How to</a></p>
</div>
<div class="feature">
<h3>Finds what takes space</h3>
<p>Usage sorts the folders and files of a branch by size. Duplicates finds files and folders with the same contents; on rclone storage it compares the hashes the service keeps and downloads nothing. <a href="howto-disk-space.md">How to</a></p>
</div>
<div class="feature">
<h3>Has a terminal inside</h3>
<p>The tree follows the shell's <code>cd</code>, and Alt+O types the tagged files into the command line. <a href="howto-terminal-tagged.md">How to</a></p>
</div>
<div class="feature">
<h3>Reads files as hostile</h3>
<p>Viewing a file runs nothing from it, and every reader has a size limit. <a href="security.md">Security</a></p>
</div>
<div class="feature">
<h3>Themes, keys and languages</h3>
<p>Twelve themes, keys set in a configuration file, and help in English, Portuguese, Spanish, French, German and Chinese. <a href="configuration.md">Configuration</a></p>
</div>
</div>

