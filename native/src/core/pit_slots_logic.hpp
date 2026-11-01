#pragma once

#include <cstddef>
#include <cstdint>

namespace runtime::pit_slots {

constexpr int LOG_FIRST = 8;
constexpr size_t KEYS = 64;
constexpr uintptr_t LOWEST_POINTER = 0x10000;

enum class Site : uint8_t { eval_own, eval_list, rank_own, rank_list };

enum class Log : uint8_t { none, line, last };

inline bool usable(uintptr_t slot)
{
    return slot >= LOWEST_POINTER;
}

struct Seen {
    uint16_t keys[KEYS] = {};
    size_t count = 0;
    int logged = 0;

    Log note(Site site, uint8_t vehicle)
    {
        uint16_t key = uint16_t(uint16_t(site) << 8 | vehicle);
        for (size_t i = 0; i < count; i++)
            if (keys[i] == key)
                return Log::none;
        if (count < KEYS)
            keys[count++] = key;
        if (logged >= LOG_FIRST)
            return Log::none;
        return ++logged == LOG_FIRST ? Log::last : Log::line;
    }
};

}
