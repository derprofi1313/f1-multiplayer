#!/usr/bin/env bash
# The repository ships no track art. This copies an archive of your own to the file manifest.json's "erp" names
# (hockenheim.erp) and its mipmaps to track/mipmaps/: from an imported track's folder (its one .erp and mipmaps/), or
# the files you name. The archive must be built for the manifest's track_code.
#   track/standin.sh ~/tracks/hockenheim/                     the folder's .erp and mipmaps/
#   track/standin.sh ~/tracks/hockenheim.erp ~/tracks/mips/    an archive and a mipmaps folder
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")"
SRC=${1:?"usage: track/standin.sh <track folder> | <archive.erp> [mipmaps folder]"}
NAME=$(sed -n 's/.*"erp": *"\([^"]*\)".*/\1/p' manifest.json)
[ -n "$NAME" ] || { echo "track/standin.sh: manifest.json names no erp" >&2; exit 2; }
if [ -d "$SRC" ]; then
    ERP=$(find "$SRC" -maxdepth 1 -name '*.erp' | head -1)
    MIPS=$SRC/mipmaps
else
    ERP=$SRC
    MIPS=${2:-}
fi
[ -f "$ERP" ] || { echo "track/standin.sh: no .erp in $SRC" >&2; exit 2; }
cp "$ERP" "$NAME"
echo "track/$NAME <- $ERP"
if [ -n "$MIPS" ] && [ -d "$MIPS" ]; then
    rm -rf mipmaps
    cp -r "$MIPS" mipmaps
    echo "track/mipmaps/ <- $MIPS"
fi
