// SPDX-License-Identifier: MIT
#include <cstdio>
#include <fstream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#include <swofty/fake_core.hpp>

#include "../native/src/carphysics.cpp"

namespace json = swofty::json;

static int failures = 0;

static void check(bool ok, const char *what)
{
    std::printf("%s  %s\n", ok ? "ok  " : "FAIL", what);
    failures += !ok;
}

struct Layered {
    uint32_t index, field, op;
    float value;
    int32_t priority;
};

static std::vector<Layered> layered;
static std::vector<uint32_t> removed;
static std::map<std::string, uint32_t> field_ids{{"aero.lift_scale", 1}, {"chassis.ballast_kg", 2}};

static int ops_available() { return 1; }
static int ops_count(uint32_t *n)
{
    *n = 2;
    return SWOFTY_ASSET_OK;
}
static int ops_car(uint32_t index, void **handle, uint32_t *is_ai)
{
    if (handle)
        *handle = nullptr;
    if (is_ai)
        *is_ai = index != 0;
    return SWOFTY_ASSET_OK;
}
static int ops_get(uint32_t, const void *, uint32_t, float *) { return SWOFTY_ASSET_UNAVAILABLE; }
static int ops_set(uint32_t, const void *, uint32_t, float) { return SWOFTY_ASSET_UNAVAILABLE; }
static int ops_field(const char *path, uint32_t *id)
{
    auto it = field_ids.find(path);
    if (it == field_ids.end())
        return SWOFTY_ASSET_INVALID;
    *id = it->second;
    return SWOFTY_ASSET_OK;
}
static int ops_layer(const char *, uint32_t index, uint32_t field, uint32_t op, float value, int32_t priority,
                     uint32_t *token)
{
    layered.push_back({index, field, op, value, priority});
    *token = uint32_t(layered.size());
    return SWOFTY_ASSET_OK;
}
static int ops_unlayer(uint32_t token)
{
    removed.push_back(token);
    return SWOFTY_ASSET_OK;
}
static int ops_refit(uint32_t, const char *) { return SWOFTY_ASSET_UNAVAILABLE; }

static const SwoftyCarOps fake_ops = {2,         ops_available, ops_count, ops_car,   ops_get,
                                      ops_set,   ops_field,     ops_layer, ops_unlayer, ops_refit};

static json::Value read_data(const char *file)
{
    std::ifstream in(file);
    std::stringstream text;
    text << in.rdbuf();
    return json::parse(text.str());
}

static void profile_record()
{
    json::Value d = read_data("registry/car-profile.json");
    const json::Value &p = d["car.profile"]["race_engine"];
    check(p["power_scale"].as_double() == 0.9725, "the race engine is x0.9725");
    check(p["limiter_offset_rpm"].as_int() == -300, "with the limiter 300 rpm lower");
    check(p["ai_grip_scale"].as_double() == 0.87, "and the AI's grip x0.87");
    check(p["when"].size() == 2 && p["when"][0].as_string() == "practice" && p["when"][1].as_string() == "race",
          "outside qualifying");
    check(p["within"][0]["data_group"].as_string() == "f1_career_1990", "in the F1 1990 data group");
}

static void handling_record()
{
    json::Value d = read_data("registry/handling@1.1.json");
    const json::Value &h = d["vehicle.handling"]["ferrari_six_speed"];
    check(h["gearbox"]["ratios"].size() == 6, "six gears");
    check(h["gearbox"]["up_speed_multiple"].size() == 5 && h["gearbox"]["down_speed_multiple"].size() == 5,
          "with a shift-speed multiple per shift");
    check(h.contains("ers") && h["ers"].is_null(), "ERS removed");
    check(h["vtf"]["stock"].as_string().find("ferrari.erp") != std::string::npos, "on the stock Ferrari file");
}

static void live_layers()
{
    swofty::detail::set_car_ops_table(&fake_ops);
    make_layers();
    check(lift.ok(), "the lift layer is made");
    check(ballast.ok(), "the ballast layer is made");
    bool all = false, player = false;
    for (const Layered &l : layered) {
        if (l.field == 1 && l.index == SWOFTY_CAR_ALL && l.op == SWOFTY_CAR_OP_SCALE && l.value == 0.95f)
            all = true;
        if (l.field == 2 && l.index == 0 && l.op == SWOFTY_CAR_OP_ADD && l.value == 12.0f)
            player = true;
    }
    check(all, "lift x0.95 reaches every car");
    check(player && layered.size() == 2, "12 kg of ballast reaches only the player's car");
    size_t before = layered.size();
    make_layers();
    check(layered.size() == before, "a second Built adds nothing");
    lift.release();
    ballast.release();
    check(removed.size() == 2, "dropping the layers removes them");
    swofty::detail::set_car_ops_table(nullptr);
}

static void parts()
{
    std::vector<std::string> added;
    cars::events::PartsRegister e{};
    e.add_fn = [](void *user, const char *node, float at, bool) -> int32_t {
        static_cast<std::vector<std::string> *>(user)->push_back(std::string(node) + "@" + std::to_string(int(at * 100)));
        return SWOFTY_ASSET_OK;
    };
    e.user = &added;
    add_wing_pieces(e);
    check(added.size() == 1 && added[0] == "frontwing_fl_03@90", "a front wing piece is added");
}

int main()
{
    swofty::fake::Core core{"mod.carphysics"};
    core.run_register();
    bool built = false, parts_register = false;
    json::Value rec = core.recording();
    const json::Value &subs = rec["subscriptions"];
    for (size_t i = 0; i < subs.size(); i++) {
        built |= subs[i]["point"].as_string() == "car.built";
        parts_register |= subs[i]["point"].as_string() == "car.parts.register";
    }
    check(built, "subscribes to car.built");
    check(parts_register, "subscribes to car.parts.register");
    profile_record();
    handling_record();
    live_layers();
    parts();
    return failures ? 1 : 0;
}
