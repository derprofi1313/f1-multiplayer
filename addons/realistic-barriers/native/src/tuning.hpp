// SPDX-License-Identifier: AGPL-3.0-or-later
// Realistic Barrier Crashes: the tuning rules. Pure functions with no Windows or game calls, so
// native/test/tests.cpp checks them on the PC without the game.
#pragma once
#include <algorithm>
#include <cctype>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>

namespace barriers {

// ---------------------------------------------------------------------------------------------
// Physics material library entry.
// Layout from the stores of Ne_Phys_ParseMaterials (F1_25.exe 0x1440bee90, Steam build d873a1c8):
// friction -> +0x08 (0x1440bf0ba), restitution -> +0x04 (0x1440bf0ed), minBounceVel -> +0x14,
// cfm -> +0x0c, correctionVel -> +0x10, minSoftK/S -> +0x18/+0x1c, maxSoftK/S -> +0x20/+0x24,
// speedForMaxSoftness -> +0x28, dynamicScale -> +0x2c, dynamicThreshold -> +0x30, and the
// 4-character name copied raw to +0x00 (0x1440bf4a7, only when strlen(name) == 4).
// ---------------------------------------------------------------------------------------------
struct Material {
    uint32_t code;
    float restitution;
    float friction;
    float cfm;
    float correction_vel;
    float min_bounce_vel;
    float min_soft_k;
    float min_soft_s;
    float max_soft_k;
    float max_soft_s;
    float speed_for_max_softness;
    float dynamic_scale;
    float dynamic_threshold;
};
static_assert(offsetof(Material, restitution) == 0x04);
static_assert(offsetof(Material, friction) == 0x08);
static_assert(offsetof(Material, min_bounce_vel) == 0x14);
static_assert(offsetof(Material, min_soft_k) == 0x18);
static_assert(offsetof(Material, max_soft_s) == 0x24);
static_assert(offsetof(Material, speed_for_max_softness) == 0x28);
static_assert(offsetof(Material, dynamic_threshold) == 0x30);

constexpr uint32_t fourcc(const char (&s)[5])
{
    return uint32_t(uint8_t(s[0])) | uint32_t(uint8_t(s[1])) << 8 | uint32_t(uint8_t(s[2])) << 16 |
           uint32_t(uint8_t(s[3])) << 24;
}

inline void code_text(uint32_t code, char out[5])
{
    for (int i = 0; i < 4; i++) {
        char c = char((code >> (8 * i)) & 0xff);
        out[i] = (c >= 0x20 && c < 0x7f) ? c : '?';
    }
    out[4] = 0;
}

// The engine's own barrier list (string at 0x145845671, used by the impact effects code at
// 0x1434983dc): SCON;SAST;SMET;SPLA;SIGN;PLAS;WALL;METL;DTYR;DRUB;RUBB;TYRW. The effect groups
// next to it put SIGN/METL/SMET with sparks (metal) and SCON/WALL with concrete dust, which is how
// the classes below are chosen. SAST is not in any effect group, so it stays untouched.
enum class Surface { other, tyre_wall, tecpro, board, hard };

inline Surface classify_material(uint32_t code)
{
    switch (code) {
    case fourcc("TYRW"): // static tyre wall
    case fourcc("DTYR"): // dynamic (loose) tyres
        return Surface::tyre_wall;
    case fourcc("DRUB"): // TecPro / rubber barrier modules (Car_Contact_ModifyVehicle treats DRUB as tecpro)
    case fourcc("RUBB"):
        return Surface::tecpro;
    case fourcc("SIGN"): // boards, advertising hoardings (Banden)
    case fourcc("PLAS"): // plastic: bollards, polystyrene blocks
    case fourcc("SPLA"):
        return Surface::board;
    case fourcc("WALL"): // concrete walls
    case fourcc("SCON"):
    case fourcc("METL"): // armco, metal fences
    case fourcc("SMET"):
        return Surface::hard;
    default:
        return Surface::other;
    }
}

inline const char *surface_name(Surface s)
{
    switch (s) {
    case Surface::tyre_wall: return "tyre wall";
    case Surface::tecpro: return "tecpro";
    case Surface::board: return "board";
    case Surface::hard: return "hard wall";
    default: return "other";
    }
}

// The player's settings, as the launcher and ADDON SETTINGS give them.
struct Settings {
    bool materials = true;     // tune barrier materials
    bool breakables = true;    // tune barrier parts that break loose
    int absorption = 60;       // 0..100: damping of soft barriers (0 = springy, 100 = dead, no rebound)
    int crush = 55;            // 10..100 cm: how far a car sinks into a tyre wall in a 108 km/h square hit
    int grip = 115;            // 100..150 %: how much tyre walls hold the car instead of letting it slide along
    int break_threshold = 100; // 25..400 %: impulse a barrier part needs to break loose, 100 = stock
    int scatter = 60;          // 20..150 %: how hard loose parts are thrown, 100 = stock
    int spread = 140;          // 50..300 %: how far around the impact parts come loose, 100 = stock
};

inline float clampf(float v, float lo, float hi) { return std::min(hi, std::max(lo, v)); }
inline float pct(int v) { return float(v) / 100.0f; }
inline bool sane(float v) { return std::isfinite(v) && v >= 0.0f && v < 1.0e9f; }

// Contact values of two materials mix as sqrt(a*b) (Ne_PhysMaterial_CombinePair 0x1440e7350). To
// move the car-against-barrier value by a factor g, the barrier side moves by g*g.
inline float pair_factor(float g) { return clampf(g * g, 0.0001f, 16.0f); }

// ---------------------------------------------------------------------------------------------
// Soft contact. Facts from the solver (0x1440f6e20, callers 0x1440f5320 / 0x1440f5900):
//  - only when a pair has soft K/S, K and S are lerped from min to max by the relative speed of the
//    two bodies over speedForMaxSoftness (0x1440f7700..0x1440f775c), max from that speed on;
//  - K and S are multiplied by the island's smallest inverse mass (= 1/heaviest body) and turned
//    into ERP = hK/(hK+S), CFM = 1/(hK+S) (0x1440f7bdf..0x1440f7c3c). In the mass-normalised
//    solver that makes K a spring in N/m and S a damper in N*s/m acting on the heaviest body.
//  - CombinePair: when one side is 0 the other side's value is taken as is; car materials are 0, so
//    the barrier's numbers are the contact's numbers.
// Stock data (ego_package/application.erp, data.physics_materials): every material has K = S = 0,
// restitution 0, minBounceVel 0, cfm 0.01, correctionVel 1.
// ---------------------------------------------------------------------------------------------
constexpr float kRefMass = 800.0f;     // F1 car with driver (the heaviest body in a car-barrier island)
constexpr float kRefSpeed = 30.0f;     // m/s into the barrier (108 km/h square-on) for the depth slider
constexpr float kSoftSpeed = 12.0f;    // m/s: above this the contact uses its softest (max) values
constexpr float kLowSpeedStiff = 4.0f; // K at rest is 4x the impact K: a car leaning on a wall does not sink

struct Soft {
    float min_k = 0, min_s = 0, max_k = 0, max_s = 0, speed = 0;
};

inline float damping_ratio(int absorption) { return 0.25f + 0.65f * clampf(pct(absorption), 0.0f, 1.0f); }

// Peak displacement of a unit-mass spring-damper (natural frequency 1) hit at unit speed.
inline float peak_factor(float zeta)
{
    if (zeta < 0.999f) {
        const float r = std::sqrt(1.0f - zeta * zeta);
        return std::exp(-zeta / r * std::atan(r / zeta));
    }
    if (zeta <= 1.001f)
        return std::exp(-1.0f);
    const float r = std::sqrt(zeta * zeta - 1.0f);
    return std::exp(-zeta / r * std::atanh(r / zeta));
}

// Share of the impact speed the car keeps when it leaves an (underdamped) contact.
inline float rebound(float zeta)
{
    if (zeta >= 1.0f)
        return 0.0f;
    return std::exp(-zeta * 3.14159265f / std::sqrt(1.0f - zeta * zeta));
}

// Spring and damper so an 800 kg car hitting at kRefSpeed sinks `depth_m` deep at most.
inline Soft soft_contact(float depth_m, float zeta)
{
    const float omega = kRefSpeed * peak_factor(zeta) / depth_m;
    Soft s;
    s.max_k = kRefMass * omega * omega;
    s.max_s = 2.0f * zeta * kRefMass * omega;
    // Same damping ratio at rest: 4x K needs 2x S.
    s.min_k = s.max_k * kLowSpeedStiff;
    s.min_s = s.max_s * std::sqrt(kLowSpeedStiff);
    s.speed = kSoftSpeed;
    return s;
}

struct MaterialPlan {
    float friction = 1;
    bool soft = false;
    Soft contact;
};

inline MaterialPlan plan_for(Surface s, const Settings &st)
{
    MaterialPlan p;
    const float depth = clampf(float(st.crush) / 100.0f, 0.10f, 1.0f);
    const float grip = clampf(pct(st.grip), 1.0f, 1.5f);
    const float zeta = damping_ratio(st.absorption);
    switch (s) {
    case Surface::tyre_wall:
        // Tyre stacks give way over their depth and the rubber grabs the bodywork.
        p.friction = pair_factor(grip);
        p.soft = true;
        p.contact = soft_contact(depth, zeta);
        break;
    case Surface::tecpro:
        // TecPro: chained polyethylene modules with foam, a little stiffer than loose tyres.
        p.friction = pair_factor(1.0f + (grip - 1.0f) * 0.5f);
        p.soft = true;
        p.contact = soft_contact(depth * 0.85f, zeta);
        break;
    case Surface::board:
    case Surface::hard:
    case Surface::other:
        // Stock restitution is 0 everywhere, so there is no rebound to take away; concrete, armco and
        // boards stay rigid (boards that give way are breakables, see below).
        break;
    }
    return p;
}

// Builds the tuned entry from a stock entry. Returns false when the entry is not a material this
// mod changes or holds values that do not look like a material (then `m` is left as it was).
// `allow_soft` is false when the CombinePair fix is not in place: then soft contacts are never
// written, because the engine would leak them to other pairs.
inline bool tune_material(Material &m, const Settings &st, bool allow_soft = true)
{
    const Surface s = classify_material(m.code);
    if (s != Surface::tyre_wall && s != Surface::tecpro)
        return false;
    const float vals[] = {m.restitution, m.friction, m.min_bounce_vel, m.min_soft_k,
                          m.min_soft_s,  m.max_soft_k, m.max_soft_s, m.speed_for_max_softness};
    for (float v : vals)
        if (!sane(v))
            return false;
    const MaterialPlan p = plan_for(s, st);
    m.friction = clampf(m.friction * p.friction, 0.0f, 4.0f);
    const bool stock_rigid = m.min_soft_k == 0 && m.min_soft_s == 0 && m.max_soft_k == 0 && m.max_soft_s == 0;
    // A future data patch that ships its own softness keeps it.
    if (p.soft && allow_soft && stock_rigid) {
        m.min_soft_k = p.contact.min_k;
        m.min_soft_s = p.contact.min_s;
        m.max_soft_k = p.contact.max_k;
        m.max_soft_s = p.contact.max_s;
        m.speed_for_max_softness = p.contact.speed;
    }
    return true;
}

inline bool same_material(const Material &a, const Material &b) { return std::memcmp(&a, &b, sizeof a) == 0; }

// ---------------------------------------------------------------------------------------------
// Breakable props (barrier parts that become loose physics bodies when hit).
// Game_Breakable_ParseActivation (0x144588c90) writes, on the breakable component:
//   +0x0e8 activationImpulse (float)  - forced to 200.0 when the name holds physical_tyres or tecpro
//   +0x0f0 explosiveness (float)
//   +0x100 impulseFalloffDist (float), +0x104 its square
//   +0x108 destroyNotReset (bool)
// and reads the instance name through the char* at +0x028.
// Game_Breakable_OnContact (0x14458bb90) activates the part when the contact impulse (report
// +0x50) reaches +0x0e8.
// ---------------------------------------------------------------------------------------------
constexpr size_t kBreakName = 0x28;
constexpr size_t kBreakImpulse = 0xe8;
constexpr size_t kBreakExplosive = 0xf0;
constexpr size_t kBreakFalloff = 0x100;
constexpr size_t kBreakFalloffSq = 0x104;

enum class Prop { other, tyres, tecpro, boards };

inline bool contains_ci(const char *hay, const char *needle)
{
    if (!hay || !needle)
        return false;
    const size_t n = std::strlen(needle);
    for (const char *p = hay; *p; p++) {
        size_t i = 0;
        while (i < n && p[i] && std::tolower((unsigned char)p[i]) == std::tolower((unsigned char)needle[i]))
            i++;
        if (i == n)
            return true;
    }
    return false;
}

inline Prop classify_prop(const char *name)
{
    if (!name)
        return Prop::other;
    // The engine itself keys on these two words (0x145bed943 / 0x145a8f5a0).
    if (contains_ci(name, "physical_tyres") || contains_ci(name, "tyre_stack") || contains_ci(name, "tyrewall") ||
        contains_ci(name, "tyre_wall") || contains_ci(name, "barr_tyre"))
        return Prop::tyres;
    if (contains_ci(name, "tecpro"))
        return Prop::tecpro;
    static const char *const boards[] = {"advert", "hoarding", "billboard", "banner", "signboard", "sign_",
                                         "board",  "fence_panel", "polystyrene", "poly_block", "marker_board"};
    for (const char *b : boards)
        if (contains_ci(name, b))
            return Prop::boards;
    return Prop::other;
}

inline const char *prop_name(Prop p)
{
    switch (p) {
    case Prop::tyres: return "tyres";
    case Prop::tecpro: return "tecpro";
    case Prop::boards: return "boards";
    default: return "other";
    }
}

struct BreakValues {
    float impulse, explosiveness, falloff;
};

struct BreakPlan {
    float impulse = 1, explosiveness = 1, falloff = 1;
};

inline BreakPlan break_plan(Prop p, const Settings &st)
{
    BreakPlan b;
    if (p == Prop::other)
        return b;
    const float threshold = clampf(pct(st.break_threshold), 0.25f, 4.0f);
    const float scatter = clampf(pct(st.scatter), 0.2f, 1.5f);
    const float spread = clampf(pct(st.spread), 0.5f, 3.0f);
    switch (p) {
    case Prop::tyres:
        // Tyres are strapped together in stacks: neighbours come loose with the hit stack, and they
        // tumble rather than fly.
        b.impulse = threshold;
        b.explosiveness = scatter;
        b.falloff = spread;
        break;
    case Prop::tecpro:
        // TecPro modules are chained and heavy: they need a bit more to break loose and move less.
        b.impulse = threshold * 1.25f;
        b.explosiveness = scatter * 0.8f;
        b.falloff = 1.0f + (spread - 1.0f) * 0.75f;
        break;
    case Prop::boards:
        // Boards snap off easily and only the hit panel goes.
        b.impulse = threshold * 0.7f;
        b.explosiveness = std::min(1.0f, scatter * 1.2f);
        b.falloff = 1.0f + (spread - 1.0f) * 0.5f;
        break;
    case Prop::other:
        break;
    }
    return b;
}

// Returns false (and writes nothing) when a value does not look like the engine's.
inline bool tune_breakable(BreakValues &v, Prop p, const Settings &st)
{
    if (p == Prop::other)
        return false;
    if (!sane(v.impulse) || !sane(v.explosiveness) || !sane(v.falloff))
        return false;
    const BreakPlan b = break_plan(p, st);
    // The engine floors the threshold at 1.0 for every other prop (vmaxss at 0x144588d58).
    v.impulse = std::max(1.0f, v.impulse * b.impulse);
    v.explosiveness = v.explosiveness * b.explosiveness;
    v.falloff = v.falloff * b.falloff;
    return true;
}

}