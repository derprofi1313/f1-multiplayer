#include "runtime.hpp"

#include <windows.h>

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <vector>

// Track Acclimatisation guards (re/db/functions.tsv Game_PracticeTrackAcclimatisation_RefreshState and
// Game_PracticeTrackAcclimatisation_FlagCurrentTarget).
//
// The Track Acclimatisation practice programme copies its targets (0x370-byte corner entries at self+0x350..+0x358)
// from the track's TrackAcclimatisationResource (Game_PracticeTrackAcclimatisation_CopyTargets). A track without
// that resource's CornerData, such as an imported community track, leaves the vector empty and null. The
// programme's start and most of its other slots return when the vector is empty, but two slots index the current
// entry without that check, and the practice programmes' message handlers run them for every programme:
// - the refresh (slot +0xf0), as the car leaves the garage: ACCESS_VIOLATION reading 0x4 at F1_25.exe+0x299e59b;
// - the penalty flag (slot +0x1a8), on a penalty or warning for the car: ACCESS_VIOLATION reading 0xc4 at
//   F1_25.exe+0x29a0634.
//
// Both guards return from their slot when the vector is empty, as the programme's start does. A programme with
// targets runs the game's code unchanged. The penalty flag is a leaf without unwind data, so its guard is a site at
// its entry (site_Game_AcclimPenalty_Entry) that continues at its ret (site_Game_AcclimPenalty_Return).
//
// SWOFTY_ACCLIM_GUARD=0 turns both off.

namespace {

namespace mem = swofty::mem;

constexpr const char *MIXIN = "swofty_core.dll (track acclimatisation guard)";
constexpr const char *PENALTY_MIXIN = "swofty_core.dll (track acclimatisation penalty guard)";
constexpr const char *TARGET = "Game_PracticeTrackAcclimatisation_RefreshState";
constexpr const char *PENALTY_ENTRY = "site_Game_AcclimPenalty_Entry";
constexpr const char *PENALTY_RETURN = "site_Game_AcclimPenalty_Return";
constexpr size_t TARGETS_BEGIN = 0x350;
constexpr size_t TARGETS_END = 0x358;
constexpr int LOG_FIRST = 4;

using Refresh = void(void *);
Refresh *orig_refresh = nullptr;
uintptr_t penalty_return = 0;
bool offered = false, settled = false, wanted = false;
std::atomic<int> skipped{0}, penalty_skipped{0};

void refresh(void *self)
{
    uintptr_t begin = 0, end = 0;
    memcpy(&begin, static_cast<const uint8_t *>(self) + TARGETS_BEGIN, sizeof begin);
    memcpy(&end, static_cast<const uint8_t *>(self) + TARGETS_END, sizeof end);
    if (begin != end) {
        orig_refresh(self);
        return;
    }
    int n = ++skipped;
    if (n > LOG_FIRST)
        return;
    swofty::log("[acclim] Track Acclimatisation programme 0x%llx has no targets at this track (targets 0x%llx); "
                "refresh skipped%s",
                (unsigned long long)(uintptr_t)self, (unsigned long long)begin,
                n == LOG_FIRST ? " (further skips are not logged)" : "");
}

void penalty_entry(SwoftyContext *c)
{
    uintptr_t begin = 0, end = 0;
    if (!mem::read(c->rcx + TARGETS_BEGIN, begin) || !mem::read(c->rcx + TARGETS_END, end) || begin != end)
        return;
    c->rip = penalty_return;
    int n = ++penalty_skipped;
    if (n > LOG_FIRST)
        return;
    swofty::log("[acclim] Track Acclimatisation programme 0x%llx has no targets at this track (targets 0x%llx); "
                "penalty flag skipped%s",
                (unsigned long long)c->rcx, (unsigned long long)begin,
                n == LOG_FIRST ? " (further skips are not logged)" : "");
}

}

namespace runtime {

std::vector<CoreMixins> track_acclim_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_ACCLIM_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[acclim] track acclimatisation guard off (SWOFTY_ACCLIM_GUARD=%s)", v);
        return out;
    }
    if (swofty::api->resolve(TARGET)) {
        wanted = true;
        out.push_back({MIXIN, {{TARGET, SWOFTY_MIXIN_WRAP, 100, (void *)&refresh, (void **)&orig_refresh}}});
    } else {
        swofty::log("[acclim] track acclimatisation guard off: needs %s, which this build does not resolve", TARGET);
    }
    penalty_return = swofty::api->resolve(PENALTY_RETURN);
    if (penalty_return && swofty::api->resolve(PENALTY_ENTRY)) {
        out.push_back({PENALTY_MIXIN, {{PENALTY_ENTRY, SWOFTY_MIXIN_SITE, 100, (void *)&penalty_entry, nullptr}}});
    } else {
        penalty_return = 0;
        swofty::log("[acclim] track acclimatisation penalty guard off: needs %s and %s, which this build does not "
                    "resolve",
                    PENALTY_ENTRY, PENALTY_RETURN);
    }
    return out;
}

void track_acclim_commit()
{
    if (!offered || settled)
        return;
    settled = true;
    if (wanted)
        swofty::log("[acclim] track acclimatisation guard %s",
                    core_mixins_active(MIXIN)
                        ? "armed: a practice programme refresh with no targets returns instead of faulting"
                        : "did not install (see the [core] lines)");
    if (penalty_return)
        swofty::log("[acclim] track acclimatisation penalty guard %s",
                    core_mixins_active(PENALTY_MIXIN)
                        ? "armed: a practice programme penalty flag with no targets returns instead of faulting"
                        : "did not install (see the [core] lines)");
}

}
