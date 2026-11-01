#include "runtime.hpp"

#include <windows.h>

#include "pit_slots_logic.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <vector>

// Pit slot guard (re/db/globals.tsv site_Sess_PitEvalOwnSlot, site_Sess_PitEvalListSlot, site_Sess_PitRankOwnSlot,
// site_Sess_PitRankListSlot).
//
// The AI pit-decision evaluators Sess_PitStrategy_EvaluateAgainstPitList and Sess_PitStrategy_EvaluateByRank compare
// the car's race position with that of every car in the global pit list {u8 vehicle, f32 time}. A position is read
// from [[session + 0x78] + 0x148][vehicle] at +0x48 and +0x50, with no check that the vehicle has a slot. Nothing ever
// erases a pit list entry, so an entry can name a vehicle whose slot the session no longer holds (Classic Career 2012:
// vehicle 21 null, slots 0 to 20 and 22 filled), and the read faults on null + 0x48 (0x143692014). The own car's slot
// is read the same way.
//
// At each of the four reads the guard looks at the slot just loaded. A listed car without a slot is skipped, as if it
// were not in the list. An own car without a slot ends the evaluation with false, so the car makes no pit decision on
// that call.
//
// SWOFTY_PIT_SLOT_GUARD=0 turns it off.

namespace {

namespace mem = swofty::mem;
namespace ps = runtime::pit_slots;

constexpr const char *MIXIN = "swofty_core.dll (pit slot guard)";
constexpr const char *EVAL_OWN = "site_Sess_PitEvalOwnSlot";
constexpr const char *EVAL_LIST = "site_Sess_PitEvalListSlot";
constexpr const char *EVAL_NEXT = "site_Sess_PitEvalListNext";
constexpr const char *EVAL_FALSE = "site_Sess_PitEvalReturnFalse";
constexpr const char *RANK_OWN = "site_Sess_PitRankOwnSlot";
constexpr const char *RANK_LIST = "site_Sess_PitRankListSlot";
constexpr const char *RANK_NEXT = "site_Sess_PitRankListNext";
constexpr const char *RANK_END = "site_Sess_PitRankEpilogue";
constexpr size_t RECORD_VEHICLE = 0xc4;

uintptr_t eval_next = 0, eval_false = 0, rank_next = 0, rank_end = 0;
bool offered = false, settled = false, wanted = false;
ps::Seen seen;

const char *where_of(ps::Site s)
{
    switch (s) {
    case ps::Site::eval_own:
    case ps::Site::rank_own:
        return "its own";
    default:
        return "a listed";
    }
}

void report(ps::Site site, uint8_t vehicle)
{
    ps::Log l = seen.note(site, vehicle);
    if (l == ps::Log::none)
        return;
    swofty::log("[pit] guard: vehicle %u has no race slot, %s car in the pit strategy check; %s%s", unsigned(vehicle),
                where_of(site),
                site == ps::Site::eval_own || site == ps::Site::rank_own ? "the check answers no" : "the entry is skipped",
                l == ps::Log::last ? " (further repairs are not logged)" : "");
}

uint8_t byte_at(uintptr_t addr)
{
    uint8_t v = 0xff;
    if (addr)
        mem::read(addr, v);
    return v;
}

void eval_own_slot(SwoftyContext *c)
{
    if (ps::usable(c->rcx))
        return;
    report(ps::Site::eval_own, byte_at(c->rdi + RECORD_VEHICLE));
    c->rip = eval_false;
}

void eval_list_slot(SwoftyContext *c)
{
    if (ps::usable(c->rax))
        return;
    report(ps::Site::eval_list, uint8_t(c->rdx));
    c->rbx = 0;
    c->rip = eval_next;
}

void rank_own_slot(SwoftyContext *c)
{
    if (ps::usable(c->rax))
        return;
    report(ps::Site::rank_own, byte_at(c->rsi + RECORD_VEHICLE));
    c->rax = 0;
    c->rip = rank_end;
}

void rank_list_slot(SwoftyContext *c)
{
    if (ps::usable(c->rdx))
        return;
    report(ps::Site::rank_list, byte_at(c->rax));
    c->rip = rank_next;
}

}

namespace runtime {

std::vector<CoreMixins> pit_slots_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_PIT_SLOT_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[pit] slot guard off (SWOFTY_PIT_SLOT_GUARD=%s)", v);
        return out;
    }
    eval_next = swofty::api->resolve(EVAL_NEXT);
    eval_false = swofty::api->resolve(EVAL_FALSE);
    rank_next = swofty::api->resolve(RANK_NEXT);
    rank_end = swofty::api->resolve(RANK_END);
    if (!eval_next || !eval_false || !rank_next || !rank_end || !swofty::api->resolve(EVAL_OWN) ||
        !swofty::api->resolve(EVAL_LIST) || !swofty::api->resolve(RANK_OWN) || !swofty::api->resolve(RANK_LIST)) {
        eval_next = eval_false = rank_next = rank_end = 0;
        swofty::log("[pit] slot guard off: needs %s, %s, %s, %s, %s, %s, %s and %s, which this build does not resolve",
                    EVAL_OWN, EVAL_LIST, EVAL_NEXT, EVAL_FALSE, RANK_OWN, RANK_LIST, RANK_NEXT, RANK_END);
        return out;
    }
    wanted = true;
    out.push_back({MIXIN,
                   {{EVAL_OWN, SWOFTY_MIXIN_SITE, 100, (void *)&eval_own_slot, nullptr},
                    {EVAL_LIST, SWOFTY_MIXIN_SITE, 100, (void *)&eval_list_slot, nullptr},
                    {RANK_OWN, SWOFTY_MIXIN_SITE, 100, (void *)&rank_own_slot, nullptr},
                    {RANK_LIST, SWOFTY_MIXIN_SITE, 100, (void *)&rank_list_slot, nullptr}}});
    return out;
}

void pit_slots_commit()
{
    if (!offered || settled || !wanted)
        return;
    settled = true;
    swofty::log("[pit] slot guard %s",
                core_mixins_active(MIXIN)
                    ? "armed: a pit list car without a race slot is skipped, and a car without one makes no pit decision"
                    : "did not install (see the [core] lines)");
}

}
