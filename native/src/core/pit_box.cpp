#include "runtime.hpp"

#include <windows.h>

#include "pit_box_logic.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <cstdint>
#include <cstring>
#include <vector>

// Pit box guard (re/db/globals.tsv site_Game_PitLane_BoxSlotRead, site_Game_PitLane_BoxPoseFalse).
//
// Game_PitLane_UpdateMovement keeps a car that sits in its garage aligned to its pit box through
// Game_PitLane_ProjectBoxPose(mgr, out, mtx, box, flags), where box is the +0xe0 of the car's pit lane record. The
// function caches one spline handle per box at [mgr + 0x7f8 + box * 8], 40 slots up to the per-car records at
// mgr + 0x938, with no bound on box. A garage holds two cars; a third car (Classic Career seats more than two cars
// to a team) has no box, its +0xe0 is -1, and the read at 0x1434c3656 faults ("reading mgr + 0x800000000 + 0x7f0").
// Garage crew guard (#613) already keeps that car out of the crew spawn; the pit lane then reaches it in the
// hold-in-garage states 10 and 11.
//
// At site_Game_PitLane_BoxSlotRead the guard looks at the box (edi). A box of 40 or more sends the function to its
// shared epilogue with edi = 2, the value its own failed projection leaves there, so it returns false having written
// nothing and the caller leaves the car where it is.
//
// SWOFTY_PIT_BOX_GUARD=0 turns it off.

namespace {

constexpr const char *MIXIN = "swofty_core.dll (pit box guard)";
constexpr const char *READ = "site_Game_PitLane_BoxSlotRead";
constexpr const char *FALSE_ = "site_Game_PitLane_BoxPoseFalse";
constexpr uintptr_t PROJECTION_FAILED = 2;

namespace pb = runtime::pit_box;

uintptr_t pose_false = 0;
bool offered = false, settled = false, wanted = false;
pb::Seen seen;

void box_read(SwoftyContext *c)
{
    uint32_t box = uint32_t(c->rdi);
    if (pb::box_usable(box))
        return;
    pb::Log l = seen.note(box);
    if (l != pb::Log::none)
        swofty::log("[pit] guard: a car in the garage has pit box %d, which has no spline slot; it is not aligned%s",
                    int32_t(box), l == pb::Log::last ? " (further repairs are not logged)" : "");
    c->rdi = PROJECTION_FAILED;
    c->rip = pose_false;
}

}

namespace runtime {

std::vector<CoreMixins> pit_box_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_PIT_BOX_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[pit] box guard off (SWOFTY_PIT_BOX_GUARD=%s)", v);
        return out;
    }
    pose_false = swofty::api->resolve(FALSE_);
    if (!pose_false || !swofty::api->resolve(READ)) {
        pose_false = 0;
        swofty::log("[pit] box guard off: needs %s and %s, which this build does not resolve", READ, FALSE_);
        return out;
    }
    wanted = true;
    out.push_back({MIXIN, {{READ, SWOFTY_MIXIN_SITE, 100, (void *)&box_read, nullptr}}});
    return out;
}

void pit_box_commit()
{
    if (!offered || settled || !wanted)
        return;
    settled = true;
    swofty::log("[pit] box guard %s",
                core_mixins_active(MIXIN) ? "armed: a garage car without a pit box is left where it is"
                                          : "did not install (see the [core] lines)");
}

}
