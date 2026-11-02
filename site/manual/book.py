"""Prepares the mdBook source of one language of the manual.

    python3 -I site/manual/book.py LANG DEST

Copies the topics of docs/manual/ (or docs/manual/LANG/, with the English topic
where a translation is missing, as the help does) into DEST/src, writes
DEST/src/SUMMARY.md from the groups of index.md and DEST/book.toml. The topics
are left as they are in the repository, since the help (F1) and the man page
read them too; only the paths of the images change here.
"""

import re
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
MANUAL = REPO / "docs" / "manual"
SCREENSHOTS = REPO / "docs" / "screenshots"
LANGUAGES = ["en", "pt", "es", "fr", "de", "zh"]
# Not a topic of the manual.
SKIP = {"TRANSLATING.md"}
LINK = re.compile(r"\[[^\]]*\]\(([a-z0-9-]+\.md)(?:#[^)]*)?\)")
IMAGE = re.compile(r"\]\((?:\.\./)+screenshots/([A-Za-z0-9._-]+)\)")


def topics(lang):
    """Each topic file name and its text, in the language or in English."""
    english = {p.name: p for p in MANUAL.glob("*.md") if p.name not in SKIP}
    own = {}
    if lang != "en":
        own = {p.name: p for p in (MANUAL / lang).glob("*.md")}
    return {name: (own.get(name) or path).read_text(encoding="utf-8") for name, path in english.items()}


def title(text, name):
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return name[:-3]


def summary(texts):
    """The side menu: the index, then each `##` group of index.md as a part,
    with the topics it links to in order; topics it does not link to (the keys)
    go at the end of the last part."""
    lines = ["# Summary", "", f"[{title(texts['index.md'], 'index.md')}](index.md)", ""]
    seen = {"index.md"}
    for line in texts["index.md"].splitlines():
        if line.startswith("## "):
            lines += ["", "# " + line[3:].strip(), ""]
        elif line.startswith("- "):
            for name in LINK.findall(line):
                if name in texts and name not in seen:
                    seen.add(name)
                    lines.append(f"- [{title(texts[name], name)}]({name})")
    for name in sorted(set(texts) - seen):
        lines.append(f"- [{title(texts[name], name)}]({name})")
    return "\n".join(lines) + "\n"


def main():
    lang, dest = sys.argv[1], Path(sys.argv[2])
    if lang not in LANGUAGES:
        sys.exit(f"unknown language: {lang}")
    src = dest / "src"
    if dest.exists():
        shutil.rmtree(dest)
    src.mkdir(parents=True)
    texts = topics(lang)
    images = set()
    for name, text in texts.items():
        images.update(IMAGE.findall(text))
        text = IMAGE.sub(lambda m: f"](screenshots/{m.group(1)})", text)
        (src / name).write_text(text, encoding="utf-8")
    if images:
        (src / "screenshots").mkdir()
        for image in sorted(images):
            shutil.copy(SCREENSHOTS / image, src / "screenshots" / image)
    (src / "SUMMARY.md").write_text(summary(texts), encoding="utf-8")
    theme = Path(__file__).resolve().parent / "theme"
    for name in ("xylem.css", "languages.js"):
        shutil.copy(theme / name, dest / name)
    (dest / "book.toml").write_text(
        f"""[book]
title = "Xylem File Manager"
language = "{lang}"
src = "src"

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/andrelnfernandes/xylem"
additional-css = ["xylem.css"]
additional-js = ["languages.js"]
no-section-label = true

[output.html.search]
limit-results = 20
""",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
