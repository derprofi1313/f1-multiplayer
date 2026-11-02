// SPDX-License-Identifier: AGPL-3.0-or-later
// A minimal x64 entry detour for game functions that the core does not export as bindings.
// It only patches when the function starts with exactly the bytes we expect, so a game update or
// another hook makes it refuse instead of corrupting code.
#pragma once
#include <cstddef>
#include <cstdint>

namespace barriers {

struct Detour {
    uint8_t *target = nullptr;
    uint8_t *tramp = nullptr;
    size_t stolen = 0;
    uint8_t saved[32] = {};
    bool active = false;
};

// A rel32 branch inside the stolen bytes (jcc 0F 8x, jmp E9 or call E8): `at` is its offset in the
// stolen bytes, `oplen` the opcode length before the rel32 (2 for jcc, 1 for jmp/call). The
// trampoline sends it to the original target through an absolute jump.
struct Rel32 {
    size_t at;
    size_t oplen;
};

// Patches `target` with an absolute jump to `handler` and returns a trampoline in *orig that runs
// the original function. `expect` must be the first `stolen` bytes of the function, whole
// instructions; any RIP-relative branch among them must be listed in `rels`. `stolen` must be
// 14..32. On failure returns false, leaves the code untouched and sets *why.
bool detour_install(Detour &d, void *target, const uint8_t *expect, size_t stolen, void *handler, void **orig,
                    const char **why, const Rel32 *rels = nullptr, size_t rel_count = 0);

// Puts the original bytes back. The trampoline stays allocated, in case a thread is still in it.
bool detour_remove(Detour &d);

}
