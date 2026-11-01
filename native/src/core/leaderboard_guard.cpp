#include "leaderboard_guard_logic.hpp"
#include "runtime.hpp"

#include <windows.h>

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <vector>

// Time Trial leaderboard guard (re/db/functions.tsv Game_TimeTrialLeaderboard_SelectBoard).
//
// Pressing View Leaderboards in Time Trial runs Game_TimeTrialLeaderboard_SelectBoard through the
// leaderboard_container binding. It builds the board key 'LBV%02u%02d%s' from the track and the weather, then
// searches the leaderboard list (self+0x478 -> +0x10 -> +0x18..+0x20) for the board whose name (+0x68) equals the
// key, and reads the board's rank thresholds (+0x88, +0x98..+0xa0). The search leaves 0 in rcx when no board
// carries the key, and the read has no null check: with the online services off (the list is empty) or at a track
// without a board, the first read faults (ACCESS_VIOLATION reading 0x98 at F1_25.exe+0x2fce9e3).
//
// site_Game_TimeTrialLeaderboard_BoardRead is that read (rcx = the board). When the board is null the guard sets
// esi to 0, the function's return value, and continues at site_Game_TimeTrialLeaderboard_BoardsDone, the function's
// own path after the board work, which releases its locals and returns esi. The caller that tests the result
// takes its failure branch, and the other caller ignores it. A board that was found runs the game's code
// unchanged.
//
// Activating the leaderboard tab runs Sess_TimeTrialLeaderboard_ActivateTab (vtable slot 0x13 of the same model), which
// reads the child list model at self+0x9a0 and calls through it with no null check (F1_25.exe+0x2fcbb09 reading 0). The
// child is null when the screen opened for a track the view skipped above. site_Sess_TimeTrialLeaderboard_TabChild is
// that read (rcx = the child). When it is null the guard continues at site_Sess_TimeTrialLeaderboard_TabDone, the
// function's own epilogue, which skips the child call and the refresh that follows it. A child that exists runs the
// game's code unchanged.
//
// SWOFTY_LEADERBOARD_GUARD=0 turns the board guard off, SWOFTY_LEADERBOARD_TAB_GUARD=0 the tab guard.

namespace {

constexpr const char *MIXIN = "swofty_core.dll (time trial leaderboard guard)";
namespace lg = runtime::leaderboard_guard;

constexpr const char *TAB_MIXIN = "swofty_core.dll (time trial leaderboard tab guard)";

uintptr_t boards_done = 0, tab_done = 0;
bool offered = false, settled = false;
std::atomic<int> skipped{0}, tab_skipped{0};

bool env_off(const char *name, const char *label)
{
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA(name, v, sizeof v);
    if (!n || n >= sizeof v || !lg::switch_off(v))
        return false;
    swofty::log("[leaderboard] %s off (%s=%s)", label, name, v);
    return true;
}

void board_read(SwoftyContext *c)
{
    if (!lg::board_missing(c->rcx))
        return;
    c->rsi = 0;
    c->rip = boards_done;
    lg::Say say = lg::say(++skipped);
    if (say == lg::Say::none)
        return;
    swofty::log("[leaderboard] Time Trial leaderboard view 0x%llx has no board for this track (online services off or no "
                "board listed); view skipped%s",
                (unsigned long long)c->rdi, say == lg::Say::last ? " (further skips are not logged)" : "");
}

void tab_child_read(SwoftyContext *c)
{
    if (!lg::tab_child_missing(c->rcx))
        return;
    c->rip = tab_done;
    lg::Say say = lg::say(++tab_skipped);
    if (say == lg::Say::none)
        return;
    swofty::log("[leaderboard] Time Trial leaderboard model 0x%llx activated its tab with no child list model; tab "
                "activation skipped%s",
                (unsigned long long)(c->rsi - 0x660), say == lg::Say::last ? " (further skips are not logged)" : "");
}

}

namespace runtime {

std::vector<CoreMixins> leaderboard_guard_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    if (!env_off("SWOFTY_LEADERBOARD_GUARD", "time trial leaderboard guard")) {
        boards_done = swofty::api->resolve("site_Game_TimeTrialLeaderboard_BoardsDone");
        if (!boards_done || !swofty::api->resolve("site_Game_TimeTrialLeaderboard_BoardRead")) {
            boards_done = 0;
            swofty::log("[leaderboard] time trial leaderboard guard off: needs site_Game_TimeTrialLeaderboard_BoardRead "
                        "and site_Game_TimeTrialLeaderboard_BoardsDone, which this build does not resolve");
        } else {
            out.push_back({MIXIN,
                           {{"site_Game_TimeTrialLeaderboard_BoardRead", SWOFTY_MIXIN_SITE, 100, (void *)&board_read, nullptr}}});
        }
    }
    if (!env_off("SWOFTY_LEADERBOARD_TAB_GUARD", "time trial leaderboard tab guard")) {
        tab_done = swofty::api->resolve("site_Sess_TimeTrialLeaderboard_TabDone");
        if (!tab_done || !swofty::api->resolve("site_Sess_TimeTrialLeaderboard_TabChild")) {
            tab_done = 0;
            swofty::log("[leaderboard] time trial leaderboard tab guard off: needs site_Sess_TimeTrialLeaderboard_TabChild "
                        "and site_Sess_TimeTrialLeaderboard_TabDone, which this build does not resolve");
        } else {
            out.push_back({TAB_MIXIN,
                           {{"site_Sess_TimeTrialLeaderboard_TabChild", SWOFTY_MIXIN_SITE, 100, (void *)&tab_child_read, nullptr}}});
        }
    }
    return out;
}

void leaderboard_guard_commit()
{
    if (!offered || settled)
        return;
    settled = true;
    if (boards_done)
        swofty::log("[leaderboard] time trial leaderboard guard %s",
                    core_mixins_active(MIXIN)
                        ? "armed: View Leaderboards at a track without a board returns instead of faulting"
                        : "did not install (see the [core] lines)");
    if (tab_done)
        swofty::log("[leaderboard] time trial leaderboard tab guard %s",
                    core_mixins_active(TAB_MIXIN)
                        ? "armed: activating the leaderboard tab without a child list model returns instead of faulting"
                        : "did not install (see the [core] lines)");
}

}
