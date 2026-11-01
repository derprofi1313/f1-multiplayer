#include "runtime.hpp"
#include "watchdog_grace_logic.hpp"

#include <windows.h>

#include <swofty/mixin.hpp>

#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <mutex>
#include <vector>

// Boot watchdog grace (re/db/globals.tsv site_Ne_WatchdogKillPath, site_Ne_WatchdogKillSkip).
//
// Ne_Watchdog_ThreadLoop kills the process (a write to address 1) when the frame tick has not fed the watchdog for
// WatchDog.delay ms (30000). On 14-16 GB machines with big modded profiles a front-end flow's Entering holds the
// main thread in game code for longer than that (median 24 s, up to 120 s against a healthy 7 s), the frame thread
// logs "The signal timed out" and the game dies as the load finishes.
//
// At site_Ne_WatchdogKillPath, reached only when the time since the last tick (r14d, ms) exceeds the max delay,
// the hook continues at site_Ne_WatchdogKillSkip, the branch the game takes while the delay has not run out, when
// a flow is in Entering (career.cpp reports every flow transition) and the silence is under 180 s. A stall with no
// flow in Entering, or one past 180 s, reaches the game's kill unchanged. A kill with the shutdown flag set (dil, the
// object's +0x120) is never held.
//
// SWOFTY_BOOT_WATCHDOG_GRACE=0 turns it off.

namespace {

namespace wg = runtime::watchdog_grace;

constexpr const char *MIXIN = "swofty_core.dll (boot watchdog grace)";

uintptr_t kill_skip = 0;
bool offered = false, settled = false;
std::mutex lock;
wg::Flows flows;
std::atomic<uint32_t> last_bucket{~0u};
std::atomic<uint64_t> held{0};

void kill_path(SwoftyContext *c)
{
    if (static_cast<uint8_t>(c->rdi))
        return;
    uint32_t elapsed = static_cast<uint32_t>(c->r14);
    uint64_t now = GetTickCount64();
    wg::Entering where;
    bool allowed = false;
    if (lock.try_lock()) {
        if (const wg::Entering *e = flows.allows(elapsed, now)) {
            where = *e;
            allowed = true;
        }
        lock.unlock();
    }
    if (!allowed) {
        if (last_bucket.exchange(~0u) != ~0u)
            swofty::log("[core] boot watchdog grace over: the frame tick was silent %u ms (cap %u ms, flows in Entering: %s), "
                        "the game's watchdog proceeds",
                        elapsed, wg::kCapMs, elapsed > wg::kCapMs ? "cap reached" : "none");
        return;
    }
    c->rip = kill_skip;
    held++;
    uint32_t b = wg::bucket(elapsed);
    if (last_bucket.exchange(b) != b)
        swofty::log("[core] boot watchdog grace: the frame tick has been silent %u ms with flow \"%s\" in Entering; the "
                    "game's watchdog kill is held (up to %u ms)",
                    elapsed, where.name, wg::kCapMs);
}

}

namespace runtime {

void watchdog_flow(uint32_t hash, uint32_t to, const char *name)
{
    if (!offered || !kill_skip)
        return;
    std::lock_guard<std::mutex> g(lock);
    flows.flow(hash, to, GetTickCount64(), name);
}

std::vector<CoreMixins> watchdog_grace_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_BOOT_WATCHDOG_GRACE", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[core] boot watchdog grace off (SWOFTY_BOOT_WATCHDOG_GRACE=%s)", v);
        return out;
    }
    kill_skip = swofty::api->resolve("site_Ne_WatchdogKillSkip");
    if (!kill_skip || !swofty::api->resolve("site_Ne_WatchdogKillPath")) {
        kill_skip = 0;
        swofty::log("[core] boot watchdog grace off: needs site_Ne_WatchdogKillPath and site_Ne_WatchdogKillSkip, which "
                    "this build does not resolve");
        return out;
    }
    out.push_back({MIXIN, {{"site_Ne_WatchdogKillPath", SWOFTY_MIXIN_SITE, 100, (void *)&kill_path, nullptr}}});
    return out;
}

void watchdog_grace_commit()
{
    if (!offered || settled || !kill_skip)
        return;
    settled = true;
    swofty::log("[core] boot watchdog grace %s",
                core_mixins_active(MIXIN)
                    ? "armed: the game's watchdog kill is held for up to 180 s while a flow is in Entering"
                    : "did not install (see the [core] lines)");
}

}
