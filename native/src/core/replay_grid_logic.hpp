#pragma once
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <string>

namespace runtime::replay_grid {

constexpr uint32_t STREAM_POS = 0x98;
constexpr uint32_t STREAM_BASE = 0xa0;
constexpr uint32_t HOLDER_CAPACITY = 0x108;
constexpr size_t STOCK_CARS = 22;
constexpr int64_t STOCK_GRID_MAX = 24;

inline std::atomic<int64_t> session_vehicles{-1};

inline uint64_t capacity_of(uint64_t raw)
{
    return raw & 0xffffffffull;
}

inline bool guards(int64_t cars)
{
    return cars < 0 || cars > STOCK_GRID_MAX;
}

inline bool outgrew(uint64_t pos, uint64_t base, uint64_t capacity)
{
    return capacity != 0 && pos > base && pos - base > capacity;
}

struct Tally {
    std::atomic<uint64_t> skipped{0};

    bool first()
    {
        return skipped.fetch_add(1, std::memory_order_relaxed) == 0;
    }
};

inline std::string line(int64_t cars, const std::string &recordable, uint64_t state, uint64_t capacity)
{
    char buf[400];
    std::string name = recordable.empty() ? "?" : recordable;
    char count[24] = "?";
    if (cars >= 0)
        snprintf(count, sizeof count, "%lld", static_cast<long long>(cars));
    snprintf(buf, sizeof buf,
             "[replay] session has %s cars, the recorder holds %zu; %s needs %llu bytes of replay state and its holder has "
             "%llu, so it is not recorded (replay and flashback lose its state; the session keeps all its cars)",
             count, STOCK_CARS, name.c_str(), static_cast<unsigned long long>(state),
             static_cast<unsigned long long>(capacity));
    return buf;
}

}
