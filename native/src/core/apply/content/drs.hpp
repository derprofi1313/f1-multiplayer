#pragma once

#include "gvdb.hpp"
#include "lower.hpp"

#include <cstdio>
#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

// rules.drs: each record becomes one drs.drs_allowed_lap group row at boot, a copy of the stock f1_2024 group with
// its RaceStart, RedFlagRestart and FullSafetyCarEnd lap_number set. The game keeps every group in a map keyed by
// its label and reads the current game setup mode's group at session load, at a full safety car end and at a red
// flag restart (re/db functions Game_DRS_*). A box names its rule in content.package rules.drs; the era picker
// points swofty_f1's group at the rule's label when the box is pressed (drs_pick). Pure, for the host tests.

namespace swofty_core::apply::content {

inline constexpr const char *DRS_TABLE = "drs.drs_allowed_lap";
inline constexpr const char *DRS_GROUPS = "drs.drs_allowed_lap.drs_allowed_lap_group";
inline constexpr const char *DRS_STOCK_GROUP = "f1_2024";
inline constexpr std::size_t DRS_LABEL_MAX = 63;
inline constexpr std::size_t DRS_KEY_MAX = 40;
inline constexpr int64_t DRS_LAP_MAX = 9999;

struct DrsLaps {
    int race_start = 1, red_flag_restart = 1, safety_car_end = 1;
};

struct DrsGroup {
    std::string label;
    DrsLaps laps;
};

inline std::string drs_label(std::string_view id)
{
    std::string key;
    for (char c : key_of(id)) {
        if (key.size() == DRS_KEY_MAX)
            break;
        bool plain = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_';
        key += plain ? c : '_';
    }
    char hash[9];
    std::snprintf(hash, sizeof hash, "%08x", unsigned(gvdb::fnv64(id) & 0xffffffffu));
    return "swofty_drs_" + key + "_" + hash;
}

inline std::optional<DrsLaps> drs_laps(const Value &rec, std::string *why)
{
    DrsLaps laps;
    struct Field {
        const char *name;
        int *out;
    };
    for (Field f : {Field{"race_start", &laps.race_start}, Field{"red_flag_restart", &laps.red_flag_restart},
                    Field{"safety_car_end", &laps.safety_car_end}}) {
        const Value &v = rec[f.name];
        if (v.is_null())
            continue;
        if (!v.is_int() || v.as_int() < 0 || v.as_int() > DRS_LAP_MAX) {
            if (why)
                *why = std::string(f.name) + " is a lap number from 0 to " + std::to_string(DRS_LAP_MAX);
            return std::nullopt;
        }
        *f.out = int(v.as_int());
    }
    return laps;
}

inline std::string drs_row(const std::string &label, const char *lap)
{
    return std::string(DRS_GROUPS) + "[_metadata_=" + label + "].drs_allowed_lap[_metadata_=" + lap + "]";
}

inline void lower_drs_rule(const std::string &id, const Value &rec, Plan &plan)
{
    plan.owner = addon_of(id);
    std::string why;
    auto laps = drs_laps(rec, &why);
    if (!laps) {
        plan.lines.push_back("rules.drs " + id + ": " + why + "; nothing applied");
        plan.refused++;
        return;
    }
    std::string label = drs_label(id);
    push_row(plan, DRS_TABLE, std::string(DRS_GROUPS) + "[_metadata_=" + DRS_STOCK_GROUP + "]",
             Value::object({{"_metadata_", label}}));
    push_op(plan, set_op(drs_row(label, "RaceStart"), "lap_number", std::to_string(laps->race_start)));
    push_op(plan, set_op(drs_row(label, "RedFlagRestart"), "lap_number", std::to_string(laps->red_flag_restart)));
    push_op(plan, set_op(drs_row(label, "FullSafetyCarEnd"), "lap_number", std::to_string(laps->safety_car_end)));
    plan.lines.push_back("rules.drs " + id + ": group " + label + " from " + DRS_STOCK_GROUP + ", race start lap " +
                         std::to_string(laps->race_start) + ", red flag restart " + std::to_string(laps->red_flag_restart) +
                         ", safety car end " + std::to_string(laps->safety_car_end));
}

struct DrsPick {
    std::string package, rule;
    std::optional<DrsGroup> group;
    std::string line;
};

// The DRS group for the box whose entrant list was pressed: the last content.package (in the order given) with that
// entrant list, its rules.drs, and that rule's group if the boot apply added it.
inline DrsPick drs_pick(const std::string &entrant_list, const std::vector<std::pair<std::string, const Value *>> &packages,
                        const std::map<std::string, DrsGroup> &groups)
{
    DrsPick p;
    const Value *found = nullptr;
    for (auto &[id, rec] : packages)
        if (rec && (*rec)["entrant_list"].is_string() && (*rec)["entrant_list"].as_string() == entrant_list) {
            p.package = id;
            found = rec;
        }
    if (!found) {
        p.line = "box " + entrant_list + ": no content.package; " + "swofty_f1 keeps its own DRS group";
        return p;
    }
    const Value &drs = (*found)["rules"]["drs"];
    if (!drs.is_string()) {
        p.line = "box " + entrant_list + " (" + p.package + "): no DRS rule; swofty_f1 keeps its own DRS group";
        return p;
    }
    p.rule = std::string(drs.as_string());
    auto it = groups.find(p.rule);
    if (it == groups.end()) {
        p.line = "box " + entrant_list + " (" + p.package + "): rules.drs " + p.rule +
                 " was not added at boot; swofty_f1 keeps its own DRS group";
        return p;
    }
    p.group = it->second;
    const DrsLaps &l = it->second.laps;
    p.line = "box " + entrant_list + " (" + p.package + "): swofty_f1 DRS group " + it->second.label + " (rules.drs " +
             p.rule + ": race start lap " + std::to_string(l.race_start) + ", red flag restart " +
             std::to_string(l.red_flag_restart) + ", safety car end " + std::to_string(l.safety_car_end) + ")";
    return p;
}

}
