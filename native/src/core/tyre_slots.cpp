#include "runtime.hpp"

#include <windows.h>

#include "tyre_slots_logic.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <vector>

// Tyre slot guard (re/db/globals.tsv site_Sess_TyreVisSlotLookup, site_Sess_TyreWearSlotLookup).
//
// A tyre component keeps a slot table (+0x9bd0, at most 5 slots: option, physical compound, visual compound; count
// +0x9c48) built from the car's own compound class (+0x9bc0, from the car entity's 'compound class') and the
// session's TrackDataSet. A slot exists for each option whose physical mask is set; a dry option holding several
// compounds is narrowed to the track's compound, and drops out when the class has none of the track's compounds.
// The fitted set (+0x9c98) comes from the session's allocation, whose class can be another one. When neither the
// set's option nor its fallback (option >= 3 ? 4 : 1) has a slot, Sess_Tyre_ShowFittedSet's fallback scan runs past
// the table (ACCESS_VIOLATION in Game_CompoundModelSet_SetTyreVisibility) and Car_Tyre_SetWearFactor formats the
// wheel compound through a null slot (ACCESS_VIOLATION reading 0x18 at F1_25.exe+0x2ea27c1). Supercars (F1 23
// port) hit it in Time Trial: its cars use the F2 car template's F2-layout class (Medium, Harder, Wet), the
// session's F1 track compounds narrow both dry options away, and a dry set finds only the Wet slot.
//
// Both sites sit right after the game's own table rebuild. A table whose pick is usable runs the game's code
// unchanged. A pick with a null compound takes a donor slot's compounds; a missing pick gets a slot for the option
// with a donor slot's compounds (a slot of the same side, dry or wet, first), so the car runs on a compound of its
// own class. With no usable slot, ShowFittedSet continues at its tail call and SetWearFactor returns, leaving the
// wheels as they are.
//
// SWOFTY_TYRE_SLOT_GUARD=0 turns it off.

namespace {

namespace mem = swofty::mem;
namespace ts = runtime::tyre_slots;

constexpr const char *MIXIN = "swofty_core.dll (tyre slot guard)";
constexpr const char *VIS_SITE = "site_Sess_TyreVisSlotLookup";
constexpr const char *VIS_TAIL = "site_Sess_TyreVisTailCall";
constexpr const char *WEAR_SITE = "site_Sess_TyreWearSlotLookup";
constexpr const char *WEAR_RETURN = "site_Sess_TyreWearReturn";
constexpr size_t TC_CAR = 0x20;
constexpr size_t CAR_LIVE = 0x58;
constexpr size_t LIVE_VEHICLE_ID = 0xe8;
constexpr size_t TC_SLOTS = 0x9bd0;
constexpr size_t TC_COUNT = 0x9c48;
constexpr size_t TC_FITTED = 0x9c98;
constexpr ptrdiff_t FITTED_OPTION = -0x6c;
constexpr size_t SLOT_STRIDE = 0x18;
constexpr size_t SLOT_PHYSICAL = 0x8;
constexpr size_t SLOT_VISUAL = 0x10;
constexpr int LOG_FIRST = 8;

uintptr_t vis_tail = 0, wear_return = 0;
bool offered = false, settled = false;
std::atomic<int> logged{0};

uint8_t vehicle_of(uintptr_t tc)
{
    uintptr_t car = 0, live = 0;
    uint8_t id = 0xff;
    if (mem::read(tc + TC_CAR, car) && car && mem::read(car + CAR_LIVE, live) && live)
        mem::read(live + LIVE_VEHICLE_ID, id);
    return id;
}

const char *name_of(ts::Kind k)
{
    switch (k) {
    case ts::Kind::fill:
        return "filled";
    case ts::Kind::add:
        return "added";
    case ts::Kind::skip:
        return "skipped";
    default:
        return "kept";
    }
}

void report(const char *where, uintptr_t tc, uint32_t option, const ts::Slot *slots, uint64_t count, const ts::Plan &p)
{
    int n = ++logged;
    if (n > LOG_FIRST)
        return;
    char list[64] = {};
    size_t at = 0;
    for (uint64_t i = 0; i < count && i < ts::CAPACITY && at + 8 < sizeof list; i++)
        at += size_t(std::snprintf(list + at, sizeof list - at, "%s%u", i ? "," : "", slots[i].option));
    if (p.kind == ts::Kind::skip)
        swofty::log("[tyres] slot guard (%s): vehicle %u's fitted option %u has no usable slot in its class's table "
                    "(count %llu, options %s); refit skipped%s",
                    where, unsigned(vehicle_of(tc)), option, (unsigned long long)count, at ? list : "none",
                    n == LOG_FIRST ? " (further repairs are not logged)" : "");
    else
        swofty::log("[tyres] slot guard (%s): vehicle %u's fitted option %u has no usable slot in its class's table "
                    "(count %llu, options %s); slot %d %s with the compounds of option %u's slot%s",
                    where, unsigned(vehicle_of(tc)), option, (unsigned long long)count, at ? list : "none", p.index,
                    name_of(p.kind), slots[p.donor].option, n == LOG_FIRST ? " (further repairs are not logged)" : "");
}

bool repair(uintptr_t tc, const char *where)
{
    uint64_t count = 0;
    uintptr_t fitted = 0;
    uint32_t option = 0;
    if (!mem::object_like(tc) || !mem::read(tc + TC_COUNT, count) || !mem::read(tc + TC_FITTED, fitted) || !fitted ||
        !mem::read(fitted + FITTED_OPTION, option))
        return true;
    ts::Slot slots[ts::CAPACITY] = {};
    uint64_t n = count <= ts::CAPACITY ? count : 0;
    for (uint64_t i = 0; i < n; i++) {
        uintptr_t s = tc + TC_SLOTS + i * SLOT_STRIDE;
        if (!mem::read(s, slots[i].option) || !mem::read(s + SLOT_PHYSICAL, slots[i].physical) ||
            !mem::read(s + SLOT_VISUAL, slots[i].visual))
            return true;
    }
    auto p = ts::plan(slots, count, option);
    if (p.kind == ts::Kind::keep)
        return true;
    if (p.kind == ts::Kind::skip) {
        report(where, tc, option, slots, count, p);
        return false;
    }
    const ts::Slot &d = slots[p.donor];
    uintptr_t s = tc + TC_SLOTS + size_t(p.index) * SLOT_STRIDE;
    if (p.kind == ts::Kind::add)
        std::memcpy(reinterpret_cast<void *>(s), &option, sizeof option);
    std::memcpy(reinterpret_cast<void *>(s + SLOT_PHYSICAL), &d.physical, sizeof d.physical);
    std::memcpy(reinterpret_cast<void *>(s + SLOT_VISUAL), &d.visual, sizeof d.visual);
    if (p.kind == ts::Kind::add) {
        uint64_t grown = count + 1;
        std::memcpy(reinterpret_cast<void *>(tc + TC_COUNT), &grown, sizeof grown);
    }
    report(where, tc, option, slots, count, p);
    return true;
}

void vis_lookup(SwoftyContext *c)
{
    if (!repair(c->r14, "set visibility"))
        c->rip = vis_tail;
}

void wear_lookup(SwoftyContext *c)
{
    if (!repair(c->rsi, "wear factor"))
        c->rip = wear_return;
}

}

namespace runtime {

std::vector<CoreMixins> tyre_slots_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_TYRE_SLOT_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[tyres] slot guard off (SWOFTY_TYRE_SLOT_GUARD=%s)", v);
        return out;
    }
    vis_tail = swofty::api->resolve(VIS_TAIL);
    wear_return = swofty::api->resolve(WEAR_RETURN);
    if (!vis_tail || !wear_return || !swofty::api->resolve(VIS_SITE) || !swofty::api->resolve(WEAR_SITE)) {
        vis_tail = wear_return = 0;
        swofty::log("[tyres] slot guard off: needs %s, %s, %s and %s, which this build does not resolve", VIS_SITE,
                    VIS_TAIL, WEAR_SITE, WEAR_RETURN);
        return out;
    }
    out.push_back({MIXIN,
                   {{VIS_SITE, SWOFTY_MIXIN_SITE, 100, (void *)&vis_lookup, nullptr},
                    {WEAR_SITE, SWOFTY_MIXIN_SITE, 100, (void *)&wear_lookup, nullptr}}});
    return out;
}

void tyre_slots_commit()
{
    if (!offered || settled || !vis_tail)
        return;
    settled = true;
    swofty::log("[tyres] slot guard %s",
                core_mixins_active(MIXIN)
                    ? "armed: a fitted tyre set whose option has no slot in the car's class takes a slot of the "
                      "class instead of faulting"
                    : "did not install (see the [core] lines)");
}

}
