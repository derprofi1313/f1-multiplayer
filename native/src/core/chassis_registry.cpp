#include "runtime.hpp"

#include <windows.h>

#include "pit_box_logic.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstdint>
#include <cstring>
#include <vector>

// Chassis body registry guard (re/db/globals.tsv site_Car_ChassisRegistryCount).
//
// Car_Chassis_Build registers every car's physics body in the registry Game_Init_PhysicsWorldRegistry allocates:
// 0x3BF60 bytes, 26 slots per array, the body count at +0x5d8 incremented with no bound. A grid of 26 cars plus the
// safety car registers 27 bodies, and the 27th writes 0 over the first 8 bytes of the next heap block's header.
//
// At site_Car_ChassisRegistryCount (the test of the registry pointer in rbx, followed by a je that skips the whole
// registration) the guard sets rbx to 0 once the count has reached 26, so the 27th body is not registered.
//
// SWOFTY_CHASSIS_REGISTRY_GUARD=0 turns it off.

namespace {

namespace mem = swofty::mem;
namespace pb = runtime::pit_box;

constexpr const char *MIXIN = "swofty_core.dll (chassis registry guard)";
constexpr const char *SITE = "site_Car_ChassisRegistryCount";
constexpr uintptr_t COUNT = 0x5d8;
constexpr int LOG_FIRST = 8;

bool offered = false, settled = false, wanted = false;
std::atomic<int> skipped{0};

void registry_count(SwoftyContext *c)
{
    uint32_t count = 0;
    if (!c->rbx || !mem::read(c->rbx + COUNT, count) || !pb::registry_full(count))
        return;
    c->rbx = 0;
    int n = ++skipped;
    if (n <= LOG_FIRST)
        swofty::log("[car] chassis registry holds %u bodies; registration #%d is skipped%s", unsigned(count), n,
                    n == LOG_FIRST ? " (further skips are not logged)" : "");
}

}

namespace runtime {

std::vector<CoreMixins> chassis_registry_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_CHASSIS_REGISTRY_GUARD", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[car] chassis registry guard off (SWOFTY_CHASSIS_REGISTRY_GUARD=%s)", v);
        return out;
    }
    if (!swofty::api->resolve(SITE)) {
        swofty::log("[car] chassis registry guard off: needs %s, which this build does not resolve", SITE);
        return out;
    }
    wanted = true;
    out.push_back({MIXIN, {{SITE, SWOFTY_MIXIN_SITE, 100, (void *)&registry_count, nullptr}}});
    return out;
}

void chassis_registry_commit()
{
    if (!offered || settled || !wanted)
        return;
    settled = true;
    swofty::log("[car] chassis registry guard %s",
                core_mixins_active(MIXIN) ? "armed: a 27th car body is not registered"
                                          : "did not install (see the [core] lines)");
}

}
