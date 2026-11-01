#pragma once

#include <cstddef>
#include <cstdint>

namespace runtime::garage_setup {

constexpr uint64_t USER_BASE = 0x10000;
constexpr uint64_t USER_END = 0x800000000000ull;
constexpr uint64_t NODE_AT = 0x38;
constexpr uint64_t INFO_AT = 0x260;
constexpr uint64_t ID_AT = 0x2a8;
constexpr int LOG_FIRST = 4;

inline bool user_pointer(uint64_t p)
{
    return p >= USER_BASE && p < USER_END;
}

using Read = bool (*)(uint64_t address, void *out, size_t n, void *user);

inline bool entry_usable(uint64_t entry, Read read, void *user)
{
    if (!user_pointer(entry))
        return false;
    uint64_t node = 0;
    if (!read(entry + NODE_AT, &node, sizeof node, user) || !user_pointer(node))
        return false;
    uint64_t info = 0;
    if (!read(node + INFO_AT, &info, sizeof info, user) || !user_pointer(info))
        return false;
    uint32_t id = 0;
    return read(info + ID_AT, &id, sizeof id, user);
}

struct Skips {
    int count = 0;

    int note()
    {
        return ++count;
    }

    bool logged(int n) const
    {
        return n <= LOG_FIRST;
    }

    bool last(int n) const
    {
        return n == LOG_FIRST;
    }
};

}
