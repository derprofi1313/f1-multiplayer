// SPDX-License-Identifier: MIT
#pragma once
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "../../../../native/src/core/registry/declare_all.hpp"
#include "../../../../native/src/core/registry/older.hpp"
#include "../../../../native/src/core/registry/table.hpp"

// Run from the addon folder (templates/std/test.sh does): the repository root is three folders up.
namespace hockenheim_test {

namespace reg = swofty_core::registry;
using swofty::json::Value;

inline int failures = 0;
inline const std::string root = "../../..";
inline const std::vector<std::string> sets = {"f1_2025", "sprint_f1_2025", "f1_2026", "sprint_f1_2026", "classic"};

inline void check(bool ok, const std::string &what)
{
    std::printf("%s  %s\n", ok ? "ok  " : "FAIL", what.c_str());
    failures += !ok;
}

inline std::string slurp(const std::string &path)
{
    std::ifstream f(path, std::ios::binary);
    std::stringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

inline Value tyres(const char *softer, const char *medium, const char *harder)
{
    return Value::object({{"softer", Value(softer)}, {"medium", Value(medium)}, {"harder", Value(harder)}});
}

// A store with every system registry declared and what the core seeds at boot: a venue per entry of
// packages/sdk/std/data/venues.json, the stock track that provides each, and Austria's data block in each set.
inline reg::Store &fresh_store(const std::vector<std::string> &order)
{
    reg::reset_store();
    auto &st = reg::store();
    st.set_order(order);
    st.set_validator(&reg::schema_validator());
    reg::declare_system_registries([&](const std::string &name, const std::string &schema) {
        if (st.declared(name))
            return false;
        return st.declare(name, swofty::json::parse(schema), false, "") == 0;
    });
    reg::older::declare(st);
    Value venues = swofty::json::parse(slurp(root + "/packages/sdk/std/data/venues.json"))["venues"];
    for (auto &[key, v] : *venues.if_object()) {
        st.add_base("track.venue", key,
                    Value::object({{"name", v["name"]}, {"country", v["country"]}, {"track", Value()}}));
        if (v["stock"].is_string())
            st.add_base("track", std::string(v["stock"].as_string()),
                        Value::object({{"code", v["stock"]},
                                       {"time_trial", Value(true)},
                                       {"provides", Value::array({Value("venue/" + key)})}}));
    }
    for (const auto &set : sets)
        st.add_base("track.data", "austria/" + set,
                    Value::object({{"track", Value("game/austria")},
                                   {"data_set", Value(set)},
                                   {"laps", Value(71)},
                                   {"fuel", Value::object({{"burn_per_lap", Value(1.5)}})},
                                   {"tyres", set == "classic" ? tyres("C5", "C4", "C3") : tyres("C3", "C2", "C1")},
                                   {"pit", Value::object({{"speed_limit", Value(60)},
                                                          {"stop_time_dry", Value(21.0)},
                                                          {"stop_time_wet", Value(22.0)}})}}));
    st.stock_loaded("track", true);
    st.stock_loaded("track.data", true);
    return st;
}

// What the core does at boot for a record track: its data block in each set is a variant of its base's.
inline void core_blocks(reg::Store &st)
{
    for (const auto &set : sets) {
        reg::Change c;
        c.patch = Value::object({{"track", Value("mod.hockenheim/hockenheim")}});
        check(st.variant("mod.hockenheim", "track.data", "hockenheim/" + set, "game/austria/" + set, c, nullptr,
                         "core") == SWOFTY_ASSET_OK,
              "the core's " + set + " block for mod.hockenheim/hockenheim is a variant of game/austria/" + set);
    }
}

inline bool same_tyres(const Value &v, const char *softer, const char *medium, const char *harder)
{
    return v["softer"].as_string() == softer && v["medium"].as_string() == medium && v["harder"].as_string() == harder;
}

inline void check_hockenheim(reg::Store &st, const std::string &form)
{
    auto t = st.get("track", "mod.hockenheim/hockenheim");
    check(t.status == reg::Status::ok, form + ": mod.hockenheim/hockenheim is valid");
    check(t.value["code"].as_string() == "hockenheim" && t.value["base"].as_string() == "game/austria",
          form + ": code hockenheim, base game/austria");
    check(t.value["provides"].size() == 1 && t.value["provides"][0].as_string() == "venue/hockenheimring",
          form + ": it provides venue/hockenheimring");
    check(t.value["files"].as_string() == "track/", form + ": its files are track/");
    check(!t.value["time_trial"].as_bool(false), form + ": no Time Trial");

    auto venue = st.get("track.venue", "venue/hockenheimring");
    check(venue.status == reg::Status::ok && venue.value["track"].as_string() == "mod.hockenheim/hockenheim",
          form + ": venue/hockenheimring resolves to it");

    for (const auto &set : sets) {
        auto d = st.get("track.data", "mod.hockenheim/hockenheim/" + set);
        bool ok = d.status == reg::Status::ok && d.value["track"].as_string() == "mod.hockenheim/hockenheim" &&
                  d.value["data_set"].as_string() == set && d.value["laps"].as_int() == 67 &&
                  d.value["fuel"]["burn_per_lap"].as_double() > 1.619 &&
                  d.value["fuel"]["burn_per_lap"].as_double() < 1.621 && d.value["pit"]["speed_limit"].as_int() == 80 &&
                  d.value["pit"]["stop_time_dry"].as_double() == 21.0;
        check(ok, form + ": " + set + " block: 67 laps, fuel 1.62, pit lane 80 and Austria's stop times");
        bool compounds = set == "classic" ? same_tyres(d.value["tyres"], "C5", "C4", "C3")
                                          : same_tyres(d.value["tyres"], "C4", "C3", "C2");
        check(compounds, form + ": " + set + (set == "classic" ? " keeps Austria's compounds" : " has C4/C3/C2"));
    }
    auto austria = st.get("track.data", "game/austria/f1_2026");
    check(austria.status == reg::Status::ok && austria.value["laps"].as_int() == 71 &&
              same_tyres(austria.value["tyres"], "C3", "C2", "C1"),
          form + ": Austria's own block is unchanged");
}

}
