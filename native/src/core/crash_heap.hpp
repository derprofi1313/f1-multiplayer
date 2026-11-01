#pragma once
#include <cstdint>
#include <cstring>

// The engine's NeMemoryHeap as Ne_Heap_DeallocateNonSmall (ne_memory_heap.cpp) reads it. The heap
// object holds its name at +0x58, its range at +0x98 (start) and +0xa0 (size), the allocated list head
// at +0x100, the guard size at +0x108 and the allocated byte count at +0x110. An allocated block is a
// 16-byte header, a start guard, the user data and an end guard: the header holds the signed offsets
// (in 16-byte units) to the previous and next block of the allocated list, the block size (header,
// guards and user data rounded up to 16) and the user size. The guards hold the strings below. Every
// read goes through a reader, so a bad pointer only ends the decode.

namespace neheap {

constexpr const char START_GUARD[] = "[BLOCKSTART] Bound checked mem.";
constexpr const char END_GUARD[] = "[BLOCKEND] Bound checked mem.  ";
constexpr size_t GUARD_TEXT = 32;

struct Heap {
    uintptr_t obj = 0, vtable = 0, start = 0, size = 0, guard = 0, list = 0, allocated = 0, name_ptr = 0;
    char name[64] = {};
    bool contains(uintptr_t a) const { return size && a >= start && a < start + size; }
};

struct Block {
    uintptr_t header = 0, user = 0;
    int32_t prev_rel = 0, next_rel = 0;
    uint32_t block_size = 0, user_size = 0;
    int start_guard = -1, end_guard = -1;
    uintptr_t end() const { return header + block_size; }
    uintptr_t list_prev() const { return prev_rel ? header + intptr_t(prev_rel) * 16 : 0; }
    uintptr_t list_next() const { return next_rel ? header + intptr_t(next_rel) * 16 : 0; }
};

template <class R, class T> bool get(R &rd, uintptr_t a, T &out)
{
    return rd.read(a, &out, sizeof out);
}

template <class R> bool decode(R &rd, uintptr_t obj, Heap &h)
{
    uint64_t f[0x118 / 8];
    if (!rd.read(obj, f, sizeof f))
        return false;
    h = {};
    h.obj = obj;
    h.vtable = f[0];
    h.name_ptr = f[0x58 / 8];
    h.start = f[0x98 / 8];
    h.size = f[0xa0 / 8];
    h.list = f[0x100 / 8];
    h.guard = f[0x108 / 8];
    h.allocated = f[0x110 / 8];
    if (!h.size || h.start + h.size < h.start || h.guard > 0x1000 || (h.guard & 0xf))
        return false;
    if (h.name_ptr && rd.read(h.name_ptr, h.name, sizeof h.name - 1)) {
        h.name[sizeof h.name - 1] = 0;
        for (char *p = h.name; *p; p++)
            if ((unsigned char)*p < 0x20 || (unsigned char)*p > 0x7e) {
                *p = 0;
                break;
            }
    } else {
        h.name[0] = 0;
    }
    return true;
}

inline uint32_t block_size_for(uint32_t user_size, uintptr_t guard)
{
    return uint32_t(((uint64_t(user_size) + 15) & ~uint64_t(15)) + 2 * guard + 0x10);
}

template <class R> int guard_state(R &rd, uintptr_t at, uintptr_t guard, const char *text)
{
    if (!guard)
        return -1;
    char buf[GUARD_TEXT];
    size_t n = guard < GUARD_TEXT ? size_t(guard) : GUARD_TEXT;
    if (!rd.read(at, buf, n))
        return -1;
    return memcmp(buf, text, n) == 0 ? 1 : 0;
}

template <class R> bool block_at(R &rd, const Heap &h, uintptr_t header, Block &b)
{
    if ((header & 0xf) != (h.start & 0xf) || !h.contains(header))
        return false;
    uint8_t raw[16];
    if (!rd.read(header, raw, sizeof raw))
        return false;
    b = {};
    b.header = header;
    memcpy(&b.prev_rel, raw, 4);
    memcpy(&b.next_rel, raw + 4, 4);
    memcpy(&b.block_size, raw + 8, 4);
    memcpy(&b.user_size, raw + 12, 4);
    if (b.block_size != block_size_for(b.user_size, h.guard) || b.end() > h.start + h.size)
        return false;
    b.user = header + 0x10 + h.guard;
    b.start_guard = guard_state(rd, header + 0x10, h.guard, START_GUARD);
    b.end_guard = guard_state(rd, b.user + b.user_size, h.guard, END_GUARD);
    return true;
}

template <class R> bool raw_header(R &rd, const Heap &h, uintptr_t header, Block &b)
{
    uint8_t raw[16];
    if (!rd.read(header, raw, sizeof raw))
        return false;
    b = {};
    b.header = header;
    b.user = header + 0x10 + h.guard;
    memcpy(&b.prev_rel, raw, 4);
    memcpy(&b.next_rel, raw + 4, 4);
    memcpy(&b.block_size, raw + 8, 4);
    memcpy(&b.user_size, raw + 12, 4);
    return true;
}

template <class R> bool containing(R &rd, const Heap &h, uintptr_t addr, Block &b, size_t max_back)
{
    if (!h.contains(addr))
        return false;
    uintptr_t at = addr - ((addr - h.start) & 0xf);
    for (size_t back = 0; back <= max_back && at >= h.start; back += 16, at -= 16) {
        Block c;
        if (block_at(rd, h, at, c) && addr < c.end()) {
            b = c;
            return true;
        }
        if (at < h.start + 16)
            break;
    }
    return false;
}

template <class R> bool before(R &rd, const Heap &h, uintptr_t header, Block &b, size_t max_back)
{
    if (header < h.start + 16)
        return false;
    for (uintptr_t at = header - 16, back = 16; back <= max_back && at >= h.start; at -= 16, back += 16) {
        Block c;
        if (block_at(rd, h, at, c) && c.end() == header) {
            b = c;
            return true;
        }
        if (at < h.start + 16)
            break;
    }
    return false;
}

}
