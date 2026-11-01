#pragma once

#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>

// The audio job worker overflow queue (re/db/globals.tsv g_Snd_JobWorkerOverflow): 24 slots of 0x30 bytes,
// +0x08 the worker function, +0x24 the job type, +0x28 the queued flag. Snd_Request_JobWorker queues a request
// at the first free slot; Snd_Run_JobWorkerThread takes queued requests from slots 0-11 only. stranded: a
// request waits in slots 12-23; move_stranded: each one moves, in order, into a free slot of 0-11 (the caller
// holds g_Snd_JobWorkerLock; the pending count stays as it is). Workers: the worker functions the request
// wrap has seen, one thunk each.

namespace runtime::audio_queue {

constexpr size_t kSlots = 24;
constexpr size_t kDrained = 12;
constexpr size_t kStride = 0x30;
constexpr size_t kFn = 0x08;
constexpr size_t kType = 0x24;
constexpr size_t kFlag = 0x28;
constexpr size_t kTableBytes = kSlots * kStride;
static_assert(kTableBytes == 1152);

inline uint8_t *slot(uint8_t *table, size_t i)
{
    return table + i * kStride;
}

inline bool stranded(const uint8_t *table)
{
    const volatile uint8_t *t = table;
    for (size_t i = kDrained; i < kSlots; i++)
        if (t[i * kStride + kFlag])
            return true;
    return false;
}

inline size_t move_stranded(uint8_t *table)
{
    size_t moved = 0, low = 0;
    for (size_t high = kDrained; high < kSlots; high++) {
        uint8_t *h = slot(table, high);
        if (!h[kFlag])
            continue;
        while (low < kDrained && slot(table, low)[kFlag])
            low++;
        if (low == kDrained)
            break;
        uint8_t *l = slot(table, low);
        std::memcpy(l + kFn, h + kFn, sizeof(void *));
        std::memcpy(l + kType, h + kType, sizeof(uint32_t));
        l[kFlag] = 1;
        h[kFlag] = 0;
        moved++;
    }
    return moved;
}

template <size_t N> struct Workers {
    std::atomic<void *> fns[N]{};

    size_t index(void *fn)
    {
        for (size_t i = 0; i < N; i++) {
            void *cur = fns[i].load(std::memory_order_acquire);
            if (cur == fn)
                return i;
            if (!cur) {
                void *expected = nullptr;
                if (fns[i].compare_exchange_strong(expected, fn, std::memory_order_acq_rel))
                    return i;
                if (expected == fn)
                    return i;
            }
        }
        return N;
    }

    void *at(size_t i) const { return fns[i].load(std::memory_order_acquire); }
};

}
