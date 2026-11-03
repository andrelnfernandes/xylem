"""Prepares the mdBook source of one language of the website.

    python3 -I site/book.py LANG DEST

The website is one book per language: English at the root, the others under
/LANG/. Each one starts with the pages of the site (site/pages/LANG/, or the
English ones where there is no translation) and goes on with the manual: the
topics of docs/manual/ (or docs/manual/LANG/, with the English topic where a
translation is missing, as the help does), its index as manual.md, and the
side menu from the groups of that index. The topics are left as they are in
the repository, since the help (F1) and the man page read them too; only the
paths of the images change here.

Writes DEST/src, DEST/theme and DEST/book.toml.
"""

import re
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SITE = REPO / "site"
MANUAL = REPO / "docs" / "manual"
SCREENSHOTS = REPO / "docs" / "screenshots"
LANGUAGES = ["en", "pt", "es", "fr", "de", "zh"]
# The pages of the site, in the order of the side menu (index.md first).
PAGES = ["index.md", "in-action.md", "features.md", "screenshots.md", "install.md", "next.md", "about.md"]
# Not a topic of the manual.
SKIP = {"TRANSLATING.md"}
LINK = re.compile(r"\[[^\]]*\]\(([a-z0-9-]+\.md)(?:#[^)]*)?\)")
IMAGE = re.compile(r"\]\((?:\.\./)+screenshots/([A-Za-z0-9._-]+)\)")
URL = "https://andrelnfernandes.github.io/xylem/"
DESCRIPTION = {
    "en": "See your whole disk as one tree, gather files from any folder, and act on all "
    "of them at once, from any terminal, right from the keyboard.",
    "pt": "Veja o disco inteiro como uma só árvore, reúna arquivos de qualquer pasta e aja "
    "sobre todos de uma vez, em qualquer terminal, direto do teclado.",
}
MANUAL_PART = {"en": "Manual", "pt": "Manual", "es": "Manual", "fr": "Manuel", "de": "Handbuch", "zh": "手册"}


def own_or_english(english_dir, own_dir, names):
    """Each file name and its text, in the language or in English."""
    out = {}
    for name in names:
        path = own_dir / name if own_dir and (own_dir / name).exists() else english_dir / name
        out[name] = path.read_text(encoding="utf-8")
    return out


def title(text, name):
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return name[:-3]


def summary(lang, pages, topics):
    """The side menu: the site's pages, then the manual's index and each `##`
    group of it as a part, with the topics it links to in order; topics it
    does not link to (the keys) go at the end of the last part."""
    lines = ["# Summary", "", f"[{title(pages['index.md'], 'index.md')}](index.md)", ""]
    lines += [f"- [{title(pages[n], n)}]({n})" for n in PAGES[1:]]
    index = topics["manual.md"]
    lines += ["", "# " + MANUAL_PART[lang], "", f"- [{title(index, 'manual.md')}](manual.md)"]
    seen = {"manual.md"}
    for line in index.splitlines():
        if line.startswith("## "):
            lines += ["", "# " + line[3:].strip(), ""]
        elif line.startswith("- "):
            for name in LINK.findall(line):
                if name in topics and name not in seen:
                    seen.add(name)
                    lines.append(f"- [{title(topics[name], name)}]({name})")
    for name in sorted(set(topics) - seen):
        lines.append(f"- [{title(topics[name], name)}]({name})")
    return "\n".join(lines) + "\n"


def redirects():
    """The addresses of before, to the same page now: the home page in
    Portuguese (/pt-br/) and the manual's own books (/manual/, /manual/LANG/)."""
    names = [p.name[:-3] for p in MANUAL.glob("*.md") if p.name not in SKIP]
    out = {"/pt-br/index.html": "../pt/index.html"}
    for name in names:
        target = "manual" if name == "index" else name
        out[f"/manual/{name}.html"] = f"../{target}.html"
        for lang in LANGUAGES[1:]:
            out[f"/manual/{lang}/{name}.html"] = f"../../{lang}/{target}.html"
    return out


def main():
    lang, dest = sys.argv[1], Path(sys.argv[2])
    if lang not in LANGUAGES:
        sys.exit(f"unknown language: {lang}")
    if dest.exists():
        shutil.rmtree(dest)
    src = dest / "src"
    src.mkdir(parents=True)

    names = sorted(p.name for p in MANUAL.glob("*.md") if p.name not in SKIP)
    topics = own_or_english(MANUAL, MANUAL / lang if lang != "en" else None, names)
    topics["manual.md"] = topics.pop("index.md")
    pages = own_or_english(SITE / "pages" / "en", SITE / "pages" / lang, PAGES)
    if set(pages) & set(topics):
        sys.exit(f"a page of the site has the name of a topic: {set(pages) & set(topics)}")

    # The images of the site stay once, in /assets/, next to the English book.
    assets = "" if lang == "en" else "../"
    images = set()
    for name, text in {**topics, **pages}.items():
        images.update(IMAGE.findall(text))
        text = IMAGE.sub(lambda m: f"](screenshots/{m.group(1)})", text)
        text = text.replace("ASSETS/", assets + "assets/")
        (src / name).write_text(text, encoding="utf-8")
    if images:
        (src / "screenshots").mkdir()
        for image in sorted(images):
            shutil.copy(SCREENSHOTS / image, src / "screenshots" / image)
    (src / "SUMMARY.md").write_text(summary(lang, pages, topics), encoding="utf-8")

    theme = dest / "theme"
    theme.mkdir()
    for name in ("xylem.css", "languages.js", "site.js"):
        shutil.copy(SITE / "theme" / name, dest / name)
    shutil.copy(REPO / "docs" / "icons" / "xylem.svg", theme / "favicon.svg")
    shutil.copy(REPO / "docs" / "icons" / "xylem-32.png", theme / "favicon.png")
    description = DESCRIPTION.get(lang, DESCRIPTION["en"])
    url = URL if lang == "en" else f"{URL}{lang}/"
    (theme / "head.hbs").write_text(
        f"""<meta name="description" content="{description}">
<meta name="author" content="André Fernandes">
<meta property="og:type" content="website">
<meta property="og:title" content="Xylem File Manager">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{url}">
<meta property="og:image" content="{URL}assets/social-preview.png">
<meta name="twitter:card" content="summary_large_image">
""",
        encoding="utf-8",
    )
    redirect = ""
    if lang == "en":
        redirect = "\n[output.html.redirect]\n" + "".join(
            f'"{old}" = "{new}"\n' for old, new in sorted(redirects().items())
        )
    (dest / "book.toml").write_text(
        f"""[book]
title = "Xylem File Manager"
language = "{lang}"
src = "src"

[output.html]
theme = "theme"
default-theme = "light"
preferred-dark-theme = "navy"
site-url = "/xylem/{'' if lang == 'en' else lang + '/'}"
git-repository-url = "https://github.com/andrelnfernandes/xylem"
additional-css = ["xylem.css"]
additional-js = ["languages.js", "site.js"]
no-section-label = true

[output.html.search]
limit-results = 20
{redirect}""",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
