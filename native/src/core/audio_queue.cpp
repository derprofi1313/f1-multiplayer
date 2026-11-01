#include "audio_queue_logic.hpp"
#include "runtime.hpp"

#include <windows.h>

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <array>
#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <utility>
#include <vector>

// Audio job queue fix (re/db/subsystems/audio.md).
//
// Wwise asks the game for job workers through Snd_Request_JobWorker. The game gives each request to an idle one of
// its 12 audio worker threads, or queues it in g_Snd_JobWorkerOverflow at the first free of 24 slots, but the
// workers (Snd_Run_JobWorkerThread) take queued requests from slots 0-11 only. A request queued in slots 12-23 never
// runs: every worker spins on g_Snd_JobWorkerPending, the Wwise audio thread waits for that job forever holding the
// Wwise main lock, a game job blocks in AK::SoundEngine::SetRTPCValue, and the watchdog ends the game 30 s later.
//
// The wrap hands Wwise a thunk in place of each worker function (8 thunks, one per distinct function; a ninth
// function passes through). After every request a thunk runs, and after every Snd_Request_JobWorker call, the
// requests queued in slots 12-23 move to free slots 0-11 under g_Snd_JobWorkerLock (audio_queue_logic.hpp); the
// pending count stays as it is. A request lands in slot 12 or later only while slots 0-11 are all queued, so the
// worker that finishes one of those moves it.
//
// SWOFTY_AUDIO_QUEUE_FIX=0 turns it off.

namespace {

namespace aq = runtime::audio_queue;
namespace mem = swofty::mem;

constexpr const char *MIXIN = "swofty_core.dll (audio queue fix)";
constexpr const char *TARGET = "Snd_Request_JobWorker";
constexpr const char *QUEUE = "g_Snd_JobWorkerOverflow";
constexpr const char *LOCK = "g_Snd_JobWorkerLock";
constexpr size_t THUNKS = 8;
constexpr int LOG_FIRST = 16;

using Worker = void (*)(uint32_t job_type, uint32_t usec);
using Request = void(void *worker, uint32_t job_type, uint32_t count);

Request *orig_request;
uintptr_t queue_addr = 0, lock_addr = 0;
bool offered = false, settled = false, wanted = false;
aq::Workers<THUNKS> workers;
std::atomic<int> queue_state{0};
std::atomic<int> logged{0};
std::atomic<uint64_t> moved_total{0};
std::atomic<bool> first{true};

uint8_t *queue()
{
    int state = queue_state.load(std::memory_order_acquire);
    auto *t = reinterpret_cast<uint8_t *>(queue_addr);
    if (state == 0) {
        bool ok = t && mem::readable(queue_addr, aq::kTableBytes);
        queue_state.store(ok ? 1 : 2, std::memory_order_release);
        if (!ok)
            swofty::log("[audio] queue fix: the overflow queue at %p is not readable; requests pass through",
                        static_cast<void *>(t));
        return ok ? t : nullptr;
    }
    return state == 1 ? t : nullptr;
}

CRITICAL_SECTION *lock()
{
    uintptr_t cs = 0;
    return lock_addr && mem::read(lock_addr, cs) && cs && mem::readable(cs, sizeof(CRITICAL_SECTION))
               ? reinterpret_cast<CRITICAL_SECTION *>(cs)
               : nullptr;
}

void sweep(const char *where)
{
    uint8_t *t = queue();
    if (!t || !aq::stranded(t))
        return;
    CRITICAL_SECTION *cs = lock();
    if (!cs)
        return;
    EnterCriticalSection(cs);
    size_t moved = aq::move_stranded(t);
    bool left = aq::stranded(t);
    LeaveCriticalSection(cs);
    if (!moved)
        return;
    uint64_t total = moved_total.fetch_add(moved) + moved;
    int n = logged.fetch_add(1) + 1;
    if (n > LOG_FIRST)
        return;
    swofty::log("[audio] queue fix: moved %zu queued worker request(s) from slots 12-23 to slots 0-11 %s (%llu in "
                "all%s)%s",
                moved, where, static_cast<unsigned long long>(total), left ? "; more wait for a free slot" : "",
                n == LOG_FIRST ? " (further moves are not logged)" : "");
}

template <size_t I> void thunk(uint32_t job_type, uint32_t usec)
{
    reinterpret_cast<Worker>(workers.at(I))(job_type, usec);
    sweep("after a worker request ran");
}

template <size_t... I> constexpr std::array<Worker, sizeof...(I)> make_thunks(std::index_sequence<I...>)
{
    return {&thunk<I>...};
}

constexpr std::array<Worker, THUNKS> thunks = make_thunks(std::make_index_sequence<THUNKS>{});

bool is_thunk(void *fn)
{
    for (Worker w : thunks)
        if (reinterpret_cast<void *>(w) == fn)
            return true;
    return false;
}

void request(void *worker, uint32_t job_type, uint32_t count)
{
    if (!worker || is_thunk(worker) || !queue()) {
        orig_request(worker, job_type, count);
        return;
    }
    size_t i = workers.index(worker);
    void *pass = i < THUNKS ? reinterpret_cast<void *>(thunks[i]) : worker;
    if (first.exchange(false))
        swofty::log("[audio] queue fix: first worker request (worker %p, job type %u, %u worker(s)) runs through "
                    "thunk %zu",
                    worker, job_type, count, i);
    else if (i == THUNKS && logged.fetch_add(1) < LOG_FIRST)
        swofty::log("[audio] queue fix: no thunk left for worker %p; its requests pass through", worker);
    orig_request(pass, job_type, count);
    sweep("after a request was queued");
}

}

namespace runtime {

std::vector<CoreMixins> audio_queue_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || !code_ready())
        return out;
    offered = true;
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_AUDIO_QUEUE_FIX", v, sizeof v);
    if (n && n < sizeof v && (!strcmp(v, "0") || !strcmp(v, "off"))) {
        swofty::log("[audio] queue fix off (SWOFTY_AUDIO_QUEUE_FIX=%s)", v);
        return out;
    }
    queue_addr = swofty::api->resolve(QUEUE);
    lock_addr = swofty::api->resolve(LOCK);
    if (!queue_addr || !lock_addr || !swofty::api->resolve(TARGET)) {
        queue_addr = lock_addr = 0;
        swofty::log("[audio] queue fix off: needs %s, %s and %s, which this build does not resolve", TARGET, QUEUE,
                    LOCK);
        return out;
    }
    wanted = true;
    out.push_back({MIXIN, {{TARGET, SWOFTY_MIXIN_WRAP, 100, (void *)&request, (void **)&orig_request}}});
    return out;
}

void audio_queue_commit()
{
    if (!offered || settled || !wanted)
        return;
    settled = true;
    swofty::log("[audio] queue fix %s",
                core_mixins_active(MIXIN)
                    ? "armed: audio worker requests queued in slots 12-23 move to slots 0-11, where the workers take them"
                    : "did not install (see the [core] lines)");
}

}
