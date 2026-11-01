#include "runtime.hpp"

#include <windows.h>

#include "session_enter_logic.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <cstdint>
#include <cstring>
#include <vector>

// Session enter AI record guard (re/db/functions.tsv Sess_Ai_BuildVehicleRecords, F1_25.exe+0x38a0e00).
//
// Game_GameModule_OnSessionEnter calls Sess_Ai_BuildVehicleRecords, which builds one 0x188-byte record for each vehicle
// slot; later code reads the records at [manager + 0x128] by vehicle slot. For a vehicle it hashes the car's name and
// looks the hash up in the map of each asset group in [manager + 0x108], stopping at the first group without it (the
// loader's "Asset group not found" line); the nodes found go into a local vector. The record is built from the first
// node with no check that there is one (mov rax,[rax] at 0x1438a41bc). A car that no group lists leaves the vector
// null, and the session start faults reading 0x0 (Supercars and era boxes on community tracks that carry no AI asset
// groups for those cars).
//
// site_Sess_AiRecords_NameHash is the instruction after the name hash is computed (r13 = hash, rcx = manager). When
// group 0's map (an MSVC std::map: head +0 leftmost, +8 root, node left +0, right +0x10, isnil byte +0x19, key +0x20)
// lacks the hash, the guard replaces r13 with the key of the map's leftmost node, so the game's own code finds a node
// and builds a full record in the vehicle's slot; the car borrows another car's AI entry. A hash group 0 lists runs the
// game's code unchanged.
//
// SWOFTY_SESSION_ENTER_GUARD=0 turns it off.

namespace {

constexpr const char *MIXIN = "swofty_core.dll (session enter guard)";
constexpr const char *HASH = "site_Sess_AiRecords_NameHash";

namespace se = runtime::session_enter;

bool offered = false, settled = false, offered_ok = false;
se::Skips skips;

bool read_memory(uint64_t address, void *out, size_t n, void *)
{
    if (!runtime::readable(reinterpret_cast<const void *>(address), n))
        return false;
    memcpy(out, reinterpret_cast<const void *>(address), n);
    return true;
}

void name_hash(SwoftyContext *c)
{
    se::Fix f = se::decide(c->rcx, c->rsp, c->r13, read_memory, nullptr);
    if (!f.replace)
        return;
    uint64_t hash = c->r13;
    c->r13 = f.key;
    int n = skips.note();
    if (!skips.logged(n))
        return;
    swofty::log("[session] vehicle slot %u (hash %llx) is in no AI asset group; using %llx for its AI record%s", f.index,
                (unsigned long long)hash, (unsigned long long)f.key,
                skips.last(n) ? " (further fixes are not logged)" : "");
}

}

namespace runtime {

std::vector<CoreMixins> session_enter_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_SESSION_ENTER_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[session] enter guard off (SWOFTY_SESSION_ENTER_GUARD=%s)", v);
        return out;
    }
    if (!swofty::api->resolve(HASH)) {
        swofty::log("[session] enter guard off: needs %s, which this build does not resolve", HASH);
        return out;
    }
    offered_ok = true;
    out.push_back({MIXIN, {{HASH, SWOFTY_MIXIN_SITE, 100, (void *)&name_hash, nullptr}}});
    return out;
}

void session_enter_commit()
{
    if (!offered || settled || !offered_ok)
        return;
    settled = true;
    swofty::log("[session] enter guard %s",
                core_mixins_active(MIXIN) ? "armed: a car that no AI asset group lists borrows an entry for its AI record"
                                          : "did not install (see the [core] lines)");
}

}
