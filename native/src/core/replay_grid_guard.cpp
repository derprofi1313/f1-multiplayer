#include "replay_grid_logic.hpp"
#include "runtime.hpp"

#include <windows.h>

#include <swofty/mixin.hpp>

#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

// Replay recorder holder guard (re/db/globals.tsv site_Ne_ReplayAfterRecord).
//
// Ne_Replay_WriteFrames walks one registry entry per recordable (stride 0x688). Each entry has a state holder, a heap
// block of a 0x80 byte header and a current and a previous buffer, whose class (capacity 0x20 to 0xffff) was picked at
// registration from the state a stock grid needs. Each frame the recordable serializes into a static scratch buffer
// through a stack stream, and the recorder copies that state into the holder's current buffer with the state's full
// size and no check against the holder. A recordable whose state grows with the session's vehicle count (a career
// addon that adds rows to the grid, 25 vehicles on a 22 car plan) outgrows its holder, and the copy writes past the
// block into the next heap block. The next walk calls the vtable of a holder that block overwrote: ACCESS_VIOLATION in
// Ne_Replay_WriteFrames+0x752, at the call of that holder's capacity method.
//
// At site_Ne_ReplayAfterRecord, after the recordable serialized and before either copy, the stream position is
// [rsp+0x98], its base [rsp+0xa0] and the holder's capacity [rsp+0x108]. A state larger than the capacity is cut to
// nothing by moving the position back to the base: the recorder then writes an empty state for that recordable and
// copies no bytes into the holder. The session's grid is not touched. A state that fits runs the game's code unchanged.
// A session of 24 cars or fewer, the size of My Team's stock grid, also runs the game's code unchanged: the recorder
// there writes CannedEventsSystem's state a few bytes past its holder, as it did before this guard, and the replay
// reader expects that state, so an emptied one would be read back as garbage.
// The first skip is logged.
//
// SWOFTY_REPLAY_GRID_GUARD=0 turns it off.

namespace {

constexpr const char *MIXIN = "swofty_core.dll (replay recorder grid guard)";
constexpr const char *SITE = "site_Ne_ReplayAfterRecord";

bool offered = false, settled = false;
runtime::replay_grid::Tally tally;

uint64_t &slot(SwoftyContext *c, uint32_t off)
{
    return *reinterpret_cast<uint64_t *>(c->rsp + off);
}

std::string name_of(uint64_t p)
{
    std::string out;
    if (!p)
        return out;
    for (size_t i = 0; i < 96; i++) {
        char ch = 0;
        if (!runtime::readable(reinterpret_cast<const void *>(p + i), 1))
            break;
        ch = *reinterpret_cast<const char *>(p + i);
        if (!ch || static_cast<unsigned char>(ch) < 0x20 || static_cast<unsigned char>(ch) > 0x7e)
            break;
        out += ch;
    }
    return out;
}

void after_record(SwoftyContext *c)
{
    namespace rg = runtime::replay_grid;
    if (!rg::guards(rg::session_vehicles.load()))
        return;
    uint64_t pos = slot(c, rg::STREAM_POS), base = slot(c, rg::STREAM_BASE);
    uint64_t cap = rg::capacity_of(slot(c, rg::HOLDER_CAPACITY));
    if (!rg::outgrew(pos, base, cap))
        return;
    slot(c, rg::STREAM_POS) = base;
    if (tally.first())
        swofty::log("%s", rg::line(rg::session_vehicles.load(), name_of(c->r13), pos - base, cap).c_str());
}

}

namespace runtime {

std::vector<CoreMixins> replay_grid_guard_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_REPLAY_GRID_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[replay] recorder grid guard off (SWOFTY_REPLAY_GRID_GUARD=%s)", v);
        return out;
    }
    if (!swofty::api->resolve(SITE)) {
        swofty::log("[replay] recorder grid guard off: needs %s, which this build does not resolve", SITE);
        return out;
    }
    out.push_back({MIXIN, {{SITE, SWOFTY_MIXIN_SITE, 100, (void *)&after_record, nullptr}}});
    return out;
}

void replay_grid_guard_commit()
{
    if (!offered || settled)
        return;
    settled = true;
    swofty::log("[replay] recorder grid guard %s",
                core_mixins_active(MIXIN) ? "armed: a recordable whose state outgrows its holder is not recorded"
                                          : "did not install (see the [core] lines)");
}

}
