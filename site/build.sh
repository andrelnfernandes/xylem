#!/bin/sh
# Assembles the website in _site/ (or in the folder given): the pages in
# site/ plus the images from docs/, so they are not kept twice, and the
# manual from docs/manual/ as one mdBook per language: /manual/ (English) and
# /manual/<lang>/. Needs python3 and mdbook. Used by
# .github/workflows/pages.yml; to look at it locally:
#   sh site/build.sh && python3 -m http.server -d _site
set -eu
cd "$(dirname "$0")/.."
out=${1:-_site}
rm -rf "$out"
mkdir -p "$out/assets/screenshots"
cp -r site/. "$out/"
rm -r "$out/build.sh" "$out/manual"
cp docs/social-preview.png docs/icons/xylem.svg docs/icons/xylem-32.png "$out/assets/"
cp docs/screenshots/*.png docs/screenshots/*.gif "$out/assets/screenshots/"
books="$out/.books"
for lang in en pt es fr de zh; do
  python3 -I site/manual/book.py "$lang" "$books/$lang"
  MDBOOK_LOG=${MDBOOK_LOG:-warn} mdbook build "$books/$lang" -d "$books/$lang/out"
done
mv "$books/en/out" "$out/manual"
for lang in pt es fr de zh; do
  mv "$books/$lang/out" "$out/manual/$lang"
done
rm -r "$books"
