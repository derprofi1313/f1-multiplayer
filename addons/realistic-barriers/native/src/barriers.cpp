// SPDX-License-Identifier: AGPL-3.0-or-later
// Realistic Barrier Crashes for F1 25 (2025 and 2026 seasons), an addon for the
// F1 Multiplayer 2016-2026 launcher (Swofty addon format).
//
// Two engine layers:
//  1. Barrier materials. After the game parses its physics material library
//     (Ne_Phys_ParseMaterials, core binding), tyre walls and TecPro become spring-damper contacts
//     sized for an 800 kg car (the stock data has every material rigid) and hold the car more.
//     A detour on Ne_PhysMaterial_CombinePair fixes the engine's stale-cache bug so that softness
//     never leaks to rigid pairs (car on asphalt, car on concrete).
//  2. Barrier parts that break loose. After each breakable prop parses its activation values
//     (Game_Breakable_ParseActivation, own verified detour), tyre stacks, TecPro modules and boards
//     get a new break threshold, throw strength and break-off radius.
// Nothing changes on disk. Every hook checks the game's bytes first and stays off when they differ.

#include <swofty/mixin.hpp>

#include "detour.hpp"
#include "game_sites.hpp"
#include "tuning.hpp"

#include <windows.h>

#include <algorithm>
#include <atomic>
#include <cstdio>
#include <cstring>
#include <iterator>
#include <mutex>
#include <unordered_map>
#include <vector>

// The build scripts stamp both from swofty.json and the checkout, so the log names the exact build.
#ifndef RB_VERSION
#define RB_VERSION "dev"
#endif
#ifndef RB_BUILD
#define RB_BUILD "local build"
#endif
#define RB "[barriers]"

SWOFTY_SETTING(bool, tune_materials, true);
SWOFTY_SETTING(bool, tune_breakables, true);
SWOFTY_SETTING(int, absorption, 60);
SWOFTY_SETTING(int, crush_depth, 55);
SWOFTY_SETTING(int, tyre_grip, 115);
SWOFTY_SETTING(int, break_threshold, 100);
SWOFTY_SETTING(int, debris_scatter, 60);
SWOFTY_SETTING(int, break_spread, 140);
SWOFTY_SETTING(bool, log_details, false);

namespace {

using barriers::Material;
using barriers::Settings;

Settings current_settings()
{
    Settings s;
    s.materials = tune_materials;
    s.breakables = tune_breakables;
    s.absorption = absorption;
    s.crush = crush_depth;
    s.grip = tyre_grip;
    s.break_threshold = break_threshold;
    s.scatter = debris_scatter;
    s.spread = break_spread;
    return s;
}

// --- memory helpers ---------------------------------------------------------------------------

size_t readable_bytes(const void *p)
{
    if (!p)
        return 0;
    MEMORY_BASIC_INFORMATION mbi{};
    if (!VirtualQuery(p, &mbi, sizeof mbi) || mbi.State != MEM_COMMIT)
        return 0;
    const DWORD ok = PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE |
                     PAGE_EXECUTE_WRITECOPY;
    if (!(mbi.Protect & ok) || (mbi.Protect & (PAGE_GUARD | PAGE_NOACCESS)))
        return 0;
    const auto end = reinterpret_cast<uintptr_t>(mbi.BaseAddress) + mbi.RegionSize;
    return end - reinterpret_cast<uintptr_t>(p);
}

bool readable(const void *p, size_t n) { return n == 0 || readable_bytes(p) >= n; }

template <class T> bool read(uintptr_t at, T &out)
{
    if (!readable(reinterpret_cast<const void *>(at), sizeof(T)))
        return false;
    std::memcpy(&out, reinterpret_cast<const void *>(at), sizeof(T));
    return true;
}

bool bytes_match(uintptr_t fn, const barriers::sites::Check *checks, size_t n, const char **which)
{
    for (size_t i = 0; i < n; i++) {
        const auto *at = reinterpret_cast<const uint8_t *>(fn + checks[i].offset);
        if (!readable(at, checks[i].size) || std::memcmp(at, checks[i].bytes, checks[i].size) != 0) {
            static char buf[48]; // only used during init, on one thread
            snprintf(buf, sizeof buf, "+0x%zx", checks[i].offset);
            *which = buf;
            return false;
        }
    }
    return true;
}

// --- layer 1: materials -----------------------------------------------------------------------

using ParseMaterialsFn = void(void *loader, void *source);
ParseMaterialsFn *orig_parse_materials = nullptr;

struct Entry {
    Material *at;
    Material stock;   // as the game parsed it
    Material written; // what this mod last wrote there
};

std::mutex g_mat_lock;
std::vector<Entry> g_entries; // barrier materials of the library, in table order
std::atomic<int> g_mat_passes{0};
std::atomic<bool> g_combine_fixed{false};

void log_material(const char *what, const Material &m)
{
    char code[5];
    barriers::code_text(m.code, code);
    swofty::log(RB " %s %s: restitution %.3f friction %.3f minBounceVel %.2f soft K %.4g..%.4g S %.4g..%.4g "
                   "speedForMaxSoftness %.2f cfm %.4g correctionVel %.3f",
                what, code, m.restitution, m.friction, m.min_bounce_vel, m.min_soft_k, m.max_soft_k, m.min_soft_s,
                m.max_soft_s, m.speed_for_max_softness, m.cfm, m.correction_vel);
}

// Field by field, so a contact reading the entry meanwhile sees each float whole.
void store(Material *m, const Material &v)
{
    m->friction = v.friction;
    m->min_soft_k = v.min_soft_k;
    m->min_soft_s = v.min_soft_s;
    m->max_soft_k = v.max_soft_k;
    m->max_soft_s = v.max_soft_s;
    m->speed_for_max_softness = v.speed_for_max_softness;
}

// Writes the tuned (or, with the layer off, the stock) values of one entry. Caller holds the lock.
void apply_entry(Entry &e, const Settings &st, bool announce)
{
    Material v = e.stock;
    const bool tuned = st.materials && barriers::tune_material(v, st, g_combine_fixed.load());
    if (!tuned)
        v = e.stock;
    store(e.at, v);
    e.written = *e.at;
    if (!announce)
        return;
    char code[5];
    barriers::code_text(v.code, code);
    const auto surface = barriers::classify_material(v.code);
    if (!tuned) {
        swofty::log(RB " %s (%s): game values", code, barriers::surface_name(surface));
        return;
    }
    const float zeta = barriers::damping_ratio(st.absorption);
    if (v.max_soft_k > 0 && e.stock.max_soft_k == 0)
        swofty::log(RB " %s (%s): friction %.3f -> %.3f; soft contact K %.0f..%.0f N/m, S %.0f..%.0f N*s/m, softest "
                       "from %.0f m/s (damping ratio %.2f, rebound %.0f%%)",
                    code, barriers::surface_name(surface), e.stock.friction, v.friction, v.min_soft_k, v.max_soft_k,
                    v.min_soft_s, v.max_soft_s, v.speed_for_max_softness, zeta, 100.0f * barriers::rebound(zeta));
    else
        swofty::log(RB " %s (%s): friction %.3f -> %.3f; %s", code, barriers::surface_name(surface), e.stock.friction,
                    v.friction,
                    e.stock.max_soft_k > 0 ? "keeps the game's own soft contact"
                                           : "stays rigid (the CombinePair fix is not in place)");
}

void tune_library(void *loader)
{
    namespace s = barriers::sites;
    const auto base = reinterpret_cast<uintptr_t>(loader);
    uintptr_t a = 0, b = 0, lib = 0, data = 0;
    uint32_t count = 0;
    uint64_t stride = 0;
    uint8_t allocated = 0;
    if (!read(base + s::kLoaderToA, a) || !read(a + s::kAToB, b) || !read(b + s::kBToLib, lib) ||
        !read(lib + s::kLibData, data) || !read(lib + s::kLibCount, count) || !read(lib + s::kLibStride, stride) ||
        !read(lib + s::kLibAllocated, allocated)) {
        swofty::log(RB " materials: the material library is not readable; left as the game made it (SAFE)");
        return;
    }
    if (!allocated || !data || count == 0 || count > 4096 || stride < sizeof(Material) || stride > 0x1000 ||
        !readable(reinterpret_cast<void *>(data), size_t(count) * stride)) {
        swofty::log(RB " materials: library holds %u entries of %llu bytes (allocated %u); not a table this mod "
                       "knows, left untouched (SAFE)",
                    count, (unsigned long long)stride, unsigned(allocated));
        return;
    }

    const Settings st = current_settings();
    const bool detail = log_details;
    const int pass = ++g_mat_passes;
    int fresh = 0, kept = 0;
    std::lock_guard<std::mutex> g(g_mat_lock);
    std::vector<Entry> next;
    for (uint32_t i = 0; i < count; i++) {
        auto *m = reinterpret_cast<Material *>(data + size_t(i) * stride);
        const auto surface = barriers::classify_material(m->code);
        if (detail)
            log_material(surface == barriers::Surface::other ? "material" : "barrier material", *m);
        if (surface != barriers::Surface::tyre_wall && surface != barriers::Surface::tecpro)
            continue;
        auto old = std::find_if(g_entries.begin(), g_entries.end(), [&](const Entry &e) { return e.at == m; });
        Entry e{m, *m, *m};
        if (old != g_entries.end() && barriers::same_material(old->written, *m)) {
            e.stock = old->stock; // the parse ran without re-reading: the entry still holds our values
            kept++;
        } else {
            fresh++; // a real parse: these are the game's values
        }
        apply_entry(e, st, true);
        next.push_back(e);
    }
    g_entries = std::move(next);
    swofty::log(RB " materials pass %d: %u entries, %zu barrier materials (%d from the game, %d kept); absorption "
                   "%d, depth %d cm, grip %d%%, soft contacts %s",
                pass, count, g_entries.size(), fresh, kept, st.absorption, st.crush, st.grip,
                g_combine_fixed ? "on" : "OFF (CombinePair fix missing)");
}

// Settings changed in the launcher or in ADDON SETTINGS: the game parses its materials once per
// start, so the mod rewrites them live from the stock copy.
void retune_all(const char *why)
{
    std::lock_guard<std::mutex> g(g_mat_lock);
    if (g_entries.empty())
        return;
    const Settings st = current_settings();
    for (auto &e : g_entries)
        apply_entry(e, st, true);
    swofty::log(RB " materials re-applied (%s): absorption %d, depth %d cm, grip %d%%, layer %s", why, st.absorption,
                st.crush, st.grip, st.materials ? "on" : "off");
}

void parse_materials(void *loader, void *source)
{
    orig_parse_materials(loader, source);
    if (!tune_materials)
        swofty::log(RB " materials: off in the addon settings for now; turn it on and it applies at once");
    tune_library(loader); // records the stock values; writes the game's own values when the layer is off
}

// --- CombinePair fix ---------------------------------------------------------------------------

using CombineFn = void *(const Material *a, const Material *b, void *cache);
CombineFn *orig_combine = nullptr;
barriers::Detour g_combine_detour;
std::atomic<uint64_t> g_soft_pairs{0};
std::atomic<uint64_t> g_cleared_pairs{0};

bool has_soft(const Material *m) { return m->min_soft_k > 0 || m->min_soft_s > 0; }

void *combine_pair(const Material *a, const Material *b, void *cache)
{
    void *r = orig_combine(a, b, cache);
    // a == b returns the material itself; only the cache record is ours to correct.
    if (a == b || !r || r != static_cast<uint8_t *>(cache) + barriers::sites::kCacheRecord)
        return r;
    if (has_soft(a) || has_soft(b)) {
        const uint64_t n = ++g_soft_pairs;
        if (n == 1 || n == 1000 || n == 100000 || n % 10000000 == 0)
            swofty::log(RB " soft barrier contact pairs so far: %llu", (unsigned long long)n);
        return r;
    }
    // Same gate as the engine (0x1440e7483): neither side soft -> the pair must be rigid.
    auto *soft = reinterpret_cast<float *>(static_cast<uint8_t *>(r) + barriers::sites::kRecordSoft);
    if (soft[0] != 0 || soft[1] != 0 || soft[2] != 0 || soft[3] != 0 || soft[4] != 0) {
        soft[0] = soft[1] = soft[2] = soft[3] = soft[4] = 0;
        ++g_cleared_pairs;
    }
    return r;
}

bool install_combine_fix(const SwoftyApi *a)
{
    namespace s = barriers::sites;
    uintptr_t hit = 0;
    const int n = a->scan ? a->scan(".xtls", s::kCombinePattern, &hit) : 0;
    if (n != 1 || !hit) {
        swofty::log(RB " combine fix: Ne_PhysMaterial_CombinePair found %d time(s); soft barriers stay off (SAFE)", n);
        return false;
    }
    const char *which = nullptr;
    if (!bytes_match(hit, s::kCombineChecks, std::size(s::kCombineChecks), &which)) {
        swofty::log(RB " combine fix: the code at %s differs from the catalogued build; soft barriers stay off (SAFE)",
                    which);
        return false;
    }
    const barriers::Rel32 je{s::kCombineJccAt, 2};
    const char *why = nullptr;
    if (!barriers::detour_install(g_combine_detour, reinterpret_cast<void *>(hit), s::kCombineProlog,
                                  sizeof s::kCombineProlog, reinterpret_cast<void *>(&combine_pair),
                                  reinterpret_cast<void **>(&orig_combine), &why, &je, 1)) {
        swofty::log(RB " combine fix: not hooked: %s; soft barriers stay off (SAFE)", why ? why : "?");
        return false;
    }
    const uintptr_t base = a->module_base ? a->module_base() : 0;
    swofty::log(RB " combine fix: hooked Ne_PhysMaterial_CombinePair at base+0x%llx", (unsigned long long)(hit - base));
    return true;
}

// --- layer 2: breakable barrier parts ---------------------------------------------------------

using ParseActivationFn = int(void *component, void *parser);
ParseActivationFn *orig_parse_activation = nullptr;
barriers::Detour g_break_detour;

std::atomic<int> g_prop_count[4];
std::atomic<int> g_other_logged{0};

// Copies the instance name without reading past committed memory.
bool copy_name(const char *src, char *out, size_t cap)
{
    const size_t avail = readable_bytes(src);
    if (!avail)
        return false;
    const size_t n = strnlen(src, std::min(avail, cap - 1));
    std::memcpy(out, src, n);
    out[n] = 0;
    return true;
}

void tune_component(void *component)
{
    const auto c = reinterpret_cast<uintptr_t>(component);
    const char *name_ptr = nullptr;
    char name[160] = "?";
    if (!read(c + barriers::kBreakName, name_ptr) || !copy_name(name_ptr, name, sizeof name))
        return;
    const auto prop = barriers::classify_prop(name);
    const int n = ++g_prop_count[int(prop)];
    if (prop == barriers::Prop::other) {
        if (log_details && g_other_logged++ < 40)
            swofty::log(RB " breakable left as is: %s", name);
        return;
    }
    auto *impulse = reinterpret_cast<float *>(c + barriers::kBreakImpulse);
    auto *explosive = reinterpret_cast<float *>(c + barriers::kBreakExplosive);
    auto *falloff = reinterpret_cast<float *>(c + barriers::kBreakFalloff);
    auto *falloff_sq = reinterpret_cast<float *>(c + barriers::kBreakFalloffSq);
    if (!readable(impulse, barriers::kBreakFalloffSq - barriers::kBreakImpulse + 4))
        return;
    barriers::BreakValues v{*impulse, *explosive, *falloff};
    const barriers::BreakValues before = v;
    if (!barriers::tune_breakable(v, prop, current_settings())) {
        swofty::log(RB " breakable %s skipped: values out of range (impulse %g, explosiveness %g, falloff %g)", name,
                    before.impulse, before.explosiveness, before.falloff);
        return;
    }
    *impulse = v.impulse;
    *explosive = v.explosiveness;
    *falloff = v.falloff;
    *falloff_sq = v.falloff * v.falloff;
    if (n <= 6 || log_details || n % 250 == 0)
        swofty::log(RB " %s #%d %s: break impulse %.1f -> %.1f, explosiveness %.3f -> %.3f, falloff %.2f -> %.2f",
                    barriers::prop_name(prop), n, name, before.impulse, v.impulse, before.explosiveness,
                    v.explosiveness, before.falloff, v.falloff);
}

int parse_activation(void *component, void *parser)
{
    const int r = orig_parse_activation(component, parser);
    if (tune_breakables)
        tune_component(component);
    return r;
}

bool install_breakables(const SwoftyApi *a)
{
    namespace s = barriers::sites;
    uintptr_t hit = 0;
    const int n = a->scan ? a->scan(".xtls", s::kBreakParsePattern, &hit) : 0;
    if (n != 1 || !hit) {
        swofty::log(RB " breakables: Game_Breakable_ParseActivation found %d time(s) on this build; barrier parts "
                       "keep the game's values (SAFE)",
                    n);
        return false;
    }
    const uintptr_t base = a->module_base ? a->module_base() : 0;
    if (base && hit - base != s::kBreakParseRva)
        swofty::log(RB " breakables: found at base+0x%llx (the catalogued build has 0x%llx); checking its bytes",
                    (unsigned long long)(hit - base), (unsigned long long)s::kBreakParseRva);
    const char *which = nullptr;
    if (!bytes_match(hit, s::kBreakParseChecks, std::size(s::kBreakParseChecks), &which)) {
        swofty::log(RB " breakables: the code at %s differs from the catalogued build; not hooked (SAFE)", which);
        return false;
    }
    const char *why = nullptr;
    if (!barriers::detour_install(g_break_detour, reinterpret_cast<void *>(hit), s::kBreakParseProlog,
                                  sizeof s::kBreakParseProlog, reinterpret_cast<void *>(&parse_activation),
                                  reinterpret_cast<void **>(&orig_parse_activation), &why)) {
        swofty::log(RB " breakables: not hooked: %s (SAFE)", why ? why : "?");
        return false;
    }
    swofty::log(RB " breakables: hooked Game_Breakable_ParseActivation at base+0x%llx",
                (unsigned long long)(hit - base));
    return true;
}

bool add_materials(const SwoftyApi *a, std::vector<SwoftyMixin> &list)
{
    namespace s = barriers::sites;
    const uintptr_t fn = a->resolve("Ne_Phys_ParseMaterials");
    if (!fn) {
        swofty::log(RB " materials: the core does not resolve Ne_Phys_ParseMaterials on this build; barrier "
                       "materials keep the game's values (SAFE)");
        return false;
    }
    const char *which = nullptr;
    if (!bytes_match(fn, s::kMaterialsChecks, std::size(s::kMaterialsChecks), &which)) {
        swofty::log(RB " materials: the material library code at %s differs from the catalogued build; not hooked "
                       "(SAFE)",
                    which);
        return false;
    }
    list.push_back({"Ne_Phys_ParseMaterials", SWOFTY_MIXIN_WRAP, 0, reinterpret_cast<void *>(&parse_materials),
                    reinterpret_cast<void **>(&orig_parse_materials)});
    return true;
}

}

SWOFTY_ON_CHANGE(tune_materials, bool) { retune_all("layer switched"); }
SWOFTY_ON_CHANGE(absorption, int) { retune_all("absorption"); }
SWOFTY_ON_CHANGE(crush_depth, int) { retune_all("depth"); }
SWOFTY_ON_CHANGE(tyre_grip, int) { retune_all("grip"); }

SWOFTY_ON_CHANGE(break_spread, int v)
{
    swofty::log(RB " break spread now %d%%; applies to barriers loaded from the next track on", v);
}

extern "C" __declspec(dllexport) const uint32_t SwoftyAddonApi = SWOFTY_API_VERSION;

extern "C" __declspec(dllexport) int SwoftyAddonInit(const SwoftyApi *a)
{
    if (!a || a->version < 3 || !a->resolve || !a->register_mixins || !a->log) {
        if (a && a->log)
            a->log(RB " needs the swofty_core mixin and settings API (v3), loader offers v%u; nothing installed "
                      "(SAFE)",
                   a->version);
        return 0;
    }
    swofty::api = a;
    swofty::log(RB " Realistic Barrier Crashes " RB_VERSION " (" RB_BUILD ") for F1 25 (2025 + 2026 seasons); "
                   "offline and LAN sessions where every player runs it");

    // The fix first: soft barrier contacts are only written while it is in place.
    g_combine_fixed = install_combine_fix(a);
    std::vector<SwoftyMixin> list;
    const bool materials = add_materials(a, list);
    bool mixins_ok = true;
    if (!list.empty() && !a->register_mixins(list.data(), list.size())) {
        swofty::log(RB " materials: register_mixins refused the hook; not installed (SAFE)");
        mixins_ok = false;
    }
    swofty::detail::init_settings(a);
    const bool breakables = install_breakables(a);

    const bool any = (materials && mixins_ok) || breakables || g_combine_fixed;
    swofty::log(RB " ready: materials %s, soft contacts %s, breakables %s (settings: absorption %d, depth %d cm, grip "
                   "%d%%, break threshold %d%%, scatter %d%%, spread %d%%)",
                materials && mixins_ok ? "hooked" : "off", g_combine_fixed ? "on" : "off",
                breakables ? "hooked" : "off", int(absorption),
                int(crush_depth), int(tyre_grip), int(break_threshold), int(debris_scatter), int(break_spread));
    if (!any) {
        swofty::log(RB " nothing could be installed on this build; the addon does nothing (SAFE)");
        return 0;
    }
    return 1;
}