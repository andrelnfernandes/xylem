"""Runs Xylem in a pseudo-terminal, sends keys and saves the screen as a PNG,
or, for OUT.gif, every step as a frame of an animation.

Usage: shot.py OUT.png|OUT.gif XY ROOT [STEP...], where ROOT is a folder or
a remote address (user@host:, then Xylem starts in $SHOT_CWD) and a step is:
  KEYS            keys to send, with escapes (e.g. '\\x1bx' for Alt+X)
  type:TEXT       TEXT typed one character at a time (one frame each)
  send:KEYS       keys sent without a frame
  sleep:SECONDS   wait without a frame
  hold:SECONDS    how long the last frame stays on screen (GIF)
  press:ctrl, press:alt, release:ctrl, release:alt
                  Ctrl or Alt alone, held down and let go, as kitty reports them

SHOT_COLS sets the width (120 by default). With SHOT_KITTY=1 the terminal
answers like kitty that it has the keyboard protocol, so Xylem shows the
Ctrl and Alt menus while those keys are held."""
import fcntl, os, pty, re, select, struct, sys, termios, time
import pyte
from PIL import Image, ImageDraw, ImageFont

COLS, ROWS, CW, CH, PAD = 120, 34, 9, 22, 12
# Another width, to check how the screens fit (SHOT_COLS=80).
COLS = int(os.environ.get("SHOT_COLS", COLS))
# Windows Terminal's "Campbell", as in the first screenshots.
NAMED = {
    "black": "0c0c0c", "red": "c50f1f", "green": "13a10e", "brown": "c19c00",
    "blue": "0037da", "magenta": "881798", "cyan": "3a96dd", "white": "cccccc",
    "brightblack": "767676", "brightred": "e74856", "brightgreen": "16c60c",
    "brightbrown": "f9f1a5", "brightblue": "3b78ff", "brightmagenta": "b4009e",
    "brightcyan": "61d6d6", "brightwhite": "f2f2f2",
}
# The same 16 colors when they come as indexes (ESC[38;5;Nm): pyte turns
# them into xterm's tones.
XTERM = ["000000", "cd0000", "00cd00", "cdcd00", "0000ee", "cd00cd", "00cdcd", "e5e5e5",
         "7f7f7f", "ff0000", "00ff00", "ffff00", "5c5cff", "ff00ff", "00ffff", "ffffff"]
CAMPBELL = ["black", "red", "green", "brown", "blue", "magenta", "cyan", "white"]
CAMPBELL += ["bright" + c for c in CAMPBELL]
for tone, name in zip(XTERM, CAMPBELL):
    NAMED[tone] = NAMED[name]
DEFAULT_FG, DEFAULT_BG = "cccccc", "0c0c0c"
FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf"

out, xy, root, steps = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
KITTY = os.environ.get("SHOT_KITTY") == "1"
KEYBOARD_PROTOCOL = re.compile(rb"\x1b\[[<>=?][0-9;]*u")
# Ctrl and Alt alone as kitty sends them: the left key's code, its modifier
# bit while pressed, and event type 3 on release.
MODIFIERS = {
    "press:ctrl": "\\x1b[57442;5u", "release:ctrl": "\\x1b[57442;1:3u",
    "press:alt": "\\x1b[57443;3u", "release:alt": "\\x1b[57443;1:3u",
}
screen = pyte.Screen(COLS, ROWS)
stream = pyte.ByteStream(screen)
pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.chdir(root if os.path.isdir(root) else os.environ["SHOT_CWD"])
    os.execv(xy, [xy, root])
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))


def pump(seconds):
    end = time.time() + seconds
    while time.time() < end:
        if select.select([fd], [], [], 0.05)[0]:
            try:
                data = os.read(fd, 65536)
            except OSError:
                return
            # pyte does not know the keyboard protocol's push and pop
            # (CSI > flags u, CSI < n u) and would print part of them.
            stream.feed(KEYBOARD_PROTOCOL.sub(b"", data))
            # kitty: the protocol is there, with no flags pushed yet; then the
            # device attributes, which crossterm waits for.
            if KITTY and b"\x1b[?u" in data:
                os.write(fd, b"\x1b[?0u\x1b[?62c")
            if b"\x1b[6n" in data:
                os.write(fd, f"\x1b[{screen.cursor.y + 1};{screen.cursor.x + 1}R".encode())


def color(name, default):
    if name == "default":
        return "#" + default
    return "#" + NAMED.get(name, name)


FONT_REGULAR, FONT_BOLD = ImageFont.truetype(FONT, 15), ImageFont.truetype(BOLD, 15)
# Box-drawing characters as lines, so that they join across cells:
# (up, down, left, right).
BOX = {
    "─": (0, 0, 1, 1), "│": (1, 1, 0, 0), "┌": (0, 1, 0, 1), "┐": (0, 1, 1, 0),
    "└": (1, 0, 0, 1), "┘": (1, 0, 1, 0), "├": (1, 1, 0, 1), "┤": (1, 1, 1, 0),
    "┬": (0, 1, 1, 1), "┴": (1, 0, 1, 1), "┼": (1, 1, 1, 1),
}
# The rounded corners of the modern themes: where the arc's center sits
# (right and down of the cell's center, as 1 or -1) and its angles.
ROUND = {"╭": (1, 1, 180, 270), "╮": (-1, 1, 270, 360), "╰": (1, -1, 90, 180), "╯": (-1, -1, 0, 90)}


def render():
    """The screen as it is now."""
    img = Image.new("RGB", (COLS * CW + 2 * PAD, ROWS * CH + 2 * PAD))
    draw = ImageDraw.Draw(img)
    # The margin takes the color of the screen's corner.
    corner = screen.buffer[0][0]
    draw.rectangle([0, 0, img.width, img.height], fill=color(corner.bg, DEFAULT_BG))
    cursor = None if screen.cursor.hidden else (screen.cursor.x, screen.cursor.y)
    for y in range(ROWS):
        row = screen.buffer[y]
        for x in range(COLS):
            ch = row[x]
            fg, bg = color(ch.fg, DEFAULT_FG), color(ch.bg, DEFAULT_BG)
            if ch.reverse or (x, y) == cursor:
                fg, bg = bg, fg
            px, py = PAD + x * CW, PAD + y * CH
            draw.rectangle([px, py, px + CW - 1, py + CH - 1], fill=bg)
            text = ch.data
            if not text.strip():
                continue
            if text in BOX:
                up, down, left, right = BOX[text]
                cx, cy = px + CW // 2, py + CH // 2
                if left:
                    draw.line([px, cy, cx, cy], fill=fg)
                if right:
                    draw.line([cx, cy, px + CW - 1, cy], fill=fg)
                if up:
                    draw.line([cx, py, cx, cy], fill=fg)
                if down:
                    draw.line([cx, cy, cx, py + CH - 1], fill=fg)
                continue
            if text in ROUND:
                dx, dy, start, end = ROUND[text]
                r = CW // 2
                cx, cy = px + CW // 2, py + CH // 2
                ax, ay = cx + dx * r, cy + dy * r
                draw.arc([ax - r, ay - r, ax + r, ay + r], start, end, fill=fg)
                # From the arc's ends to the cell's edges.
                edge_x = px + CW - 1 if dx > 0 else px
                draw.line([ax, cy, edge_x, cy], fill=fg)
                edge_y = py + CH - 1 if dy > 0 else py
                draw.line([cx, ay, cx, edge_y], fill=fg)
                continue
            draw.text((px, py + 3), text, font=FONT_BOLD if ch.bold else FONT_REGULAR, fill=fg)
    return img


def send(keys):
    os.write(fd, keys.encode().decode("unicode_escape").encode("utf-8"))


animate = out.endswith(".gif")
frames = []  # [image, milliseconds]


def frame(ms):
    if animate:
        frames.append([render(), ms])


pump(1.5)
# The first frame of a loop must not be an empty screen.
for _ in range(50):
    if any(line.strip() for line in screen.display):
        break
    pump(0.1)
pump(0.3)
frame(1000)
for step in steps:
    if step.startswith("sleep:"):
        pump(float(step[6:]))
    elif step.startswith("hold:"):
        if frames:
            frames[-1][1] = int(float(step[5:]) * 1000)
    elif step.startswith("send:"):
        send(step[5:])
        pump(0.8)
    elif step.startswith("type:"):
        for ch in step[5:]:
            send(ch)
            pump(0.08 if animate else 0.02)
            frame(90)
        pump(0.3)
    else:
        send(MODIFIERS.get(step, step))
        pump(0.8)
        frame(700)
pump(0.5)

if animate:
    # Equal frames in a row become one, shown for their total time.
    merged = []
    for img, ms in frames:
        if merged and img.tobytes() == merged[-1][0].tobytes():
            merged[-1][1] += ms
        else:
            merged.append([img, ms])
    images = [img.convert("P", palette=Image.ADAPTIVE, colors=128) for img, _ in merged]
    images[0].save(out, save_all=True, append_images=images[1:], loop=0,
                   duration=[ms for _, ms in merged], optimize=True, disposal=1)
else:
    render().save(out)
os.kill(pid, 9)
