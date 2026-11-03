# Features

<p class="xylem-lead">All of it, in detail</p>
<p class="section-lead">The help (<kbd>F1</kbd>) explains each one, in six languages.</p>
<div class="features">
<div class="feature">
<h3>One tree, many files</h3>
<p>A directory tree logged into memory with the file window below it, Branch and Showall
to list the files of a whole branch or of everything logged, and tags that group
operations act on.</p>
</div>
<div class="feature">
<h3>File operations</h3>
<p>Copy and move with or without the paths, with progress and conflict details. Delete to
the trash with Oops to restore, Prune, rename with masks, permissions and dates,
Compare between two windows.</p>
</div>
<div class="feature">
<h3>Finding things</h3>
<p>Filespec by name, date and size, Treespec to type a path, bookmarks, and
<kbd>Ctrl</kbd>+<kbd>S</kbd> to search the tagged files for text.</p>
</div>
<div class="feature">
<h3>Looking inside and editing</h3>
<p>A text and hex viewer with search, soft wrap, line numbers and syntax colors, a table
mode with a SQL filter for spreadsheets, CSV, JSON Lines, DBF, SQLite and Parquet, Markdown
files shown rendered, plus Autoview next to the list. <kbd>E</kbd> opens your text editor or a hex editor
(<code>hexedit</code>, <code>heh</code>, <code>bvi</code>, <code>dhex</code>) right
inside Xylem's frame and color theme — or disables binary editing in <kbd>F12</kbd>.</p>
</div>
<div class="feature">
<h3>Batch over records</h3>
<p><kbd>Ctrl</kbd>+<kbd>B</kbd> turns tagged files, the records of a table or the lines of a
text into a script, JSON, CSV, XML, HTML, SQL or a SQLite database, with escapes that
keep hostile data in its place.</p>
</div>
<div class="feature">
<h3>Duplicates</h3>
<p><kbd>Alt</kbd>+<kbd>Y</kbd> finds files and whole folders with the same contents, without
downloading anything from a Google Drive mounted by rclone, and always leaves one copy.</p>
</div>
<div class="feature">
<h3>Archives</h3>
<p>ZIP, 7z and TAR archives open as directories: browse them, view and copy out of them.
ZIP and TAR can be created too.</p>
</div>
<div class="feature">
<h3>Built-in terminal</h3>
<p><kbd>Alt</kbd>+<kbd>X</kbd> opens a shell whose <code>cd</code> takes the tree along;
<kbd>Alt</kbd>+<kbd>O</kbd> types the tagged files at the prompt.</p>
</div>
<div class="feature">
<h3>Remote and cloud</h3>
<p>Browse and copy from machines over SSH, with their own terminal. Google Drive, Amazon
S3 and some 70 other services through <a href="https://rclone.org">rclone</a>.</p>
</div>
<div class="feature">
<h3>Git and more</h3>
<p>An optional Git menu with status letters in the file list, the clipboard, a
configuration screen (<kbd>F12</kbd>) and twelve themes.</p>
</div>
</div>


## Themes

<p class="xylem-lead">Make it yours</p>
<p class="section-lead">Pick a theme with <kbd>F4</kbd>, or set <code>theme</code> in the
configuration file. Every color can also be changed one by one.</p>
<div class="themes">
<a href="ASSETS/screenshots/theme-nord.png" aria-current="true">nord</a>
<a href="ASSETS/screenshots/theme-slate.png">slate</a>
<a href="ASSETS/screenshots/theme-graphite.png">graphite</a>
<a href="ASSETS/screenshots/theme-daylight.png">daylight</a>
<a href="ASSETS/screenshots/theme-vivid.png">vivid</a>
<a href="ASSETS/screenshots/theme-dark.png">dark</a>
<a href="ASSETS/screenshots/theme-light.png">light</a>
<a href="ASSETS/screenshots/theme-gruvbox.png">gruvbox</a>
<a href="ASSETS/screenshots/theme-solarized.png">solarized</a>
<a href="ASSETS/screenshots/theme-mono.png">mono</a>
<a href="ASSETS/screenshots/theme-norton.png">norton</a>
<a href="ASSETS/screenshots/tree.png">classic</a>
</div>
<div class="window theme-view"><img src="ASSETS/screenshots/theme-nord.png" width="1104" height="772" loading="lazy" alt="The same screen in the nord theme" data-alt="The same screen in the {} theme"></div>


## Disk usage

<p class="xylem-lead">Where did the space go?</p>
<p class="section-lead"><kbd>Alt</kbd>+<kbd>W</kbd> lists the folders and files of a branch by the space they
take, largest first, with each one's share and, in the panel beside it, its date. Go down
folder by folder to what fills the disk: an old backup, a forgotten ISO, a cache that grew.
One disk at a time, hard links counted once, and the space on the disk as
<code>du</code> counts it.</p>
<div class="window theme-view"><img src="ASSETS/screenshots/usage-vivid.png" width="1104" height="772" loading="lazy" alt="The Usage view of a home folder: an old 72 MiB backup file at the top with 37%, then the projects, videos and downloads folders"></div>
<p class="section-lead after">In the tree, <kbd>Alt</kbd>+<kbd>K</kbd> → <strong>Percent</strong> shows the same shares
next to each folder. Pointed at a Google Drive or an S3 bucket mounted with
<a href="https://rclone.org">rclone</a>, it shows what fills them, too.</p>

