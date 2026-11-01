#include "../runtime.hpp"

#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstring>
#include <memory>
#include <string>
#include <unordered_set>
#include <vector>

#include "../registry/table.hpp"
#include "car_live.hpp"
#include "car_profile.hpp"
#include "context.hpp"
#include "core_events.hpp"
#include "hooks.hpp"
#include "session_points.hpp"

// The live car operations on the running game (car_live.hpp): g_Sess_VehicleTable and g_Sess_VehicleCount
// resolve once the code is ready, memory reads and writes are checked (swofty::mem), writes run only on the
// flow thread (the one that ticks on_frame), and nothing is handed over on an exe that isn't the KB build,
// whose field offsets these are. car.built and car.reset are game points: the mid-hook on site_Car_LiveCarBuilt
// and the detour on Car_Systems_Reset_Thunk sit there while an addon subscribes, and the events fire from the next
// frame (car_live.hpp, Ops::take).

namespace swofty_core::hooks {
namespace {

namespace mem = swofty::mem;

struct CoreTag {};
using Ops = car_live::Ops<CoreTag>;

std::atomic<uint32_t> g_flow_tid{0};

bool on_flow_thread()
{
    uint32_t t = g_flow_tid.load();
    return !t || t == GetCurrentThreadId();
}

std::unique_ptr<session_points::MidPoint> g_built, g_setup;
std::unique_ptr<GamePoint> g_reset, g_replay, g_wing, g_hit, g_pit;

// The KB rows the live levers rest on. A row is ready once it has an independent agree verify row (re/PROTOCOL.md
// section 5); the others stay off and log "<field> off: needs <row>". Flip a row here after the lead's verify agent
// has recorded its agree row.
struct Row {
    const char *name;
    bool ready;
};

constexpr Row ROWS[] = {
    {"site_Car_EngineDriveTorque", true},
    {"Car_Upgrades_ApplyToVehicle", true},
    {"site_Car_SetupTuningApplied", true},
    {"site_Ai_TableGripInput", true},
    {"CarF1Aero.lift_scale", true},
    {"CarF1Aero.drag_scale", true},
    {"CarBrakeEcu.brake_bias", true},
    {"CarBrakeEcu.brake_pressure", true},
    {"CarSuspTyreType.grip_multiplier", false},
    {"CarSuspAxle.setup_camber", true},
    {"CarSuspAxle.setup_toe", true},
    {"CarSuspAxle.setup_anti_roll", true},
    {"CarSuspAxle.ride_height_offset", true},
    {"Car_Tuning_SetChassisZCom", true},
    {"CarChassis.tyre_physics_group", true},
    {"Car_Tyre_SetWearFactor", true},
    {"Car_FrontWing_Init", true},
    {"Car_Register_DetachablePart_Thunk", true},
    {"Car_FrontWing_OnCollision", true},
    {"Game_Pitlane_SetVehicleStatus", true},
    {"Game_Get_ModelNodeId", true},
    {"PitStopInstance.car", false},
    {"Car_FuelEcu_SetMode_Thunk", true},
};

bool row_ready(const char *name)
{
    for (const Row &r : ROWS)
        if (!std::strcmp(r.name, name))
            return r.ready;
    return false;
}

void m_built(safetyhook::Context &ctx) { Ops::built(ctx.rdi); }
void m_setup(safetyhook::Context &ctx) { Ops::setup_applied(ctx.rdi); }

void d_reset(void *loader, uint32_t arg)
{
    g_reset->original<void>(loader, arg);
    Ops::reset(reinterpret_cast<uintptr_t>(loader));
}

uint64_t d_replay(void *loader, void *name, uint32_t flag, float perc)
{
    uint64_t r = g_replay->original<uint64_t>(loader, name, flag, perc);
    Ops::replayed(reinterpret_cast<uintptr_t>(loader), perc);
    return r;
}

constexpr uintptr_t WING_CAR = 0x08, WING_SIDE_SCALE = 0x34, WING_FULL_SCALE = 0x38, WING_LEFT = 0x48, WING_RIGHT = 0x4c;
constexpr uintptr_t PITLANE_STATUS_TABLE = 0x68, PITLANE_STATUS = 0x04, PITLANE_STOPS = 0x14;
constexpr uint32_t MAX_PIT_CARS = 27;

bool detach_predicate(void *wing, float k)
{
    float full = 0, left = 0, right = 0;
    if (!mem::read(reinterpret_cast<uintptr_t>(wing) + WING_FULL_SCALE, full) ||
        !mem::read(reinterpret_cast<uintptr_t>(wing) + WING_LEFT, left) ||
        !mem::read(reinterpret_cast<uintptr_t>(wing) + WING_RIGHT, right))
        return false;
    return k * full <= left + right;
}

int32_t add_part(void *wing, const char *node, float detach_at, bool)
{
    uintptr_t reg = swofty::api->resolve("Car_Register_DetachablePart_Thunk");
    uintptr_t intern = swofty::api->resolve("Game_Get_ModelNodeId");
    if (!reg || !intern || !wing)
        return SWOFTY_ASSET_UNAVAILABLE;
    uint32_t id = reinterpret_cast<uint32_t (*)(const char *)>(intern)(node);
    struct Callback {
        bool (*fn)(void *, float);
        void *ctx;
    } cb{&detach_predicate, wing};
    alignas(16) float params[28] = {};
    int32_t kind = 2;
    std::memcpy(&params[0], &kind, sizeof kind);
    params[1] = 0.4f;
    for (int i = 0; i < 4; i++)
        params[4 + i * 5] = 1.0f;
    params[20] = params[21] = params[22] = 0.05f;
    reinterpret_cast<void (*)(void *, void *, uint32_t *, void *, float)>(reg)(wing, &cb, &id, params, detach_at);
    return SWOFTY_ASSET_OK;
}

void d_wing_init(void *wing, char reset)
{
    g_wing->original<void>(wing, reset);
    uintptr_t car = 0;
    uint8_t id = 0;
    if (reset || !mem::read(reinterpret_cast<uintptr_t>(wing) + WING_CAR, car) || !car ||
        !mem::read(car + 0xe8, id))
        return;
    events::car_parts_register(id, "", wing, [wing](const char *node, float at, bool debris) {
        return add_part(wing, node, at, debris);
    });
}

void d_wing_hit(void *wing, void *contact)
{
    float left = 0, right = 0, left_after = 0, right_after = 0;
    uintptr_t w = reinterpret_cast<uintptr_t>(wing);
    bool ok = mem::read(w + WING_LEFT, left) && mem::read(w + WING_RIGHT, right);
    g_hit->original<void>(wing, contact);
    if (ok && mem::read(w + WING_LEFT, left_after) && mem::read(w + WING_RIGHT, right_after)) {
        Ops::damaged(w, swofty::cars::events::Damage::front_wing_left, left_after - left);
        Ops::damaged(w, swofty::cars::events::Damage::front_wing_right, right_after - right);
    }
}

void d_pit_status(void *lane, uint8_t vehicle, uint32_t status)
{
    uint32_t before = 0, stops_before = 0, after = 0, stops_after = 0;
    uintptr_t entry = 0;
    auto read_entry = [&](uint32_t &state, uint32_t &stops) {
        uintptr_t base = reinterpret_cast<uintptr_t>(lane) + PITLANE_STATUS_TABLE + uintptr_t(vehicle) * 16;
        return mem::read(base, entry) && entry && mem::read(entry + PITLANE_STATUS, state) &&
               mem::read(entry + PITLANE_STOPS, stops);
    };
    bool known = vehicle < MAX_PIT_CARS && read_entry(before, stops_before);
    g_pit->original<void>(lane, vehicle, status);
    if (!known || !read_entry(after, stops_after) || after == before)
        return;
    events::session_pit(vehicle, before, after);
    if (stops_after != stops_before)
        events::session_pit_stop(vehicle, stops_after);
}

std::string session_type()
{
    auto facts = core().current();
    return facts && facts->session ? facts->session->type : std::string();
}

std::vector<car_profile::Entry> profile_entries()
{
    std::vector<car_profile::Entry> out;
    auto &store = swofty_core::registry::store();
    for (const std::string &id : store.list("car.profile", "")) {
        auto rec = store.get("car.profile", id);
        if (rec.status != swofty_core::registry::Status::ok)
            continue;
        car_profile::Entry e;
        e.id = id;
        std::string why;
        if (!car_profile::parse(rec.value, e.profile, &why)) {
            swofty::log("[cars] car.profile %s skipped: %s", id.c_str(), why.c_str());
            continue;
        }
        out.push_back(std::move(e));
    }
    return out;
}

bool within(const car_profile::Entry &e)
{
    if (e.profile.within.is_null())
        return true;
    swofty::json::Value arr = e.profile.within.is_array() ? e.profile.within : swofty::json::Value::array();
    if (!e.profile.within.is_array())
        arr.push_back(e.profile.within);
    Selectors sel;
    std::string why;
    if (!Selectors::compile(swofty::json::dump(arr), sel, &why)) {
        swofty::log("[cars] car.profile %s: within does not compile: %s", e.id.c_str(), why.c_str());
        return false;
    }
    auto facts = core().current();
    return facts && sel.match(*facts);
}

std::unordered_set<std::string> g_logged;

void apply_profiles(uint32_t index)
{
    auto entries = profile_entries();
    uint32_t ai = 1;
    Ops::op_car(index, nullptr, &ai);
    auto report = car_profile::Applier<Ops>::evaluate(entries, session_type(), within, index, ai != 0);
    for (const auto &id : report.needs_identity)
        if (g_logged.insert("identity:" + id).second)
            swofty::log("[cars] car.profile %s skipped: its cars selector needs the car's vehicle, team or driver (T121)",
                        id.c_str());
    for (const auto &why : report.refused)
        if (g_logged.insert("refused:" + why).second)
            swofty::log("[cars] car.profile layer refused: %s (the field is off or unknown)", why.c_str());
    for (const auto &e : entries)
        if (e.profile.fuel_modes && g_logged.insert("fuel:" + e.id).second)
            swofty::log("[cars] car.profile %s: fuel_modes off: the fuel mode call is not bound", e.id.c_str());
    if (report.matched && g_logged.insert("applied:" + std::to_string(index) + session_type()).second)
        swofty::log("[cars] car %u: %zu car.profile layer(s) from %zu profile(s) in a %s session", index, report.layers,
                    report.matched, session_type().c_str());
}

uint64_t g_epoch = 0;

void frame(double, void *)
{
    g_flow_tid = GetCurrentThreadId();
    auto facts = core().current();
    bool changed = facts && facts->epoch != g_epoch;
    if (facts)
        g_epoch = facts->epoch;
    if (changed) {
        uint32_t n = 0;
        if (Ops::op_count(&n) == SWOFTY_ASSET_OK)
            for (uint32_t i = 0; i < n; i++)
                apply_profiles(i);
        Ops::flush();
    }
    for (const auto &p : Ops::take()) {
        swofty::cars::live::Car car(p.index, reinterpret_cast<void *>(p.car));
        if (p.what != Ops::What::setup)
            apply_profiles(p.index);
        Ops::apply_car(p.index);
        switch (p.what) {
        case Ops::What::built: events::car_built(car); break;
        case Ops::What::reset: events::car_reset(car, false); break;
        case Ops::What::replayed: events::car_upgrades_replayed(car, p.progress); break;
        case Ops::What::setup: events::car_setup_applied(car); break;
        case Ops::What::damage: events::car_damage(car, p.part, p.progress); break;
        }
    }
    Ops::flush();
}

void setup()
{
    car_live::Env e;
    e.read = [](uintptr_t a, void *out, size_t n) {
        if (!mem::readable(a, n))
            return false;
        std::memcpy(out, reinterpret_cast<const void *>(a), n);
        return true;
    };
    e.write = [](uintptr_t a, const void *in, size_t n) {
        if (!mem::readable(a, n))
            return false;
        std::memcpy(reinterpret_cast<void *>(a), in, n);
        return true;
    };
    e.flow_thread = on_flow_thread;
    e.enabled = [] { return runtime::enabled("events", nullptr); };
    e.row_ready = row_ready;
    if (!runtime::same_build()) {
        swofty::log("[cars] live car operations off: the field offsets are the KB build's and this is not that build");
    } else {
        e.table = swofty::api->resolve("g_Sess_VehicleTable");
        e.count = swofty::api->resolve("g_Sess_VehicleCount");
        if (!e.table || !e.count)
            swofty::log("[cars] live car operations off: g_Sess_VehicleTable or g_Sess_VehicleCount does not resolve");
        else
            swofty::log("[cars] live car operations on: %zu fields", sizeof car_live::FIELDS / sizeof car_live::FIELDS[0]);
        if (uintptr_t fn = swofty::api->resolve("Car_Engine_ComputeDerived_Thunk"))
            e.engine_derived = [fn](uintptr_t engine) {
                reinterpret_cast<void (*)(void *)>(fn)(reinterpret_cast<void *>(engine));
            };
        else
            swofty::log("[cars] engine_limiter off: Car_Engine_ComputeDerived_Thunk does not resolve");
        if (uintptr_t fn = swofty::api->resolve("Car_Tuning_SetFinalDrive"))
            e.gearbox_reset = [fn](uintptr_t gearbox, uint32_t flags) {
                reinterpret_cast<void (*)(void *, uint32_t)>(fn)(reinterpret_cast<void *>(gearbox), flags);
            };
        else
            swofty::log("[cars] final_drive and gear ratios off: Car_Tuning_SetFinalDrive does not resolve");
        e.chassis_reset = [](uintptr_t chassis, uint32_t flags) {
            uintptr_t vt = 0, fn = 0;
            if (!mem::object_like(chassis) || !mem::read(chassis, vt) || !mem::read(vt + 0x18, fn) || !mem::in_image(fn))
                return;
            reinterpret_cast<void (*)(void *, uint32_t)>(fn)(reinterpret_cast<void *>(chassis), flags);
        };
        if (uintptr_t fn = swofty::api->resolve("Car_Tyre_SetWearFactor"))
            e.tyre_refit = [fn](uintptr_t component) {
                reinterpret_cast<void (*)(void *, bool, uint16_t)>(fn)(reinterpret_cast<void *>(component), true, 0x101);
            };
        for (const car_live::Field &f : car_live::FIELDS)
            if (f.needs && !row_ready(f.needs) && g_logged.insert(std::string("off:") + f.needs).second)
                swofty::log("[cars] %s off: needs %s", f.name, f.needs);
        swofty::log("[session] session.pit_stop stationary time and tyre change off: needs PitStopInstance.car");
        swofty::log("[cars] refit_tyres off: needs Car_Tyre_SetWearFactor, CarChassis.tyre_physics_group and the car to tyre component path");
        swofty::log("[cars] set_fuel_mode is not offered: the fuel mode call is not bound");
    }
    Ops::bind(std::move(e));
    const SwoftyEvents *ev = runtime::events_table();
    uint32_t token = 0;
    if (ev && ev->on_frame)
        ev->on_frame(frame, nullptr, &token);
}

}

void declare_car_points(Table &t)
{
    g_built = std::make_unique<session_points::MidPoint>("car.built", "site_Car_LiveCarBuilt", &m_built, game_ops());
    t.declare_game("car.built", SWOFTY_POINT_NOTIFY, swofty::cars::events::Built::version, g_built.get());
    g_reset = std::make_unique<GamePoint>("car.reset", "Car_Systems_Reset_Thunk", reinterpret_cast<void *>(&d_reset),
                                          game_ops());
    t.declare_game("car.reset", SWOFTY_POINT_NOTIFY, swofty::cars::events::Reset::version, g_reset.get());
    g_replay = std::make_unique<GamePoint>("car.upgrades.replayed", "Car_Upgrades_ApplyToVehicle",
                                           reinterpret_cast<void *>(&d_replay), game_ops());
    t.declare_game("car.upgrades.replayed", SWOFTY_POINT_NOTIFY, swofty::cars::events::UpgradesReplayed::version,
                   g_replay.get());
    g_wing = std::make_unique<GamePoint>("car.parts.register", "Car_FrontWing_Init", reinterpret_cast<void *>(&d_wing_init),
                                         game_ops());
    t.declare_game("car.parts.register", SWOFTY_POINT_NOTIFY, swofty::cars::events::PartsRegister::version, g_wing.get());
    g_hit = std::make_unique<GamePoint>("car.damage", "Car_FrontWing_OnCollision", reinterpret_cast<void *>(&d_wing_hit),
                                        game_ops());
    t.declare_game("car.damage", SWOFTY_POINT_NOTIFY, swofty::cars::events::Damage::version, g_hit.get());
    g_pit = std::make_unique<GamePoint>("session.pit", "Game_Pitlane_SetVehicleStatus",
                                        reinterpret_cast<void *>(&d_pit_status), game_ops());
    t.declare_game("session.pit", SWOFTY_POINT_NOTIFY, swofty::sessions::events::Pit::version, g_pit.get());
    t.declare_game("session.pit_stop", SWOFTY_POINT_NOTIFY, swofty::sessions::events::PitStop::version, g_pit.get());
    g_setup = std::make_unique<session_points::MidPoint>("car.setup.applied", "site_Car_SetupTuningApplied", &m_setup,
                                                         game_ops());
    t.declare_game("car.setup.applied", SWOFTY_POINT_NOTIFY, swofty::cars::events::SetupApplied::version, g_setup.get());
}

void arm_car_points()
{
    if (!runtime::same_build())
        return;
    g_built->keep(true);
    g_built->install(false, "the core");
    g_reset->keep(true);
    g_reset->install(false, "the core");
    if (row_ready("Car_Upgrades_ApplyToVehicle")) {
        g_replay->keep(true);
        g_replay->install(false, "the core");
    }
    if (row_ready("Game_Pitlane_SetVehicleStatus")) {
        g_pit->keep(true);
        g_pit->install(false, "the core");
    }
    if (row_ready("site_Car_SetupTuningApplied")) {
        g_setup->keep(true);
        g_setup->install(false, "the core");
    }
}

}

namespace {

constexpr const char *LEVERS = "swofty_core.dll (car levers)";
bool g_levers_offered = false;

void engine_torque(SwoftyContext *c)
{
    c->xmm0.f32[0] = swofty_core::hooks::Ops::drive_torque(c->rbx, c->xmm0.f32[0], c->xmm7.f32[0],
                                                          [&] {
                                                              float limiter = 0;
                                                              swofty::mem::read(c->rbx + 0x2528, limiter);
                                                              return limiter;
                                                          }());
}

void ai_grip(SwoftyContext *c)
{
    c->xmm1.f32[0] = swofty_core::hooks::Ops::ai_grip(uint32_t(c->r12), uint8_t(c->rcx), c->xmm1.f32[0]);
}

}

namespace runtime {

std::vector<CoreMixins> car_levers_mixins()
{
    std::vector<CoreMixins> out;
    if (g_levers_offered || !code_ready() || !same_build())
        return out;
    g_levers_offered = true;
    std::vector<SwoftyMixin> mixins;
    if (swofty_core::hooks::row_ready("site_Car_EngineDriveTorque") && swofty::api->resolve("site_Car_EngineDriveTorque"))
        mixins.push_back({"site_Car_EngineDriveTorque", SWOFTY_MIXIN_SITE, 100, (void *)&engine_torque, nullptr});
    else
        swofty::log("[cars] engine.power_scale and engine.limiter_offset_rpm off: needs site_Car_EngineDriveTorque");
    if (swofty_core::hooks::row_ready("site_Ai_TableGripInput") && swofty::api->resolve("site_Ai_TableGripInput"))
        mixins.push_back({"site_Ai_TableGripInput", SWOFTY_MIXIN_SITE, 100, (void *)&ai_grip, nullptr});
    else
        swofty::log("[cars] ai.grip_scale off: needs site_Ai_TableGripInput");
    if (!mixins.empty())
        out.push_back({LEVERS, std::move(mixins)});
    return out;
}

void car_levers_commit()
{
    if (g_levers_offered)
        swofty::log("[cars] car levers %s", core_mixins_active(LEVERS) ? "armed" : "did not install (see the [core] lines)");
}

}

extern "C" __declspec(dllexport) const SwoftyCarOps *SwoftyCoreCarOps(void)
{
    return &swofty_core::hooks::Ops::table;
}

SWOFTY_ON_LOAD
{
    runtime::when_code_ready("live car operations", swofty_core::hooks::setup);
}
