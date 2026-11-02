// SPDX-License-Identifier: AGPL-3.0-or-later
// Where the mod touches F1_25.exe. Every byte here was read from the Steam build
// sha256 d873a1c8d2044cef... (game build 25251167, PE timestamp 1789118616), the build the
// launcher's knowledge base (f1-multiplayer/re/db) describes. native/test/tests.cpp re-checks all
// of it against an F1_25.exe on disk.
#pragma once
#include <cstddef>
#include <cstdint>

namespace barriers::sites {

struct Check {
    size_t offset;
    const uint8_t *bytes;
    size_t size;
};

// ---- Game_Breakable_ParseActivation (VA 0x144588c90, .xtls) -------------------------------
// Not a core binding, so the mod finds it by this signature (unique in .xtls) and detours it.
inline constexpr const char *kBreakParsePattern =
    "41 56 56 57 53 48 83 EC 38 C5 F8 29 74 24 20 49 89 D6 48 89 CE E8 ?? ?? ?? ?? 89 C7 85 C0 0F 85 ?? ?? ?? ?? "
    "48 8D 9E E8 00 00 00";
inline constexpr uintptr_t kBreakParseRva = 0x4588c90;

// push r14; push rsi; push rdi; push rbx; sub rsp,0x38; vmovaps [rsp+0x20],xmm6 (15 bytes,
// no RIP-relative operand): the bytes the detour moves into its trampoline.
inline constexpr uint8_t kBreakParseProlog[15] = {0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x83, 0xEC,
                                                  0x38, 0xC5, 0xF8, 0x29, 0x74, 0x24, 0x20};

// The field stores the tuning relies on: lea rbx,[rsi+0xe8] / lea r8,[rsi+0xf0] /
// lea r8,[rsi+0x100] / mov r14,[rsi+0x28] (name) / vmovss [rsi+0xe8],xmm6 / vmovss [rsi+0x104],xmm0.
inline constexpr uint8_t kBp24[] = {0x48, 0x8D, 0x9E, 0xE8, 0x00, 0x00, 0x00};
inline constexpr uint8_t kBp3f[] = {0x4C, 0x8D, 0x86, 0xF0, 0x00, 0x00, 0x00};
inline constexpr uint8_t kBp5a[] = {0x4C, 0x8D, 0x86, 0x00, 0x01, 0x00, 0x00};
inline constexpr uint8_t kBp90[] = {0x4C, 0x8B, 0x76, 0x28};
inline constexpr uint8_t kBpD0[] = {0xC5, 0xFA, 0x11, 0xB6, 0xE8, 0x00, 0x00, 0x00};
inline constexpr uint8_t kBpE4[] = {0xC5, 0xFA, 0x11, 0x86, 0x04, 0x01, 0x00, 0x00};

inline constexpr Check kBreakParseChecks[] = {
    {0x00, kBreakParseProlog, sizeof kBreakParseProlog},
    {0x24, kBp24, sizeof kBp24},
    {0x3f, kBp3f, sizeof kBp3f},
    {0x5a, kBp5a, sizeof kBp5a},
    {0x90, kBp90, sizeof kBp90},
    {0xd0, kBpD0, sizeof kBpD0},
    {0xe4, kBpE4, sizeof kBpE4},
};

// ---- Ne_Phys_ParseMaterials (VA 0x1440bee90, core binding "Ne_Phys_ParseMaterials") -------
// The core resolves and hooks it. The mod also checks the code that finds the material array, so
// it never walks a table it does not understand:
//   +0x170 mov rax,[rsi+0x28]; mov rax,[rax+8]; mov rax,[rax+0x28]        (library pointer chain)
//   +0x17f imul r13,[rax+0x18]; add r13,[rax+8]; cmp [rax+0x10],r14d      (stride, data, count)
//   +0x195 cmp byte [rax+0x20],0                                           (allocated flag)
inline constexpr uintptr_t kMaterialsRva = 0x40bee90;
inline constexpr uint8_t kPm170[] = {0x48, 0x8B, 0x46, 0x28, 0x48, 0x8B, 0x40, 0x08, 0x48, 0x8B, 0x40, 0x28};
inline constexpr uint8_t kPm17f[] = {0x4C, 0x0F, 0xAF, 0x68, 0x18, 0x4C, 0x03, 0x68, 0x08, 0x44, 0x39, 0x70, 0x10};
inline constexpr uint8_t kPm195[] = {0x80, 0x78, 0x20, 0x00};

inline constexpr Check kMaterialsChecks[] = {
    {0x170, kPm170, sizeof kPm170},
    {0x17f, kPm17f, sizeof kPm17f},
    {0x195, kPm195, sizeof kPm195},
};

// ---- Ne_PhysMaterial_CombinePair (VA 0x1440e7350, .xtls) -----------------------------------
// record *(const Material *a, const Material *b, cache *c): mixes two materials with sqrt(a*b) into
// the one-entry cache c and returns c+0x10, laid out like a Material (soft K/S at +0x18..+0x28).
// Engine bug: when a.minSoftK, b.minSoftK, a.minSoftS and b.minSoftS are all <= 0 it jumps to
// 0x1440e761b and leaves the soft fields of the PREVIOUS pair in the cache. With every stock
// material at 0 that never matters; once tyre walls are soft, a later rigid pair (car on asphalt,
// car on concrete) would inherit their softness. The mod clears those fields for such pairs.
// Callers: the contact solver at 0x1440f73d9 and 0x1440f9be9.
inline constexpr const char *kCombinePattern =
    "56 57 53 48 83 EC 20 48 89 CF 48 39 D1 0F 84 ?? ?? ?? ?? 4C 89 C6 48 89 D3 49 8B 08 49 8B 40 08 48 39 F9";
inline constexpr uintptr_t kCombineRva = 0x40e7350;
// push rsi; push rdi; push rbx; sub rsp,0x20; mov rdi,rcx; cmp rcx,rdx; je 0x1440e7655 (rel32 at +15)
inline constexpr uint8_t kCombineProlog[19] = {0x56, 0x57, 0x53, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x89, 0xCF,
                                               0x48, 0x39, 0xD1, 0x0F, 0x84, 0xF2, 0x02, 0x00, 0x00};
inline constexpr size_t kCombineJccAt = 13; // the je, opcode 0F 84
// +0x133 vmovss xmm0,[rdi+0x18]; vxorps xmm2; vucomiss; vmovss xmm1,[rbx+0x18]  (the soft gate)
// +0x2cb mov dword [rsi+0x3c],0                                                   (skip target)
// +0x2fe add rsi,0x10                                                             (returns c+0x10)
inline constexpr uint8_t kCb133[] = {0xC5, 0xFA, 0x10, 0x47, 0x18, 0xC5, 0xE8, 0x57, 0xD2,
                                     0xC5, 0xF8, 0x2E, 0xC2, 0xC5, 0xFA, 0x10, 0x4B, 0x18};
inline constexpr uint8_t kCb2cb[] = {0xC7, 0x46, 0x3C, 0x00, 0x00, 0x00, 0x00};
inline constexpr uint8_t kCb2fe[] = {0x48, 0x83, 0xC6, 0x10};
inline constexpr Check kCombineChecks[] = {
    {0x000, kCombineProlog, sizeof kCombineProlog},
    {0x133, kCb133, sizeof kCb133},
    {0x2cb, kCb2cb, sizeof kCb2cb},
    {0x2fe, kCb2fe, sizeof kCb2fe},
};
inline constexpr size_t kCacheRecord = 0x10; // returned record = cache + 0x10
inline constexpr size_t kRecordSoft = 0x18;  // minK, minS, maxK, maxS, speedForMaxSoftness

// Material library, from the code above: lib = *(*(*(loader+0x28)+8)+0x28);
// entries at *(lib+8), count u32 at lib+0x10, stride u64 at lib+0x18, allocated u8 at lib+0x20.
inline constexpr size_t kLoaderToA = 0x28, kAToB = 0x08, kBToLib = 0x28;
inline constexpr size_t kLibData = 0x08, kLibCount = 0x10, kLibStride = 0x18, kLibAllocated = 0x20;

}
