// SPDX-License-Identifier: AGPL-3.0-or-later
#include "detour.hpp"

#include <cstring>
#include <windows.h>

namespace barriers {

namespace {

constexpr size_t kJmp = 14;         // FF 25 00 00 00 00 <abs64>
constexpr size_t kTrampSize = 256;  // stolen bytes + jump back + one absolute jump per relocated branch
constexpr size_t kMaxRels = 4;

void write_jmp(uint8_t *at, const void *to)
{
    at[0] = 0xFF;
    at[1] = 0x25;
    std::memset(at + 2, 0, 4);
    const uint64_t a = reinterpret_cast<uint64_t>(to);
    std::memcpy(at + 6, &a, 8);
}

bool write_code(uint8_t *at, const uint8_t *bytes, size_t n)
{
    DWORD old = 0;
    if (!VirtualProtect(at, n, PAGE_EXECUTE_READWRITE, &old))
        return false;
    std::memcpy(at, bytes, n);
    DWORD ignored = 0;
    VirtualProtect(at, n, old, &ignored);
    FlushInstructionCache(GetCurrentProcess(), at, n);
    return true;
}

}

bool detour_install(Detour &d, void *target, const uint8_t *expect, size_t stolen, void *handler, void **orig,
                    const char **why, const Rel32 *rels, size_t rel_count)
{
    const char *dummy = nullptr;
    if (!why)
        why = &dummy;
    if (d.active) {
        *why = "already installed";
        return false;
    }
    if (!target || !expect || !handler || !orig || stolen < kJmp || stolen > sizeof d.saved || rel_count > kMaxRels ||
        (rel_count && !rels)) {
        *why = "bad arguments";
        return false;
    }
    for (size_t i = 0; i < rel_count; i++)
        if (rels[i].oplen < 1 || rels[i].oplen > 2 || rels[i].at + rels[i].oplen + 4 > stolen) {
            *why = "a relocated branch lies outside the stolen bytes";
            return false;
        }
    auto *t = static_cast<uint8_t *>(target);
    MEMORY_BASIC_INFORMATION mbi{};
    if (!VirtualQuery(t, &mbi, sizeof mbi) || mbi.State != MEM_COMMIT ||
        !(mbi.Protect & (PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))) {
        *why = "target is not committed executable memory";
        return false;
    }
    if (std::memcmp(t, expect, stolen) != 0) {
        *why = "the function does not start with the expected bytes (other game build, or already hooked)";
        return false;
    }
    auto *tramp = static_cast<uint8_t *>(VirtualAlloc(nullptr, kTrampSize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
    if (!tramp) {
        *why = "VirtualAlloc failed";
        return false;
    }
    std::memcpy(tramp, t, stolen);
    write_jmp(tramp + stolen, t + stolen);
    // Each rel32 branch now points at an absolute jump to its original destination.
    uint8_t *stub = tramp + stolen + kJmp;
    for (size_t i = 0; i < rel_count; i++, stub += kJmp) {
        const size_t rel_at = rels[i].at + rels[i].oplen;
        int32_t rel = 0;
        std::memcpy(&rel, t + rel_at, 4);
        const uint8_t *dest = t + rel_at + 4 + rel;
        write_jmp(stub, dest);
        const int64_t nrel = stub - (tramp + rel_at + 4);
        const int32_t nrel32 = int32_t(nrel);
        std::memcpy(tramp + rel_at, &nrel32, 4);
    }
    DWORD old = 0;
    if (!VirtualProtect(tramp, kTrampSize, PAGE_EXECUTE_READ, &old)) {
        VirtualFree(tramp, 0, MEM_RELEASE);
        *why = "VirtualProtect on the trampoline failed";
        return false;
    }
    FlushInstructionCache(GetCurrentProcess(), tramp, kTrampSize);

    uint8_t patch[32];
    std::memset(patch, 0x90, sizeof patch); // NOP pad: never executed, the jump leaves first
    write_jmp(patch, handler);
    std::memcpy(d.saved, t, stolen);
    *orig = tramp; // set before the jump goes live: the handler calls through it
    if (!write_code(t, patch, stolen)) {
        *orig = nullptr;
        VirtualFree(tramp, 0, MEM_RELEASE);
        *why = "VirtualProtect on the game code failed";
        return false;
    }
    d.target = t;
    d.tramp = tramp;
    d.stolen = stolen;
    d.active = true;
    *why = nullptr;
    return true;
}

bool detour_remove(Detour &d)
{
    if (!d.active)
        return false;
    if (!write_code(d.target, d.saved, d.stolen))
        return false;
    d.active = false;
    return true;
}

}
