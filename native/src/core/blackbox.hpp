#pragma once
#include <atomic>
#include <cstdint>
#include <cstring>

// Always-on flight recorder for the crash report: the last events (flow states, screens, sessions,
// heap errors, first-chance exceptions, exits) and the last game-log warnings, each in a fixed ring.
// A note claims a slot with one atomic increment and copies at most TEXT bytes into it; nothing locks
// or allocates. A slot's sequence number is cleared while it is written, so a reader skips a slot that
// a writer is still filling or that was reused while it was being read.

namespace blackbox {

constexpr unsigned TEXT = 176, KIND = 12;

struct Entry {
    std::atomic<uint64_t> seq{0};
    uint64_t ms = 0;
    uint32_t tid = 0;
    char kind[KIND] = {};
    char text[TEXT] = {};
};

struct Copy {
    uint64_t seq, ms;
    uint32_t tid;
    char kind[KIND];
    char text[TEXT];
};

inline size_t copy_into(char *out, size_t at, size_t cap, const char *s)
{
    if (!s)
        return at;
    while (at + 1 < cap && *s)
        out[at++] = *s++;
    out[at] = 0;
    return at;
}

template <unsigned N> struct Ring {
    static_assert(N && !(N & (N - 1)), "ring size is a power of two");
    Entry slots[N];
    std::atomic<uint64_t> next{0};

    void put(uint64_t ms, uint32_t tid, const char *kind, const char *a, const char *b = nullptr)
    {
        uint64_t n = next.fetch_add(1, std::memory_order_relaxed);
        Entry &e = slots[n & (N - 1)];
        e.seq.store(0, std::memory_order_relaxed);
        std::atomic_thread_fence(std::memory_order_release);
        e.ms = ms;
        e.tid = tid;
        copy_into(e.kind, 0, KIND, kind);
        copy_into(e.text, copy_into(e.text, 0, TEXT, a), TEXT, b);
        e.seq.store(n + 1, std::memory_order_release);
    }

    uint64_t total() const { return next.load(std::memory_order_acquire); }

    template <class F> unsigned each(F f) const
    {
        uint64_t end = next.load(std::memory_order_acquire), from = end > N ? end - N : 0;
        unsigned shown = 0;
        for (uint64_t n = from; n < end; n++) {
            const Entry &e = slots[n & (N - 1)];
            Copy c;
            c.seq = e.seq.load(std::memory_order_acquire);
            if (c.seq != n + 1)
                continue;
            c.ms = e.ms;
            c.tid = e.tid;
            memcpy(c.kind, e.kind, KIND);
            memcpy(c.text, e.text, TEXT);
            c.kind[KIND - 1] = c.text[TEXT - 1] = 0;
            std::atomic_thread_fence(std::memory_order_acquire);
            if (e.seq.load(std::memory_order_relaxed) != c.seq)
                continue;
            f(c);
            shown++;
        }
        return shown;
    }

    bool last(const char *kind, Copy &out) const
    {
        bool found = false;
        each([&](const Copy &c) {
            if (!strcmp(c.kind, kind))
                out = c, found = true;
        });
        return found;
    }
};

void note(const char *kind, const char *a, const char *b = nullptr);

}
