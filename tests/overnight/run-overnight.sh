#!/usr/bin/env bash
# The 1.7.1 overnight in-game run: every suite in manifest.json, one after the other, each with a save backup
# and the game lock, the UDP settings swapped in for the suites that need them, and one summary at the end.
#
#   tests/overnight/run-overnight.sh                  # every suite
#   SUITES="guards ai" tests/overnight/run-overnight.sh
#   DRY=1 tests/overnight/run-overnight.sh            # plan only, no game
#
# Needs a swofty built from this branch and a loader built from it (native/build.sh), because the pad settle
# and retry live in the in-game agent:
#   SWOFTY=/path/to/swofty SWOFTY_LOADER_DIR=/path/to/loader tests/overnight/run-overnight.sh
set -uo pipefail
HERE=$(cd "$(dirname "$(readlink -f "$0")")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)

SWOFTY=${SWOFTY:-/mnt/gaming/swofty-std16/test1613/swofty}
export SWOFTY_DATA_DIR=${SWOFTY_DATA_DIR:-/mnt/gaming/swofty-std16/ig12-data}
export SWOFTY_LOADER_DIR=${SWOFTY_LOADER_DIR:-/mnt/gaming/swofty-std16/test1613/loader}
GAME=${GAME:-/mnt/gaming/SteamLibrary/steamapps/common/F1 25}
PFX=${PFX:-/mnt/gaming/SteamLibrary/steamapps/compatdata/3059520/pfx/drive_c/users/steamuser/Documents/My Games/F1 25}
HW=$PFX/hardwaresettings/hardware_settings_config.xml
USERDATA=${USERDATA:-$HOME/.local/share/Steam/userdata/843502855/3059520}
BACKUPS=${BACKUPS:-/mnt/work/f1-save-backups}
LOCK=${SWOFTY_GAME_LOCK:-/mnt/work/.f1-game.lock.d}
WAIT=${WAIT:-3h}
UDP_FORMAT=${UDP_FORMAT:-2026}
UDP_PORT=${UDP_PORT:-20799}
STAMP=$(date +%Y%m%d-%H%M%S)
OUT=${OUT:-/mnt/work/overnight-171/$STAMP}
export SWOFTY_TEST_FIXTURES=${SWOFTY_TEST_FIXTURES:-$(python3 -c "import json,sys;print(json.load(open('$HERE/manifest.json'))['fixtures'])")}
PROFILE=${PROFILE:-$(python3 -c "import json;print(json.load(open('$HERE/manifest.json'))['profile'])")}
mkdir -p "$OUT"
SUMMARY=$OUT/summary.md
HW_SAVED=

restore_hw() {
    if [ -n "$HW_SAVED" ] && [ -f "$HW_SAVED" ]; then
        cp -a "$HW_SAVED" "$HW" && echo "udp settings restored"
        HW_SAVED=
    fi
}
trap restore_hw EXIT INT TERM

say() { echo "[$(date +%H:%M:%S)] $*" | tee -a "$OUT/run.log"; }

backup_saves() {
    local b=$BACKUPS/$STAMP-overnight-$1
    mkdir -p "$b"
    [ -d "$USERDATA" ] && cp -a "$USERDATA" "$b/userdata-3059520"
    [ -d "$PFX" ] && cp -a "$PFX" "$b/mygames-F1-25"
    say "saves backed up to $b"
}

udp_on() {
    [ -f "$HW" ] || { say "no $HW; the UDP suites cannot run"; return 1; }
    HW_SAVED=$OUT/hw.xml
    cp -a "$HW" "$HW_SAVED"
    sed -i -E "s|<udp [^>]*/>|<udp enabled=\"true\" broadcast=\"false\" ip=\"127.0.0.1\" port=\"$UDP_PORT\" sendRate=\"20\" format=\"$UDP_FORMAT\" yourTelemetry=\"restricted\" onlineNames=\"off\" />|" "$HW"
    grep -o '<udp [^>]*/>' "$HW" | tee -a "$OUT/run.log"
}

install_addons() {
    local src=$1 id
    [ -d "$ROOT/tests/overnight/$src" ] || return 0
    id=$(basename "$src")
    if [ ! -d "$SWOFTY_DATA_DIR/addons/$id" ]; then
        cp -a "$ROOT/tests/overnight/$src" "$SWOFTY_DATA_DIR/addons/$id"
        say "installed the test addon $id into $SWOFTY_DATA_DIR/addons"
    else
        rm -rf "$SWOFTY_DATA_DIR/addons/$id" && cp -a "$ROOT/tests/overnight/$src" "$SWOFTY_DATA_DIR/addons/$id"
    fi
}

mapfile -t ROWS < <(python3 - "$HERE/manifest.json" <<'EOF'
import json, sys
for s in json.load(open(sys.argv[1]))["suites"]:
    name = s["file"].removesuffix(".test.json")
    print("\t".join([name, s["file"], s.get("addons_from", "-"), "1" if s.get("udp") else "0",
                     "1" if s.get("session") else "0", "1" if s.get("expect_refusal") else "0", s.get("after", "-")]))
EOF
)

printf '# Overnight run %s\n\n| suite | result | passed | failed | skipped | notes |\n|---|---|---|---|---|---|\n' "$STAMP" > "$SUMMARY"
FAILED=0

for row in "${ROWS[@]}"; do
    IFS=$'\t' read -r name file addons_from udp session refusal after <<<"$row"
    if [ -n "${SUITES:-}" ] && ! [[ " $SUITES " == *" $name "* ]]; then
        continue
    fi
    suite=$HERE/$file
    rdir=$OUT/$name
    mkdir -p "$rdir"
    [ "$addons_from" != "-" ] && install_addons "$addons_from"

    if ! "$SWOFTY" test plan "$suite" --profile "$PROFILE" >"$rdir/plan.txt" 2>&1; then
        reason=$(head -1 "$rdir/plan.txt")
        if [ "$session" = 1 ] && grep -q 'unknown kind "session"' "$rdir/plan.txt"; then
            say "SKIP $name: this swofty has no session environment ($reason)"
            echo "| $name | SKIP | 0 | 0 | all | no session env in this swofty build |" >> "$SUMMARY"
        else
            say "ERROR $name: the suite does not plan: $reason"
            echo "| $name | ERROR | 0 | 0 | 0 | does not plan: $reason |" >> "$SUMMARY"
            FAILED=1
        fi
        continue
    fi
    cat "$rdir/plan.txt" | tee -a "$OUT/run.log"
    if [ "${DRY:-}" = 1 ]; then
        echo "| $name | PLANNED | - | - | - | dry run |" >> "$SUMMARY"
        continue
    fi

    backup_saves "$name"
    if [ "$udp" = 1 ]; then
        udp_on || { echo "| $name | ERROR | 0 | 0 | 0 | no hardware_settings_config.xml |" >> "$SUMMARY"; FAILED=1; continue; }
    fi
    say "RUN $name"
    "$SWOFTY" test run "$suite" --profile "$PROFILE" --game "$GAME" --lock "$LOCK" --wait "$WAIT" --out "$rdir/report" \
        >"$rdir/run.txt" 2>&1
    rc=$?
    [ "$udp" = 1 ] && restore_hw
    latest=$(ls -dt "$SWOFTY_DATA_DIR"/sessions/*/ 2>/dev/null | head -1)
    [ -n "$latest" ] && cp -f "$latest/loader.log" "$rdir/loader.log" 2>/dev/null || true

    if [ "$after" = careers ]; then
        "$SWOFTY" careers >"$rdir/careers.txt" 2>&1 || true
        say "careers after $name: $(wc -l <"$rdir/careers.txt") line(s) in $rdir/careers.txt"
    fi

    python3 - "$rdir/report/report.json" "$name" "$refusal" "$rc" "$SUMMARY" <<'EOF' || FAILED=1
import json, sys
path, name, refusal, rc, summary = sys.argv[1:]
try:
    r = json.load(open(path))
except Exception as e:
    open(summary, "a").write(f"| {name} | ERROR | 0 | 0 | 0 | no report (exit {rc}): {e} |\n")
    sys.exit(1)
passed = failed = skipped = 0
notes = []
for t in r.get("tests", []):
    st = t.get("status")
    why = " ".join(f.get("actual", "") for f in t.get("failures", []))
    unavailable = "unavailable" in why or "needs session setup" in why or "unknown command" in why
    if st == "pass":
        passed += 1
    elif refusal == "1" and st == "fail" and "refus" in why:
        passed += 1
        notes.append(f"{t['id']}: refused as expected")
    elif unavailable and ("session" in name or t.get("env", "").startswith("session")):
        skipped += 1
        notes.append(f"{t['id']}: SKIP {why[:140]}")
    elif st in ("skip", "skipped"):
        skipped += 1
        notes.append(f"{t['id']}: skipped {t.get('reason', '')}")
    else:
        failed += 1
        notes.append(f"{t['id']}: {why[:200] or st}")
result = "PASS" if failed == 0 and passed else ("SKIP" if failed == 0 else "FAIL")
open(summary, "a").write(f"| {name} | {result} | {passed} | {failed} | {skipped} | {'; '.join(notes).replace('|', '/')} |\n")
sys.exit(1 if failed else 0)
EOF
    say "done $name (swofty exit $rc)"
done

say "summary: $SUMMARY"
cat "$SUMMARY"
exit $FAILED
