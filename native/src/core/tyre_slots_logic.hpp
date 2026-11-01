#pragma once
#include <cstddef>
#include <cstdint>

// The per-car tyre slot table (tyre component +0x9bd0, at most 5 slots 0x18 apart: u32 option, +8 physical
// compound, +0x10 visual compound; count +0x9c48), as the core's tyre slot sites see it right after the game
// rebuilt it. The game looks a fitted set's option up in it and, when that option has no slot, the fallback
// (option >= 3 ? 4 : 1). Car_Tyre_SetWearFactor then reads the slot's physical compound and
// Sess_Tyre_ShowFittedSet its visual compound without a null check, and the latter's fallback scan has no bound.
// plan() keeps a table whose pick is usable, fills a pick with a null compound from a donor slot, adds a slot for
// the option from a donor when there is room, and skips the refit when no slot can serve.

namespace runtime::tyre_slots {

constexpr size_t CAPACITY = 5;

struct Slot {
    uint32_t option = 0;
    uint64_t physical = 0;
    uint64_t visual = 0;
};

enum class Kind { keep, fill, add, skip };

struct Plan {
    Kind kind = Kind::skip;
    int index = -1;
    int donor = -1;
};

inline uint32_t fallback(uint32_t option)
{
    return option >= 3 ? 4u : 1u;
}

inline bool usable(const Slot &s)
{
    return s.physical && s.visual;
}

inline int find(const Slot *slots, uint64_t count, uint32_t option)
{
    if (count > CAPACITY)
        return -1;
    for (uint64_t i = 0; i < count; i++)
        if (slots[i].option == option)
            return int(i);
    uint32_t alt = fallback(option);
    for (uint64_t i = 0; i < count; i++)
        if (slots[i].option == alt)
            return int(i);
    return -1;
}

inline int donor(const Slot *slots, uint64_t count, uint32_t option)
{
    if (count > CAPACITY)
        return -1;
    bool wet = option >= 3;
    int any = -1;
    for (uint64_t i = 0; i < count; i++) {
        if (!usable(slots[i]))
            continue;
        if ((slots[i].option >= 3) == wet)
            return int(i);
        if (any < 0)
            any = int(i);
    }
    return any;
}

inline Plan plan(const Slot *slots, uint64_t count, uint32_t option)
{
    if (count > CAPACITY)
        return {};
    int pick = find(slots, count, option);
    if (pick >= 0 && usable(slots[pick]))
        return {Kind::keep, pick, -1};
    int from = donor(slots, count, option);
    if (from < 0)
        return {};
    if (pick >= 0)
        return {Kind::fill, pick, from};
    if (count < CAPACITY)
        return {Kind::add, int(count), from};
    return {};
}

}
