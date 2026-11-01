#pragma once

#include "runtime.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <string>

// Shared by the per-career character overrides (characters.cpp) and the career wizard control
// (career_wizard.cpp): the calling module of an API call, and the loaded career's data group.
//
// The data group is read the way the driver-career save-slot predicate 0x1438fb390 reads it: for
// each championship of the career entity (CareerEntity +0x1040..+0x1048, 16-byte entries,
// championship at +8), the CareerSetupComponent from the championship's component list (+0x208,
// get at vtable slot +0x10 with k_Career_CareerSetupComponentDesc); its data group index is the
// component's virtual at +0x268, the group list (*(component +0x290)) +0x560..+0x568 (16-byte
// entries, group at +8) and the name *(group +0x28) +0x10. The first championship with such a
// component names the career. These offsets are not KB struct rows yet (see the docs).

namespace runtime::career_ui {

namespace mem = swofty::mem;

constexpr uintptr_t CHAMPIONSHIPS = 0x1040, CHAMPIONSHIPS_END = 0x1048, CHAMPIONSHIP_COMPONENTS = 0x208;
constexpr uintptr_t COMPONENT_GET_SLOT = 0x10, GROUP_INDEX_SLOT = 0x268, GROUP_SOURCE = 0x290, GROUP_LIST = 0x560,
                    GROUP_LABEL = 0x28, NAME_TEXT = 0x10;
constexpr size_t MAX_CHAMPIONSHIPS = 64, MAX_GROUPS = 256;

inline HMODULE caller(const void *ret)
{
    HMODULE m = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       reinterpret_cast<LPCWSTR>(ret), &m);
    return m;
}

inline int copy_out(const std::string &s, char *out, size_t cap)
{
    if (!out || !cap)
        return SWOFTY_ASSET_INVALID;
    out[0] = 0;
    if (s.empty())
        return SWOFTY_ASSET_LATE;
    if (s.size() + 1 > cap)
        return SWOFTY_ASSET_INVALID;
    memcpy(out, s.c_str(), s.size() + 1);
    return SWOFTY_ASSET_OK;
}

inline bool valid_name(const char *s)
{
    if (!s || !*s)
        return false;
    for (size_t i = 0; s[i]; i++)
        if (i >= 127 || (unsigned char)s[i] < 0x20)
            return false;
    return true;
}

inline std::string group_of_championship(uintptr_t champ, uintptr_t desc)
{
    using Get = void *(void *list, const void *desc, uint32_t search_parents);
    using Index = uint8_t(void *component);
    uintptr_t list = champ + CHAMPIONSHIP_COMPONENTS, vt = 0, get = 0;
    if (!mem::read(list, vt) || !mem::in_image(vt) || !mem::read(vt + COMPONENT_GET_SLOT, get) || !mem::in_image(get))
        return {};
    auto comp = reinterpret_cast<uintptr_t>(
        reinterpret_cast<Get *>(get)(reinterpret_cast<void *>(list), reinterpret_cast<const void *>(desc), 1));
    uintptr_t cvt = 0, fn = 0, src = 0, lb = 0, le = 0, group = 0, named = 0, text = 0;
    if (!mem::object_like(comp) || !mem::read(comp, cvt) || !mem::read(cvt + GROUP_INDEX_SLOT, fn) || !mem::in_image(fn) ||
        !mem::read(comp + GROUP_SOURCE, src) || !src || !mem::read(src + GROUP_LIST, lb) ||
        !mem::read(src + GROUP_LIST + 8, le) || !lb || le < lb || (le - lb) / 0x10 > MAX_GROUPS)
        return {};
    size_t index = reinterpret_cast<Index *>(fn)(reinterpret_cast<void *>(comp));
    if (index >= (le - lb) / 0x10 || !mem::read(lb + index * 0x10 + 8, group) || !group ||
        !mem::read(group + GROUP_LABEL, named) || !named || !mem::read(named + NAME_TEXT, text))
        return {};
    std::string out;
    mem::c_string(text, out, 128);
    return out;
}

inline std::string loaded_group()
{
    const SwoftyCareer *c = career_table();
    void *e = c && c->entity ? c->entity() : nullptr;
    uintptr_t desc = swofty::api->resolve("k_Career_CareerSetupComponentDesc"), b = 0, end = 0;
    if (!e || !desc || !mem::read(reinterpret_cast<uintptr_t>(e) + CHAMPIONSHIPS, b) ||
        !mem::read(reinterpret_cast<uintptr_t>(e) + CHAMPIONSHIPS_END, end) || !b || end < b ||
        (end - b) / 0x10 > MAX_CHAMPIONSHIPS)
        return {};
    for (; b < end; b += 0x10) {
        uintptr_t champ = 0;
        if (!mem::read(b + 8, champ) || !mem::object_like(champ))
            continue;
        std::string g = group_of_championship(champ, desc);
        if (!g.empty())
            return g;
    }
    return {};
}

}
