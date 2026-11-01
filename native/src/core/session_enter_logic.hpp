#pragma once

#include <cstddef>
#include <cstdint>

namespace runtime::session_enter {

constexpr uint64_t GROUPS_BEGIN_AT = 0x108;
constexpr uint64_t GROUPS_END_AT = 0x110;
constexpr uint64_t GROUP_MAP_AT = 0x8;
constexpr uint64_t NODE_LEFT = 0x0;
constexpr uint64_t NODE_PARENT = 0x8;
constexpr uint64_t NODE_RIGHT = 0x10;
constexpr uint64_t NODE_ISNIL = 0x19;
constexpr uint64_t NODE_KEY = 0x20;
constexpr uint64_t INDEX_AT = 0x60;
constexpr uint64_t USER_BASE = 0x10000;
constexpr uint64_t USER_END = 0x800000000000ull;
constexpr uint32_t NO_INDEX = 0xffffffffu;
constexpr int MAX_DEPTH = 96;
constexpr int LOG_FIRST = 4;

using Read = bool (*)(uint64_t address, void *out, size_t n, void *user);

struct Fix {
    bool replace = false;
    uint64_t key = 0;
    uint32_t index = NO_INDEX;
};

inline bool user_pointer(uint64_t p)
{
    return p >= USER_BASE && p < USER_END;
}

inline bool word(Read read, void *user, uint64_t address, uint64_t *out)
{
    return read(address, out, sizeof *out, user);
}

inline bool nil(Read read, void *user, uint64_t node, bool *out)
{
    uint8_t b = 0;
    if (!user_pointer(node) || !read(node + NODE_ISNIL, &b, 1, user))
        return false;
    *out = b != 0;
    return true;
}

inline Fix decide(uint64_t manager, uint64_t rsp, uint64_t hash, Read read, void *user)
{
    Fix f;
    uint64_t begin = 0, end = 0, group = 0, head = 0, node = 0;
    if (!user_pointer(manager) || !word(read, user, manager + GROUPS_BEGIN_AT, &begin) ||
        !word(read, user, manager + GROUPS_END_AT, &end) || begin == end || !word(read, user, begin, &group) ||
        !user_pointer(group) || !word(read, user, group + GROUP_MAP_AT, &head) || !user_pointer(head) ||
        !word(read, user, head + NODE_PARENT, &node))
        return f;
    uint64_t best = head;
    for (int depth = 0;; depth++) {
        bool isnil = false;
        if (depth >= MAX_DEPTH || !nil(read, user, node, &isnil))
            return f;
        if (isnil)
            break;
        uint64_t key = 0, next = 0;
        if (!word(read, user, node + NODE_KEY, &key))
            return f;
        if (key >= hash) {
            best = node;
            if (!word(read, user, node + NODE_LEFT, &next))
                return f;
        } else if (!word(read, user, node + NODE_RIGHT, &next)) {
            return f;
        }
        node = next;
    }
    if (best != head) {
        uint64_t key = 0;
        if (!word(read, user, best + NODE_KEY, &key))
            return f;
        if (key == hash)
            return f;
    }
    uint64_t first = 0, first_key = 0;
    bool isnil = false;
    if (!word(read, user, head + NODE_LEFT, &first) || first == head || !nil(read, user, first, &isnil) || isnil ||
        !word(read, user, first + NODE_KEY, &first_key))
        return f;
    uint64_t index = 0;
    if (word(read, user, rsp + INDEX_AT, &index))
        f.index = static_cast<uint32_t>(index & 0xff);
    f.replace = true;
    f.key = first_key;
    return f;
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
