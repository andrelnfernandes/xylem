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
# Usage: run.sh [shot...]   (default: all; e.g. run.sh tree terminal demo)
# The PNGs go to docs/screenshots/, the animation (demo) to docs/demo.gif.
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
  case $name in demo) file=docs/demo.gif;; demo-*) file=docs/screenshots/$name.gif;; *) file=docs/screenshots/$name.png;; esac
  HOME=/home/user SHELL=/bin/bash XDG_CONFIG_HOME=$W/root/config/$cfg \
  XDG_STATE_HOME=/mnt/tmp/state XDG_CACHE_HOME=/mnt/tmp/cache \
  XY_SSH=/mnt/tmp/ssh FAKE_SSH_HOME=/home/deploy FAKE_SSH_ASK=$ask SHOT_CWD=/home/user/projects SHOT_KITTY=$kitty \
  "$PY" -I "$T/shot.py" "$X/$file" "$X/target/release/xy" "${root:-/home/user/projects}" "$@"
  echo "$file"; }
run tree classic "*" sleep:1 "$(down 12)"
for theme in dark light mono norton solarized gruvbox nord slate graphite daylight vivid; do
  run theme-$theme $theme "*" sleep:1 "$(down 12)"
done
run branch-tagged classic "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "$D$D"
run git-menu git "*" sleep:1 "$(down 12)" "\r" "$(down 7)" "\007"
run split classic "*" sleep:1 "$(down 12)" "\033[19~" "$(up 8)"
run terminal classic "*" sleep:1 "$(down 12)" "\033x" sleep:1.5 "cd model && ls\r" sleep:1
# The animation at the top of the README: the built-in terminal with the
# tree following a cd; then Branch, tags, and Alt+O typing the tagged files.
run demo classic hold:1.5 "*" hold:1 "\033x" hold:1 "type:cd xylem/src" hold:.4 "\r" hold:2.5 \
  "\033x" hold:1 "$(up 9)" hold:.8 b hold:1.2 "$D" hold:.4 t hold:.5 "$D" hold:.3 "$D" hold:.4 \
  t hold:.5 t hold:1.5 "send:\033x" "\014" hold:.6 "type:wc -c " hold:.4 "\033o" hold:1 \
  "\r" hold:2.5 "\033x" hold:3
run terminal-tagged classic "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "\033x" sleep:1.5 "\014" "wc -c " "\033o" "\r" sleep:1
# The website shows the same screens in modern themes.
run tree-nord nord "*" sleep:1 "$(down 12)"
run branch-tagged-dark dark "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "$D$D"
run split-gruvbox gruvbox "*" sleep:1 "$(down 12)" "\033[19~" "$(up 8)"
run viewer-nord nord "*" sleep:1 "$(down 12)" "\r" "$(down 10)" v sleep:1
run git-menu-dark git-dark "*" sleep:1 "$(down 12)" "\r" "$(down 7)" "\007"
run terminal-tagged-nord nord "*" sleep:1 "$(down 3)" b "$D" t "$D$D" t t "\033x" sleep:1.5 "\014" "wc -c " "\033o" "\r" sleep:1
# The Usage view (Alt+U) of the whole home directory.
root=/home/user
run usage-nord nord sleep:1 "\033u" sleep:2
run usage-vivid vivid sleep:1 "\033u" sleep:2
# Alt+K Percent in the tree, the whole home logged.
run percent-vivid vivid "*" sleep:1.5 "\033k" sleep:.3 p sleep:.5
root=
# Ctrl and Alt held down, as in kitty: the menu changes while the key is held,
# in the tree and in the file window; then Ctrl+G and the Git menu.
kitty=1
run demo-menus-nord git-nord hold:1 "*" hold:1 "$(down 12)" hold:1 \
  press:ctrl hold:2.5 release:ctrl hold:.8 press:alt hold:2.5 release:alt hold:1 \
  "\r" hold:1 press:ctrl hold:2.5 release:ctrl hold:.8 press:alt hold:2.5 release:alt hold:1 \
  "\007" hold:3 "\033" hold:1.5
kitty=
run demo-nord nord hold:1.5 "*" hold:1 "\033x" hold:1 "type:cd xylem/src" hold:.4 "\r" hold:2.5 \
  "\033x" hold:1 "$(up 9)" hold:.8 b hold:1.2 "$D" hold:.4 t hold:.5 "$D" hold:.3 "$D" hold:.4 \
  t hold:.5 t hold:1.5 "send:\033x" "\014" hold:.6 "type:wc -c " hold:.4 "\033o" hold:1 \
  "\r" hold:2.5 "\033x" hold:3
# The short videos on the website, with the demo files.
root=/home/user/showcase
# A database: the list of tables, a filter, a query with a join, and what it refuses.
run demo-sqlite-nord nord hold:.8 "\r" hold:.5 "$D$D" hold:.6 v hold:2.5 "\t" hold:2 "type:ord" hold:.8 "\r" hold:2 \
  "&" hold:.5 "send:\025" "type:price > 100" hold:.5 "\r" hold:2.5 \
  "&" hold:.5 "send:\025" "type:SELECT c.name, count(*) AS orders, sum(o.price) AS total FROM customers c JOIN orders o ON o.customer = c.id GROUP BY c.name ORDER BY total DESC" hold:.8 "\r" hold:4.5 \
  "&" hold:.3 "send:\025" "type:DELETE FROM orders" hold:.4 "\r" hold:4
# A tour of the formats, all with the same data, and a log filtered by a regular expression.
run demo-formats-gruvbox gruvbox hold:.8 "\r" hold:.5 "$(down 8)" hold:.6 v hold:2.5 "]" hold:2.5 "\033" hold:.4 \
  "$(up 2)" hold:.3 v hold:2.5 "\033" hold:.4 "$(up 2)" hold:.3 v hold:3 "\033" hold:.4 "$(up 1)" hold:.3 v hold:2.5 "\033" hold:.4 \
  "$(up 3)" hold:.4 v hold:2 "&" hold:.4 "\033OS" hold:.6 "type:\" [45][0-9][0-9] " hold:.6 "\r" hold:4
# Markdown shown rendered, and its source.
run demo-markdown-nord nord hold:.8 "\r" hold:.5 "$(down 10)" hold:.6 v hold:3.5 "\033[6~" hold:3 "t" hold:3 "t" hold:2.5
# Duplicates: the copies found, and choosing which go, without deleting anything.
run demo-duplicates-vivid vivid hold:.8 "*" hold:1 "\033d" sleep:1 hold:3.5 "a" hold:3.5 "a" hold:3 "g" hold:4
# Last, since it writes files into the list: a table becomes a page, then a script with
# its values escaped, then a database.
run demo-batch-nord nord hold:1 "\r" hold:.6 "$D" hold:.8 v hold:2.5 \
  "\002" hold:.6 "send:\025" "type:report.html" hold:.4 "\r" hold:2.5 "type::report" hold:.8 "\r" hold:4 "\r" hold:2 \
  "\002" hold:.6 "send:\025" "type:update.sql" hold:.4 "\r" hold:1 "type::update" hold:.8 "\r" hold:5 "\r" hold:2 \
  "\002" hold:.6 "send:\025" "type:out.db" hold:.4 "\r" hold:1 "\r" hold:3 "\r" hold:2 \
  "\033" hold:1 "$(down 9)" hold:1 v hold:4
root=
# A remote machine: xy deploy@webserver:
root=deploy@webserver:
# Three backups copied here (Ctrl+C), into the folder picked on the right.
copy() { run "$1" "$2" sleep:2 "*" sleep:1 "\033[19~" sleep:1 "\t" "$D" "\t" "$D" "\r" t t t "\003"; }
# The terminal there (Alt+X), the tree following its cd.
terminal() { run "$1" "$2" sleep:2 "*" sleep:1 "\033x" sleep:1.5 "cd logs && du -h *\r" sleep:1; }
copy remote classic
terminal remote-terminal classic
# ssh asking for the password, in the footer.
ask=1; run remote-password classic sleep:2 "send:s3cr3t"; ask=
copy remote-nord nord
terminal remote-terminal-dark dark
# The password shot leaves ssh and xy --askpass waiting for an answer.
pkill -f "$T/fake_ssh.py|$X/target/release/xy --askpass" || true
' sh "$HOME" "$TOOLS" "$REPO" "$WORK" "$PYTHON" "$@"
