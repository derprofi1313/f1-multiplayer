#!/usr/bin/env bash
# Builds native/realistic_barriers.dll with zig (the same toolchain as the loader). native/build.sh calls this
# for the release, which then packs the addon into native/bundled/. The host tests run on Windows with
# build.ps1 (llvm-mingw), because they execute the hooks.
set -euo pipefail
SELF=$(readlink -f "$0")
cd "$(dirname "$SELF")"

if command -v zig >/dev/null; then
    ZIG=(zig)
else
    ZIG=(nix run nixpkgs#zig --)
fi

export ZIG_GLOBAL_CACHE_DIR=$PWD/build/zig-cache ZIG_LOCAL_CACHE_DIR=$PWD/build/zig-cache
mkdir -p build

SDK=../../../packages/sdk/include
PIN=()
command -v taskset >/dev/null && PIN=(taskset -c 2-7,10,11,14-31 nice -n 10)

# The version comes from swofty.json, the build stamp from the checkout, so loader.log names the exact build.
VERSION=$(sed -n 's/^  "version": *"\([^"]*\)".*/\1/p' ../swofty.json | head -1)
[[ -n $VERSION ]] || { echo "build.sh: no version in ../swofty.json" >&2; exit 1; }
COMMIT=$(git rev-parse --short=8 HEAD 2>/dev/null || echo local)
BUILD="build $(date -u +%Y-%m-%d) $COMMIT"

"${PIN[@]}" "${ZIG[@]}" c++ -target x86_64-windows-gnu -std=c++23 -O2 -Wall -Wextra -I$SDK -D__USE_MINGW_ANSI_STDIO=1 \
    "-DRB_VERSION=\"$VERSION\"" "-DRB_BUILD=\"$BUILD\"" \
    -shared -s -o realistic_barriers.dll \
    src/barriers.cpp src/detour.cpp

rm -f realistic_barriers.lib realistic_barriers.pdb
echo "built $(pwd)/realistic_barriers.dll ($VERSION, $BUILD)"
