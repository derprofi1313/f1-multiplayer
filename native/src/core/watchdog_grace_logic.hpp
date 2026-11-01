#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>

// The boot watchdog grace (re/db/globals.tsv site_Ne_WatchdogKillPath). The game's watchdog thread
// (Ne_Watchdog_ThreadLoop) wakes every second, reads the time since the frame tick last fed it (Ne_Watchdog_Tick)
// and kills the process when that exceeds WatchDog.delay (30000 ms). A front-end flow that sits in Entering for
// longer than that on a big modded profile stalls the frame tick for the whole load, and the game dies mid-load.
//
// Flows tracks the flows now in Entering. allows: the kill is held while a flow entered Entering no more than
// kEnteringMaxMs ago and the frame tick has been silent for at most kCapMs; past the cap the game's own kill
// runs. bucket: which kBucketMs-wide slice of the stall the silence is in, so the log speaks once per slice.

namespace runtime::watchdog_grace {

constexpr uint32_t kCapMs = 180000;
constexpr uint64_t kEnteringMaxMs = 600000;
constexpr uint32_t kBucketMs = 30000;
constexpr size_t kSlots = 16;
constexpr size_t kName = 48;

struct Entering {
    uint32_t hash = 0;
    uint64_t since = 0;
    char name[kName] = {};
    bool used = false;
};

class Flows {
public:
    void flow(uint32_t hash, uint32_t to, uint64_t now, const char *name)
    {
        for (auto &e : slots_)
            if (e.used && e.hash == hash) {
                e.used = false;
                break;
            }
        if (to != 1)
            return;
        Entering *put = nullptr;
        for (auto &e : slots_)
            if (!e.used) {
                put = &e;
                break;
            }
        if (!put) {
            put = &slots_[0];
            for (auto &e : slots_)
                if (e.since < put->since)
                    put = &e;
        }
        put->used = true;
        put->hash = hash;
        put->since = now;
        std::strncpy(put->name, name ? name : "", kName - 1);
        put->name[kName - 1] = 0;
    }

    const Entering *newest(uint64_t now) const
    {
        const Entering *best = nullptr;
        for (auto &e : slots_) {
            if (!e.used || now < e.since || now - e.since > kEnteringMaxMs)
                continue;
            if (!best || e.since > best->since)
                best = &e;
        }
        return best;
    }

    const Entering *allows(uint32_t elapsed_ms, uint64_t now) const
    {
        return elapsed_ms <= kCapMs ? newest(now) : nullptr;
    }

    size_t open(uint64_t now) const
    {
        size_t n = 0;
        for (auto &e : slots_)
            if (e.used && now >= e.since && now - e.since <= kEnteringMaxMs)
                n++;
        return n;
    }

private:
    Entering slots_[kSlots];
};

inline uint32_t bucket(uint32_t elapsed_ms)
{
    return elapsed_ms / kBucketMs;
}

}
