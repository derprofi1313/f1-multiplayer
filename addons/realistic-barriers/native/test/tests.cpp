// SPDX-License-Identifier: AGPL-3.0-or-later
// Host tests for Realistic Barrier Crashes. Runs on Windows without the game:
//   1. the tuning rules (tuning.hpp),
//   2. the whole plugin (barriers.cpp) against a fake core and fake game memory: the material
//      library walk through the core mixin, and the breakable detour executing a fake game function
//      that starts with the real F1 25 prologue bytes,
//   3. optionally (argv[1] = path to F1_25.exe): every signature and byte check against the real exe.
#include "../src/barriers.cpp"
#include "../src/detour.cpp"

#include <cstdarg>
#include <cstdio>
#include <fstream>
#include <map>
#include <string>

static int g_fail = 0, g_pass = 0;
#define CHECK(c)                                                                                                       \
    do {                                                                                                               \
        if (c)                                                                                                         \
            g_pass++;                                                                                                  \
        else {                                                                                                         \
            g_fail++;                                                                                                  \
            std::printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c);                                                   \
        }                                                                                                              \
    } while (0)
static bool approx(float a, float b, float eps = 1e-4f) { return std::fabs(a - b) <= eps * std::max(1.0f, std::fabs(b)); }

// ---------------------------------------------------------------------------------------------
// 1. Rules
// ---------------------------------------------------------------------------------------------
static void test_rules()
{
    using namespace barriers;
    CHECK(classify_material(fourcc("TYRW")) == Surface::tyre_wall);
    CHECK(classify_material(fourcc("DTYR")) == Surface::tyre_wall);
    CHECK(classify_material(fourcc("DRUB")) == Surface::tecpro);
    CHECK(classify_material(fourcc("RUBB")) == Surface::tecpro);
    CHECK(classify_material(fourcc("SIGN")) == Surface::board);
    CHECK(classify_material(fourcc("PLAS")) == Surface::board);
    CHECK(classify_material(fourcc("WALL")) == Surface::hard);
    CHECK(classify_material(fourcc("METL")) == Surface::hard);
    CHECK(classify_material(fourcc("SAST")) == Surface::other);
    CHECK(classify_material(fourcc("CARW")) == Surface::other); // car bodywork is never touched
    CHECK(classify_material(fourcc("ASTR")) == Surface::other); // track surfaces are never touched
    CHECK(fourcc("DRUB") == 0x42555244u);                      // the immediate Game_Breakable_OnContact compares

    // Stock entries as data.physics_materials ships them: everything rigid, restitution 0.
    Settings st; // defaults: absorption 60, depth 55 cm, grip 115 %
    Material m{fourcc("TYRW"), 0.0f, 0.4f, 0.01f, 1.0f, 0.0f, 0.f, 0.f, 0.f, 0.f, 0.f, 0.f, 10.f};
    Material stock = m;
    CHECK(tune_material(m, st));
    CHECK(approx(m.friction, 0.4f * 1.15f * 1.15f));
    CHECK(m.max_soft_k > 1e5f && m.max_soft_k < 2e6f); // a spring sized for an 800 kg car, N/m
    CHECK(approx(m.min_soft_k, 4.0f * m.max_soft_k));
    CHECK(approx(m.min_soft_s, 2.0f * m.max_soft_s));
    CHECK(approx(m.speed_for_max_softness, kSoftSpeed));
    CHECK(m.restitution == 0 && m.cfm == stock.cfm && m.correction_vel == stock.correction_vel);
    CHECK(m.min_bounce_vel == 0 && m.dynamic_threshold == stock.dynamic_threshold);
    // Damping ratio of the tuned contact is the slider's.
    const float zeta = damping_ratio(st.absorption);
    CHECK(approx(m.max_soft_s / (2.0f * std::sqrt(m.max_soft_k * kRefMass)), zeta, 1e-3f));
    CHECK(approx(m.min_soft_s / (2.0f * std::sqrt(m.min_soft_k * kRefMass)), zeta, 1e-3f));

    // The physics the numbers promise: an 800 kg car at 30 m/s into the spring-damper.
    auto simulate = [](float k, float s, float v0, float &peak, float &out) {
        double x = 0, v = v0, h = 1.0 / 20000.0;
        peak = 0;
        for (int i = 0; i < 400000; i++) {
            const double a = -(k * x + s * v) / kRefMass;
            v += a * h;
            x += v * h;
            peak = std::max(peak, float(x));
            if (x < 0) // left the barrier
                break;
        }
        out = float(-v);
    };
    float peak = 0, out = 0;
    simulate(m.max_soft_k, m.max_soft_s, 30.0f, peak, out);
    std::printf("tyre wall: 30 m/s -> %.3f m deep, leaves at %.2f m/s (%.0f%%), zeta %.2f\n", peak, out,
                100 * out / 30, zeta);
    CHECK(approx(peak, 0.55f, 0.03f));
    CHECK(out / 30.0f < 0.12f && approx(out / 30.0f, rebound(zeta), 0.15f));
    simulate(m.max_soft_k, m.max_soft_s, 15.0f, peak, out);
    CHECK(approx(peak, 0.275f, 0.03f)); // half the speed, half the depth (linear contact)

    // TecPro is a bit stiffer.
    Material tp{fourcc("DRUB"), 0.0f, 0.4f, 0.01f, 1.0f, 0.0f, 0, 0, 0, 0, 0, 0, 10.f};
    CHECK(tune_material(tp, st));
    simulate(tp.max_soft_k, tp.max_soft_s, 30.0f, peak, out);
    CHECK(approx(peak, 0.55f * 0.85f, 0.03f));
    CHECK(tp.friction > 0.4f && tp.friction < m.friction);

    // Full absorption (zeta 0.9): practically no rebound, same depth.
    Settings dead = st;
    dead.absorption = 100;
    Material d = stock;
    CHECK(tune_material(d, dead));
    simulate(d.max_soft_k, d.max_soft_s, 30.0f, peak, out);
    CHECK(rebound(damping_ratio(100)) < 0.005f && out < 0.01f * 30 && approx(peak, 0.55f, 0.03f));
    CHECK(rebound(1.2f) == 0 && approx(peak_factor(1.0f), std::exp(-1.0f)));

    // Without the CombinePair fix soft contacts are never written.
    Material nofix = stock;
    CHECK(tune_material(nofix, st, false));
    CHECK(nofix.min_soft_k == 0 && nofix.max_soft_k == 0 && nofix.min_soft_s == 0 && nofix.max_soft_s == 0);
    CHECK(nofix.friction > stock.friction);

    // A data patch with its own softness keeps it.
    Material own{fourcc("TYRW"), 0.0f, 0.4f, 0.01f, 1.0f, 0.0f, 5000.f, 90.f, 3000.f, 60.f, 20.f, 0.f, 10.f};
    Material own_copy = own;
    CHECK(tune_material(own, st));
    CHECK(own.min_soft_k == own_copy.min_soft_k && own.max_soft_s == own_copy.max_soft_s);

    // Concrete, armco, boards and the track stay as the game made them.
    for (const char *c : {"WALL", "METL", "SIGN", "PLAS", "SCON", "ASTR", "CARW", "SAST"}) {
        Material x{0, 0.0f, 0.4f, 0.01f, 1.0f, 0, 0, 0, 0, 0, 0, 0, 10.f};
        std::memcpy(&x.code, c, 4);
        Material x0 = x;
        CHECK(!tune_material(x, st));
        CHECK(std::memcmp(&x, &x0, sizeof x) == 0);
    }

    Material bad{fourcc("TYRW"), NAN, 0.7f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
    Material bad_copy = bad;
    CHECK(!tune_material(bad, st));
    CHECK(std::memcmp(&bad, &bad_copy, sizeof bad) == 0);

    // Slider ends stay in range.
    Settings lo = st, hi = st;
    lo.crush = 10, lo.absorption = 0, hi.crush = 100, hi.absorption = 100;
    Material a1 = stock, a2 = stock;
    CHECK(tune_material(a1, lo) && tune_material(a2, hi));
    CHECK(sane(a1.min_soft_k) && sane(a2.max_soft_k) && a1.max_soft_k > a2.max_soft_k);
    CHECK(classify_prop("core_barr_physical_tyres_a_02") == Prop::tyres);
    CHECK(classify_prop("TRK_Physical_Tyres_Stack_4") == Prop::tyres);
    CHECK(classify_prop("tecpro_dynamic_main_a1") == Prop::tecpro);
    CHECK(classify_prop("adv_board_rolex_03") == Prop::boards);
    CHECK(classify_prop("trackside_hoarding_b") == Prop::boards);
    CHECK(classify_prop("core_cone_a") == Prop::other);
    CHECK(classify_prop(nullptr) == Prop::other);

    BreakValues v{200.f, 1.0f, 2.0f};
    CHECK(tune_breakable(v, Prop::tyres, st));
    CHECK(approx(v.impulse, 200.f) && approx(v.explosiveness, 0.6f) && approx(v.falloff, 2.8f));
    BreakValues t{200.f, 1.0f, 2.0f};
    CHECK(tune_breakable(t, Prop::tecpro, st));
    CHECK(approx(t.impulse, 250.f) && approx(t.explosiveness, 0.48f) && approx(t.falloff, 2.0f * 1.3f));
    BreakValues b{50.f, 1.0f, 1.0f};
    CHECK(tune_breakable(b, Prop::boards, st));
    CHECK(approx(b.impulse, 35.f) && approx(b.explosiveness, 0.72f) && approx(b.falloff, 1.2f));
    BreakValues tiny{1.0f, 1.0f, 1.0f};
    Settings sensitive = st;
    sensitive.break_threshold = 25;
    CHECK(tune_breakable(tiny, Prop::boards, sensitive));
    CHECK(tiny.impulse >= 1.0f); // the engine's own floor
    BreakValues inf{INFINITY, 1, 1};
    CHECK(!tune_breakable(inf, Prop::tyres, st));
    BreakValues o{200.f, 1, 1};
    CHECK(!tune_breakable(o, Prop::other, st));
}

// ---------------------------------------------------------------------------------------------
// 2. The plugin against a fake core
// ---------------------------------------------------------------------------------------------
namespace fake {

std::string log_text;
std::map<std::string, double> sliders;
std::map<std::string, bool> toggles;
uintptr_t materials_fn = 0, breakable_fn = 0, combine_fn = 0;
SwoftyMixin mixin{};
int mixins = 0;
int scans = 0;

void log(const char *fmt, ...)
{
    char buf[1024];
    va_list ap;
    va_start(ap, fmt);
    std::vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    log_text += buf;
    log_text += '\n';
}
uintptr_t module_base() { return 0; }
int scan(const char *section, const char *pattern, uintptr_t *out)
{
    scans++;
    if (std::strcmp(section, ".xtls"))
        return 0;
    if (!std::strcmp(pattern, barriers::sites::kBreakParsePattern))
        *out = breakable_fn;
    else if (!std::strcmp(pattern, barriers::sites::kCombinePattern))
        *out = combine_fn;
    else
        return 0;
    return *out ? 1 : 0;
}
uintptr_t resolve(const char *id) { return std::strcmp(id, "Ne_Phys_ParseMaterials") ? 0 : materials_fn; }
void original_parse_materials(void *, void *);
int register_mixins(const SwoftyMixin *m, size_t n)
{
    if (n != 1)
        return 0;
    mixin = m[0];
    mixins++;
    *m[0].orig = reinterpret_cast<void *>(&original_parse_materials);
    return 1;
}
int setting(const char *id, uint32_t type, SwoftySettingValue *out)
{
    out->type = type;
    if (type == SWOFTY_SETTING_SLIDER && sliders.count(id)) {
        out->slider = sliders[id];
        return 1;
    }
    if (type == SWOFTY_SETTING_TOGGLE && toggles.count(id)) {
        out->toggle = toggles[id];
        return 1;
    }
    return 0;
}
int watch_setting(const char *, SwoftySettingFn, void *) { return 1; }

// The game's material library, as the pointer chain in Ne_Phys_ParseMaterials walks it.
alignas(16) uint8_t loader[0x40], obj_a[0x20], obj_b[0x40], lib[0x30];
alignas(16) uint8_t entries[4][0x60];
const barriers::Material stock_entries[4] = {
    // as data.physics_materials ships them: rigid, restitution 0
    {barriers::fourcc("TYRW"), 0.0f, 0.40f, 0.01f, 1.0f, 0.0f, 0.f, 0.f, 0.f, 0.f, 0.f, 0.f, 10.f},
    {barriers::fourcc("WALL"), 0.0f, 0.40f, 0.01f, 1.0f, 0.0f, 0.f, 0.f, 0.f, 0.f, 0.f, 0.f, 10.f},
    {barriers::fourcc("ASTR"), 0.0f, 0.05f, 0.01f, 1.0f, 0.0f, 0.f, 0.f, 0.f, 0.f, 0.f, 0.f, 10.f},
    {barriers::fourcc("DRUB"), 0.0f, 0.40f, 0.01f, 1.0f, 0.0f, 0.f, 0.f, 0.f, 0.f, 0.f, 0.f, 10.f},
};
int parses = 0;
void original_parse_materials(void *, void *)
{
    parses++;
    for (int i = 0; i < 4; i++)
        std::memcpy(entries[i], &stock_entries[i], sizeof(barriers::Material));
}
void build_library()
{
    auto put = [](uint8_t *at, size_t off, auto v) { std::memcpy(at + off, &v, sizeof v); };
    put(loader, 0x28, reinterpret_cast<uintptr_t>(obj_a));
    put(obj_a, 0x08, reinterpret_cast<uintptr_t>(obj_b));
    put(obj_b, 0x28, reinterpret_cast<uintptr_t>(lib));
    put(lib, 0x08, reinterpret_cast<uintptr_t>(entries));
    put(lib, 0x10, uint32_t(4));
    put(lib, 0x18, uint64_t(0x60));
    put(lib, 0x20, uint8_t(1));
}

// Fake Game_Breakable_ParseActivation: the real 15-byte prologue, a jump over the check bytes,
// then an epilogue that undoes the prologue and returns 0, as the game's does.
uint8_t *make_breakable_fn()
{
    auto *f = static_cast<uint8_t *>(VirtualAlloc(nullptr, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE));
    std::memset(f, 0xCC, 0x1000);
    for (auto &c : barriers::sites::kBreakParseChecks)
        std::memcpy(f + c.offset, c.bytes, c.size);
    f[0x0f] = 0xE9; // jmp +0x100
    const int32_t rel = 0x100 - (0x0f + 5);
    std::memcpy(f + 0x10, &rel, 4);
    const uint8_t body[] = {0xB8, 0x00, 0x00, 0x00, 0x00,       // mov eax, 0
                            0xC5, 0xF8, 0x28, 0x74, 0x24, 0x20, // vmovaps xmm6, [rsp+0x20]
                            0x48, 0x83, 0xC4, 0x38,             // add rsp, 0x38
                            0x5B, 0x5F, 0x5E, 0x41, 0x5E,       // pop rbx; pop rdi; pop rsi; pop r14
                            0xC3};
    std::memcpy(f + 0x100, body, sizeof body);
    FlushInstructionCache(GetCurrentProcess(), f, 0x1000);
    return f;
}

// Fake Ne_PhysMaterial_CombinePair: the real 19-byte prologue with its je rel32 to +0x305 (where
// the game returns rdi = a for a == b), and in place of the mixing code a stub that leaves the cache
// as it was (the engine's skip path for two rigid materials) and returns cache + 0x10.
uint8_t *make_combine_fn()
{
    auto *f = static_cast<uint8_t *>(VirtualAlloc(nullptr, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE));
    std::memset(f, 0xCC, 0x1000);
    for (auto &c : barriers::sites::kCombineChecks)
        std::memcpy(f + c.offset, c.bytes, c.size);
    const uint8_t mix[] = {0x49, 0x8D, 0x40, 0x10,  // lea rax, [r8+0x10]
                           0x48, 0x83, 0xC4, 0x20,  // add rsp, 0x20
                           0x5B, 0x5F, 0x5E, 0xC3}; // pop rbx; pop rdi; pop rsi; ret
    std::memcpy(f + 0x13, mix, sizeof mix);
    const uint8_t same[] = {0x48, 0x89, 0xF8, // mov rax, rdi
                            0x48, 0x83, 0xC4, 0x20, 0x5B, 0x5F, 0x5E, 0xC3};
    std::memcpy(f + 0x305, same, sizeof same);
    FlushInstructionCache(GetCurrentProcess(), f, 0x1000);
    return f;
}

}

static void test_plugin()
{
    uint8_t *mat_code = static_cast<uint8_t *>(std::calloc(1, 0x200));
    for (auto &c : barriers::sites::kMaterialsChecks)
        std::memcpy(mat_code + c.offset, c.bytes, c.size);
    fake::materials_fn = reinterpret_cast<uintptr_t>(mat_code);
    uint8_t *brk = fake::make_breakable_fn();
    fake::breakable_fn = reinterpret_cast<uintptr_t>(brk);
    uint8_t *cmb = fake::make_combine_fn();
    fake::combine_fn = reinterpret_cast<uintptr_t>(cmb);
    fake::build_library();

    SwoftyApi api{};
    api.version = SWOFTY_API_VERSION;
    api.log = &fake::log;
    api.module_base = &fake::module_base;
    api.scan = &fake::scan;
    api.resolve = &fake::resolve;
    api.register_mixins = &fake::register_mixins;
    api.setting = &fake::setting;
    api.watch_setting = &fake::watch_setting;

    CHECK(SwoftyAddonInit(&api) == 1);
    CHECK(fake::mixins == 1);
    CHECK(fake::mixin.target && std::strcmp(fake::mixin.target, "Ne_Phys_ParseMaterials") == 0);
    CHECK(fake::mixin.kind == SWOFTY_MIXIN_WRAP);
    CHECK(fake::log_text.find("breakables: hooked") != std::string::npos);
    CHECK(fake::log_text.find("combine fix: hooked") != std::string::npos);
    CHECK(g_combine_fixed.load());
    CHECK(brk[0] == 0xFF && brk[1] == 0x25); // the detour jumps are in place
    CHECK(cmb[0] == 0xFF && cmb[1] == 0x25);

    // Layer 1: the core calls the mixin handler where the game calls Ne_Phys_ParseMaterials.
    auto handler = reinterpret_cast<void (*)(void *, void *)>(fake::mixin.handler);
    handler(fake::loader, nullptr);
    CHECK(fake::parses == 1);
    barriers::Material tyre, wall, road, tecpro;
    std::memcpy(&tyre, fake::entries[0], sizeof tyre);
    std::memcpy(&wall, fake::entries[1], sizeof wall);
    std::memcpy(&road, fake::entries[2], sizeof road);
    std::memcpy(&tecpro, fake::entries[3], sizeof tecpro);
    CHECK(tyre.friction > fake::stock_entries[0].friction);
    CHECK(tyre.max_soft_k > 0 && tyre.min_soft_k > tyre.max_soft_k && tyre.speed_for_max_softness > 0);
    CHECK(tecpro.max_soft_k > tyre.max_soft_k); // stiffer
    CHECK(std::memcmp(&wall, &fake::stock_entries[1], sizeof wall) == 0); // concrete untouched
    CHECK(std::memcmp(&road, &fake::stock_entries[2], sizeof road) == 0); // track surface untouched

    // The game may run the parse again without re-reading (loader flags set): no double tuning.
    barriers::Material before_again = tyre;
    tune_library(fake::loader);
    std::memcpy(&tyre, fake::entries[0], sizeof tyre);
    CHECK(barriers::same_material(tyre, before_again));
    CHECK(fake::log_text.find("(0 from the game, 2 kept)") != std::string::npos);

    // Live: a slider change rewrites the entries from the stock copy at once.
    fake::sliders["crush_depth"] = 30;
    retune_all("test");
    barriers::Material shallow;
    std::memcpy(&shallow, fake::entries[0], sizeof shallow);
    CHECK(shallow.max_soft_k > before_again.max_soft_k * 3.0f); // 55 -> 30 cm: (55/30)^2 = 3.4x stiffer
    CHECK(approx(shallow.friction, before_again.friction));
    fake::sliders.erase("crush_depth");
    fake::toggles["tune_materials"] = false;
    retune_all("test off");
    std::memcpy(&shallow, fake::entries[0], sizeof shallow);
    CHECK(barriers::same_material(shallow, fake::stock_entries[0]));
    fake::toggles.erase("tune_materials");
    retune_all("test on");
    std::memcpy(&shallow, fake::entries[0], sizeof shallow);
    CHECK(barriers::same_material(shallow, before_again));

    // CombinePair fix, called through the detour as the solver calls it.
    using Combine = void *(const barriers::Material *, const barriers::Material *, void *);
    auto combine = reinterpret_cast<Combine *>(cmb);
    alignas(16) uint8_t cache[0x50] = {};
    auto stale = [&] {
        const float v[5] = {2.2e6f, 5.0e4f, 5.5e5f, 2.5e4f, 12.f}; // a tyre-wall pair left in the cache
        std::memcpy(cache + 0x28, v, sizeof v);
    };
    auto soft_of = [&](int i) { float x; std::memcpy(&x, cache + 0x28 + 4 * i, 4); return x; };
    const auto *m_wall = reinterpret_cast<const barriers::Material *>(fake::entries[1]);
    const auto *m_road = reinterpret_cast<const barriers::Material *>(fake::entries[2]);
    const auto *m_tyre = reinterpret_cast<const barriers::Material *>(fake::entries[0]);
    stale();
    CHECK(combine(m_wall, m_road, cache) == cache + 0x10);
    CHECK(soft_of(0) == 0 && soft_of(1) == 0 && soft_of(2) == 0 && soft_of(3) == 0 && soft_of(4) == 0);
    CHECK(g_cleared_pairs.load() == 1);
    stale();
    CHECK(combine(m_tyre, m_road, cache) == cache + 0x10); // a soft side: the engine's own result stays
    CHECK(soft_of(0) == 2.2e6f && g_soft_pairs.load() == 1);
    stale();
    CHECK(combine(m_wall, m_wall, cache) == m_wall); // a == b takes the relocated je
    CHECK(soft_of(0) == 2.2e6f);
    CHECK(fake::log_text.find("soft barrier contact pairs so far: 1") != std::string::npos);

    // A real re-parse restores stock values; they are tuned once again, to the same result.
    handler(fake::loader, nullptr);
    std::memcpy(&tyre, fake::entries[0], sizeof tyre);
    CHECK(barriers::same_material(tyre, before_again));

    // Off in the settings at boot: the game's values stay.
    fake::toggles["tune_materials"] = false;
    handler(fake::loader, nullptr);
    std::memcpy(&tyre, fake::entries[0], sizeof tyre);
    CHECK(barriers::same_material(tyre, fake::stock_entries[0]));
    fake::toggles.erase("tune_materials");

    CHECK(barriers::detour_remove(g_combine_detour));
    CHECK(std::memcmp(cmb, barriers::sites::kCombineProlog, sizeof barriers::sites::kCombineProlog) == 0);
    CHECK(combine(m_wall, m_wall, cache) == m_wall);

    // Layer 2: call the fake game function through the detour, as the game does at track load.
    alignas(16) uint8_t comp[0x120] = {};
    const char *name = "core_barr_physical_tyres_a_02";
    std::memcpy(comp + 0x28, &name, sizeof name);
    const float impulse = 200.f, explosive = 1.0f, falloff = 2.0f;
    std::memcpy(comp + 0xe8, &impulse, 4);
    std::memcpy(comp + 0xf0, &explosive, 4);
    std::memcpy(comp + 0x100, &falloff, 4);
    auto game_fn = reinterpret_cast<int (*)(void *, void *)>(brk);
    CHECK(game_fn(comp, nullptr) == 0);
    float got[4];
    std::memcpy(&got[0], comp + 0xe8, 4);
    std::memcpy(&got[1], comp + 0xf0, 4);
    std::memcpy(&got[2], comp + 0x100, 4);
    std::memcpy(&got[3], comp + 0x104, 4);
    CHECK(approx(got[0], 200.f));
    CHECK(approx(got[1], 0.6f));
    CHECK(approx(got[2], 2.8f));
    CHECK(approx(got[3], 2.8f * 2.8f));

    // A prop that is no barrier keeps its values.
    alignas(16) uint8_t cone[0x120] = {};
    const char *cone_name = "core_cone_a_01";
    std::memcpy(cone + 0x28, &cone_name, sizeof cone_name);
    std::memcpy(cone + 0xe8, &impulse, 4);
    std::memcpy(cone + 0xf0, &explosive, 4);
    std::memcpy(cone + 0x100, &falloff, 4);
    CHECK(game_fn(cone, nullptr) == 0);
    std::memcpy(&got[1], cone + 0xf0, 4);
    std::memcpy(&got[2], cone + 0x100, 4);
    CHECK(approx(got[1], 1.0f) && approx(got[2], 2.0f));

    // Settings from the launcher reach the tuning (sliders arrive as doubles).
    fake::sliders["debris_scatter"] = 100;
    fake::sliders["break_spread"] = 100;
    alignas(16) uint8_t stack[0x120] = {};
    std::memcpy(stack + 0x28, &name, sizeof name);
    std::memcpy(stack + 0xe8, &impulse, 4);
    std::memcpy(stack + 0xf0, &explosive, 4);
    std::memcpy(stack + 0x100, &falloff, 4);
    CHECK(game_fn(stack, nullptr) == 0);
    std::memcpy(&got[1], stack + 0xf0, 4);
    std::memcpy(&got[2], stack + 0x100, 4);
    CHECK(approx(got[1], 1.0f) && approx(got[2], 2.0f));
    fake::sliders.clear();

    // The detour comes off cleanly and the function runs as before.
    CHECK(barriers::detour_remove(g_break_detour));
    CHECK(std::memcmp(brk, barriers::sites::kBreakParseProlog, 15) == 0);
    CHECK(game_fn(comp, nullptr) == 0);

    // A function that does not start with the expected bytes is refused and left alone.
    brk[3] = 0x90;
    barriers::Detour d2;
    void *orig2 = nullptr;
    const char *why = nullptr;
    CHECK(!barriers::detour_install(d2, brk, barriers::sites::kBreakParseProlog, 15,
                                    reinterpret_cast<void *>(&parse_activation), &orig2, &why));
    CHECK(why != nullptr && brk[0] == 0x41);

    // A materials function whose library code differs is not hooked at all.
    mat_code[0x171] ^= 0xff;
    std::vector<SwoftyMixin> list;
    CHECK(!add_materials(&api, list) && list.empty());
}

// ---------------------------------------------------------------------------------------------
// 3. The real F1_25.exe
// ---------------------------------------------------------------------------------------------
static bool pattern_at(const std::vector<uint8_t> &img, size_t at, const char *pattern)
{
    size_t i = at;
    for (const char *p = pattern; *p;) {
        while (*p == ' ')
            p++;
        if (!*p)
            break;
        if (i >= img.size())
            return false;
        if (p[0] == '?') {
            p += 2;
        } else {
            const unsigned v = std::strtoul(std::string(p, 2).c_str(), nullptr, 16);
            if (img[i] != v)
                return false;
            p += 2;
        }
        i++;
    }
    return true;
}

static void test_exe(const char *path)
{
    std::ifstream f(path, std::ios::binary);
    if (!f) {
        std::printf("exe check: cannot open %s\n", path);
        g_fail++;
        return;
    }
    std::vector<uint8_t> img((std::istreambuf_iterator<char>(f)), {});
    auto u16 = [&](size_t o) { return uint32_t(img[o]) | uint32_t(img[o + 1]) << 8; };
    auto u32 = [&](size_t o) { return u16(o) | u16(o + 2) << 16; };
    const size_t pe = u32(0x3c);
    const unsigned nsec = u16(pe + 6), optsz = u16(pe + 20);
    std::printf("exe: %s, PE timestamp %u, %u sections\n", path, u32(pe + 8), nsec);
    CHECK(u32(pe + 8) == 1789118616u); // the catalogued Steam build
    size_t sec = pe + 24 + optsz;
    uint32_t xva = 0, xsize = 0, xraw = 0;
    struct Sec {
        uint32_t va, vsize, raw, rsize;
    };
    std::vector<Sec> secs;
    for (unsigned s = 0; s < nsec; s++, sec += 40) {
        Sec x{u32(sec + 12), u32(sec + 8), u32(sec + 20), u32(sec + 16)};
        secs.push_back(x);
        if (std::memcmp(&img[sec], ".xtls", 5) == 0)
            xva = x.va, xsize = x.rsize, xraw = x.raw;
    }
    CHECK(xsize > 0);
    auto off_of = [&](uint32_t rva) -> size_t {
        for (auto &x : secs)
            if (rva >= x.va && rva < x.va + x.rsize)
                return rva - x.va + x.raw;
        return 0;
    };
    // Signature: exactly one hit in .xtls, at the catalogued function.
    int hits = 0;
    uint32_t hit_rva = 0;
    for (size_t o = xraw; o + 64 < size_t(xraw) + xsize; o++)
        if (img[o] == 0x41 && img[o + 1] == 0x56 && pattern_at(img, o, barriers::sites::kBreakParsePattern))
            hits++, hit_rva = uint32_t(o - xraw + xva);
    std::printf("exe: breakable signature %d hit(s), rva 0x%x\n", hits, hit_rva);
    CHECK(hits == 1 && hit_rva == barriers::sites::kBreakParseRva);
    const size_t bo = off_of(barriers::sites::kBreakParseRva);
    for (auto &c : barriers::sites::kBreakParseChecks)
        CHECK(std::memcmp(&img[bo + c.offset], c.bytes, c.size) == 0);
    const size_t mo = off_of(barriers::sites::kMaterialsRva);
    for (auto &c : barriers::sites::kMaterialsChecks)
        CHECK(std::memcmp(&img[mo + c.offset], c.bytes, c.size) == 0);
    // CombinePair: unique signature at the catalogued address, every checked byte, and the je
    // the detour relocates really lands on the a == b return at +0x305.
    int chits = 0;
    uint32_t c_rva = 0;
    for (size_t o = xraw; o + 64 < size_t(xraw) + xsize; o++)
        if (img[o] == 0x56 && img[o + 1] == 0x57 && pattern_at(img, o, barriers::sites::kCombinePattern))
            chits++, c_rva = uint32_t(o - xraw + xva);
    std::printf("exe: combine signature %d hit(s), rva 0x%x\n", chits, c_rva);
    CHECK(chits == 1 && c_rva == barriers::sites::kCombineRva);
    const size_t co = off_of(barriers::sites::kCombineRva);
    for (auto &c : barriers::sites::kCombineChecks)
        CHECK(std::memcmp(&img[co + c.offset], c.bytes, c.size) == 0);
    int32_t je = 0;
    std::memcpy(&je, &img[co + barriers::sites::kCombineJccAt + 2], 4);
    CHECK(barriers::sites::kCombineJccAt + 6 + je == 0x305);
    const uint8_t ret_a[] = {0x48, 0x89, 0xF8, 0x48, 0x83, 0xC4, 0x20, 0x5B, 0x5F, 0x5E, 0xC3};
    CHECK(std::memcmp(&img[co + 0x305], ret_a, sizeof ret_a) == 0);
    // The solver reads the soft fields of the record at +0x18..+0x24 (0x1440f7730..0x1440f774b).
    const uint8_t solver_soft[] = {0xC5, 0xFA, 0x10, 0x48, 0x20, 0xC5, 0xFA, 0x10, 0x50, 0x18, 0xC5, 0xFA, 0x10, 0x58, 0x1C};
    CHECK(std::memcmp(&img[off_of(0x40f7730)], solver_soft, sizeof solver_soft) == 0);
    // The core binding's own pattern for Ne_Phys_ParseMaterials (native/dist bindings.json).
    CHECK(pattern_at(img, mo, "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC 78 C5 F8 29 7C 24 60 C5 F8 29 74 24 50 48 "
                              "89 CE 80 79 18 00"));
    // The two names the engine itself gives the 200.0 break impulse.
    const std::string blob(img.begin(), img.end());
    CHECK(blob.find(std::string("physical_tyres\0", 15)) != std::string::npos);
    CHECK(blob.find("SCON;SAST;SMET;SPLA;SIGN;PLAS;WALL;METL;DTYR;DRUB;RUBB;TYRW") != std::string::npos);
}

int main(int argc, char **argv)
{
    test_rules();
    test_plugin();
    if (argc > 1)
        test_exe(argv[1]);
    if (std::getenv("RB_SHOW_LOG"))
        std::printf("--- plugin log ---\n%s", fake::log_text.c_str());
    std::printf("%d passed, %d failed\n", g_pass, g_fail);
    return g_fail ? 1 : 0;
}