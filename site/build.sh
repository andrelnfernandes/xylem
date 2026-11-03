#!/bin/sh
# Assembles the website in _site/ (or in the folder given): one mdBook per
# language, English at the root and the others under /<lang>/, each with the
# pages of site/pages/ and then the manual from docs/manual/ (site/book.py
# prepares them), plus the images from docs/ in /assets/, so they are not kept
# twice. Needs python3 and mdbook. Used by .github/workflows/pages.yml; to look
# at it locally:
#   sh site/build.sh && python3 -m http.server -d _site
set -eu
cd "$(dirname "$0")/.."
out=${1:-_site}
books="${out%/}.books"
rm -rf "$out" "$books"
for lang in en pt es fr de zh; do
  python3 -I site/book.py "$lang" "$books/$lang"
  MDBOOK_LOG=${MDBOOK_LOG:-warn} mdbook build "$books/$lang" -d "$books/$lang/out"
done
mv "$books/en/out" "$out"
for lang in pt es fr de zh; do
  mv "$books/$lang/out" "$out/$lang"
done
rm -r "$books"
mkdir -p "$out/assets/screenshots"
cp docs/social-preview.png docs/icons/xylem.svg docs/icons/xylem-32.png "$out/assets/"
cp docs/screenshots/*.png docs/screenshots/*.gif "$out/assets/screenshots/"
