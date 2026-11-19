#!/bin/sh
# Regenerates the README screenshots (Linux): runs Xylem in an emulated
# 120x34 terminal (pyte) on a sample tree and saves each screen as a PNG,
# with Windows Terminal's "Campbell" colors and DejaVu Sans Mono.
#
# The sample tree is mounted at /home, so that paths read
# /home/user/projects, only inside a private namespace (unshare): nothing
# is created outside your home directory. The remote shots reach
# deploy@webserver (/home/deploy in the sample) through fake_ssh.py, which
# stands in for ssh: no SSH server is needed.
#
# Needs: unshare (util-linux), git, DejaVu Sans Mono and a Python with pyte
# and Pillow, e.g.
#   python3 -m venv ~/.cache/xylem-screenshots/venv
#   ~/.cache/xylem-screenshots/venv/bin/pip install pyte pillow
#   PYTHON=~/.cache/xylem-screenshots/venv/bin/python docs/screenshots/tools/run.sh
#
# Usage: run.sh [shot...]   (default: all; e.g. run.sh tree terminal demo-intro)
# The PNGs and the animations (demo-*, howto-*) go to docs/screenshots/.
set -e
TOOLS=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$TOOLS/../../.." && pwd)
WORK="${XDG_CACHE_HOME:-$HOME/.cache}/xylem-screenshots"
PYTHON=$(command -v "${PYTHON:-python3}")
mkdir -p "$WORK"
(cd "$REPO" && cargo build --release -q)
"$TOOLS/fixture.sh" "$WORK/root" "$REPO/src"
exec unshare -rm sh -c '
set -e
real="$1"; TOOLS="$2"; REPO="$3"; WORK="$4"; PYTHON="$5"; shift 5
# Paths under the home directory, seen from inside (where /home is the
# sample tree): the home directory is also mounted at /mnt/real.
inside() { case "$1" in "$real"/*) echo "/mnt/real${1#$real}";; *) echo "$1";; esac; }
mount -t tmpfs none /mnt
mkdir /mnt/real /mnt/tmp
mount --rbind "$real" /mnt/real
mount --bind "$WORK/root/home" /home
T=$(inside "$TOOLS"); X=$(inside "$REPO"); W=$(inside "$WORK"); PY=$(inside "$PYTHON")
# The remote shots reach deploy@webserver through fake_ssh.py, not ssh.
printf "#!/bin/sh\nexec \"%s\" -I \"%s/fake_ssh.py\" \"\$@\"\n" "$PY" "$T" > /mnt/tmp/ssh
chmod +x /mnt/tmp/ssh
D="\033[B"; U="\033[A"
down() { n=$1; s=""; while [ $n -gt 0 ]; do s="$s$D"; n=$((n-1)); done; printf %s "$s"; }
up() { n=$1; s=""; while [ $n -gt 0 ]; do s="$s$U"; n=$((n-1)); done; printf %s "$s"; }
WANT="$*"
run() { name=$1; cfg=$2; shift 2
  [ -z "$WANT" ] || case " $WANT " in *" $name "*) ;; *) return 0;; esac
  case $name in demo-*|howto-*) file=docs/screenshots/$name.gif;; *) file=docs/screenshots/$name.png;; esac
  HOME=/home/user SHELL=/bin/bash XDG_CONFIG_HOME=$W/root/config/$cfg \
  XDG_STATE_HOME=/mnt/tmp/state XDG_CACHE_HOME=/mnt/tmp/cache \
  SHOT_DUMP=$SHOT_DUMP XY_SSH=/mnt/tmp/ssh FAKE_SSH_HOME=/home/deploy FAKE_SSH_ASK=$ask SHOT_CWD=/home/user/projects SHOT_KITTY=$kitty \
  "$PY" -I "$T/shot.py" "$X/$file" "$X/target/release/xy" "${root:-/home/user/projects}" "$@"
  echo "$file"; }
# The theme gallery of the website: the same screen in each theme.
run theme-classic classic "*" sleep:1 "$(down 12)"
for theme in dark light mono norton solarized gruvbox nord slate graphite daylight vivid; do
  run theme-$theme $theme "*" sleep:1 "$(down 12)"
done
# Every other shot is in the dark theme, so the manual, the README and
# the website look the same.
run tree dark "*" sleep:1 "$(down 12)"
run branch-tagged dark "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "$D$D"
run git-menu git-dark "*" sleep:1 "$(down 12)" "\r" "$(down 7)" "\007"
run split dark "*" sleep:1 "$(down 12)" "\033[19~" "$(up 8)"
run terminal dark "*" sleep:1 "$(down 12)" "\033x" sleep:1.5 "cd model && ls\r" sleep:1
run terminal-tagged dark "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "\033x" sleep:1.5 "\014" "wc -c " "\033o" "\r" sleep:1
run viewer dark "*" sleep:1 "$(down 12)" "\r" "$(down 10)" v sleep:1
# The Usage view (Alt+U) of the whole home directory.
root=/home/user
run usage dark sleep:1 "\033u" sleep:2
root=
# Ctrl and Alt held down, as in kitty: the menu changes while the key is held,
# in the tree and in the file window; then Ctrl+G and the Git menu.
kitty=1
run demo-menus git-dark hold:1 "*" hold:1 "$(down 12)" hold:1 \
  press:ctrl hold:2.5 release:ctrl hold:.8 press:alt hold:2.5 release:alt hold:1 \
  "\r" hold:1 press:ctrl hold:2.5 release:ctrl hold:.8 press:alt hold:2.5 release:alt hold:1 \
  "\007" hold:3 "\033" hold:1.5
kitty=
# The animation at the top of the README and of the website: the built-in
# terminal with the tree following a cd; then Branch, tags, and Alt+O typing
# the tagged files.
run demo-intro dark hold:1.5 "*" hold:1 "\033x" hold:1 "type:cd xylem/src" hold:.4 "\r" hold:2.5 \
  "\033x" hold:1 "$(up 9)" hold:.8 b hold:1.2 "$D" hold:.4 t hold:.5 "$D" hold:.3 "$D" hold:.4 \
  t hold:.5 t hold:1.5 "send:\033x" "\014" hold:.6 "type:wc -c " hold:.4 "\033o" hold:1 \
  "\r" hold:2.5 "\033x" hold:3
# The short videos on the website, with the demo files.
root=/home/user/showcase
# A database: the list of tables, a filter, a query with a join, and what it refuses.
run demo-sqlite dark hold:.8 "\r" hold:.5 "$D$D" hold:.6 v hold:2.5 "\t" hold:2 "type:ord" hold:.8 "\r" hold:2 \
  "&" hold:.5 "send:\025" "type:price > 100" hold:.5 "\r" hold:2.5 \
  "&" hold:.5 "send:\025" "type:SELECT c.name, count(*) AS orders, sum(o.price) AS total FROM customers c JOIN orders o ON o.customer = c.id GROUP BY c.name ORDER BY total DESC" hold:.8 "\r" hold:4.5 \
  "&" hold:.3 "send:\025" "type:DELETE FROM orders" hold:.4 "\r" hold:4
# A tour of the formats, all with the same data, and a log filtered by a regular expression.
run demo-formats dark hold:.8 "\r" hold:.5 "$(down 8)" hold:.6 v hold:2.5 "]" hold:2.5 "\033" hold:.4 \
  "$(up 2)" hold:.3 v hold:2.5 "\033" hold:.4 "$(up 2)" hold:.3 v hold:3 "\033" hold:.4 "$(up 1)" hold:.3 v hold:2.5 "\033" hold:.4 \
  "$(up 3)" hold:.4 v hold:2 "&" hold:.4 "\033OS" hold:.6 "type:\" [45][0-9][0-9] " hold:.6 "\r" hold:4
# Markdown shown rendered, and its source.
run demo-markdown dark hold:.8 "\r" hold:.5 "$(down 10)" hold:.6 v hold:3.5 "\033[6~" hold:3 "t" hold:3 "t" hold:2.5
# Duplicates: the copies found, and choosing which go, without deleting anything.
run demo-duplicates dark hold:.8 "*" hold:1 "\033d" sleep:1 hold:3.5 "a" hold:3.5 "a" hold:3 "g" hold:4
# Last, since it writes files into the list: a table becomes a page, then a script with
# its values escaped, then a database.
run demo-batch dark hold:1 "\r" hold:.6 "$D" hold:.8 v hold:2.5 \
  "\002" hold:.6 "send:\025" "type:report.html" hold:.4 "\r" hold:2.5 "type::report" hold:.8 "\r" hold:4 "\r" hold:2 \
  "\002" hold:.6 "send:\025" "type:update.sql" hold:.4 "\r" hold:1 "type::update" hold:.8 "\r" hold:5 "\r" hold:2 \
  "\002" hold:.6 "send:\025" "type:out.db" hold:.4 "\r" hold:1 "\r" hold:3 "\r" hold:2 \
  "\033" hold:1 "$(down 9)" hold:1 v hold:4
root=
# How-to: copy the photos of a whole branch, keeping their folders.
run howto-copy-paths dark hold:1.2 "step:cursor on projects" hold:1.5 \
  "step:B" b hold:2 \
  "step:F  *.jpg  Enter" f hold:.6 "type:*.jpg" hold:.5 "\r" hold:2 \
  "step:Ctrl+T" "\024" hold:2 \
  "step:Alt+C" "\033c" hold:1.5 \
  "step:/mnt/backup  F7" "type:/mnt/backup" hold:.6 "\033[18~" hold:1.2 "\033[18~" hold:1.2 "\033[18~" hold:2 \
  "step:Enter" "\r" hold:2.5 y "expect:Copied 21 of 21 files" hold:4
# How-to: rename the photos of a folder with a counter.
run howto-rename-mask dark hold:1 "*" hold:.8 "step:cursor on trip  Enter" "$(down 6)" hold:.8 "\r" hold:1.5 \
  "step:Ctrl+T" "\024" hold:1.5 \
  "step:Ctrl+R" "\022" hold:1.2 \
  "step:Ctrl+U  trip_<001>.*  Enter" "\025" hold:.8 "type:trip_<001>.*" hold:.8 "\r" hold:1 \
  "expect: trip_001.jpg" "expect:Renamed 9 of 9 files" hold:4
# How-to: keep the files that meet a SQL condition, and tag them.
run howto-sql-filter dark hold:1 "step:B" b hold:1.5 \
  "step:W" w hold:1 \
  "step:ext = 'jpg' AND size > 1000000  Enter" "type:ext = '\''jpg'\'' AND size > 1000000" hold:.8 "\r" hold:2 "expect:12 files meet the condition" \
  "step:Ctrl+T" "\024" hold:1 "expect:■ IMG_20240201.jpg" hold:4
# How-to: use the tagged files in a command of the built-in terminal.
run howto-terminal-tagged dark hold:1 "step:cursor on notes  →  Enter" "$D$D" hold:.6 "\033[C" hold:.6 "\r" hold:1.2 \
  "step:Ctrl+T" "\024" hold:1.5 \
  "step:Alt+X" "\033x" sleep:1.5 hold:1.5 \
  "step:wc -c  Alt+O" "type:wc -c " hold:.6 "\033o" hold:2 \
  "step:Enter" "\r" sleep:1 "expect:total" hold:4
# How-to: copy files from a machine over SSH (fake_ssh.py stands in for ssh).
run howto-ssh-copy dark hold:1 "step:F8" "\033[19~" hold:1.2 \
  "step:L  deploy@webserver:  Enter" l hold:.6 "type:deploy@webserver:" hold:.6 "\r" sleep:3 "expect:Connected to deploy@webserver" hold:2 \
  "step:cursor on backups  →  Enter" "$D" hold:.6 "\033[C" sleep:1 hold:.8 "\r" hold:1.5 \
  "step:T  T  T" t hold:.4 t hold:.4 t hold:1.2 \
  "step:Ctrl+C" "\003" hold:2.5 \
  "step:Enter" "\r" sleep:2 "expect:Copied 3 of 3 files" hold:4
# How-to: find duplicates and keep one copy of each.
root=/home/user/showcase
# How-to: filter a CSV file with a SQL condition in the viewer.
run howto-table-sql dark hold:1 "step:Enter  cursor on customers.csv  V" "\r" hold:.6 "$D" hold:.6 v sleep:1 hold:2.5 \
  "step:&" "&" hold:1.2 \
  "step:plan = '\''team'\'' OR balance > 1000  Enter" "type:plan = '\''team'\'' OR balance > 1000" hold:.8 "\r" sleep:1 "expect:Zoë Müller" hold:4
run howto-duplicates dark hold:1 "step:*" "*" hold:1 \
  "step:Alt+D" "\033d" sleep:1 hold:2.5 \
  "step:A" a hold:2.5 \
  "step:D  T  Y" d hold:2 t hold:2 y hold:1 "expect:Trashed 4 of 4 files" hold:3
# How-to: find what fills the disk.
root=/home/user
run howto-disk-space dark hold:1 "step:Alt+U" "\033u" sleep:2 hold:2.5 \
  "step:↓  Enter" "$D" hold:.8 "\r" hold:2 \
  "step:Enter" "\r" hold:2.5 \
  "step:↓  Del" "$D" hold:.8 "\033[3~" "expect:DELETE projects-2026-09.tar.gz?" hold:2.5 \
  "step:T  Y" t "expect:to trash?" hold:2 y hold:1 "expect:Trashed 1 of 1 files" hold:4
root=
# A remote machine: xy deploy@webserver:
root=deploy@webserver:
# Three backups copied here (Ctrl+C), into the folder picked on the right.
copy() { run "$1" "$2" sleep:2 "*" sleep:1 "\033[19~" sleep:1 "\t" "$D" "\t" "$D" "\r" t t t "\003"; }
# The terminal there (Alt+X), the tree following its cd.
terminal() { run "$1" "$2" sleep:2 "*" sleep:1 "\033x" sleep:1.5 "cd logs && du -h *\r" sleep:1; }
copy remote dark
terminal remote-terminal dark
# ssh asking for the password, in the footer.
ask=1; run remote-password dark sleep:2 "send:s3cr3t"; ask=
# The password shot leaves ssh and xy --askpass waiting for an answer.
pkill -f "$T/fake_ssh.py|$X/target/release/xy --askpass" || true
' sh "$HOME" "$TOOLS" "$REPO" "$WORK" "$PYTHON" "$@"
