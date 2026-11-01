#pragma once

#include "base.hpp"
#include "facts_file.hpp"

#include <swofty/json.hpp>

#include <algorithm>
#include <cstdint>
#include <functional>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

// Whether a track fits a box (the frozen fit rules of work/std16/TRACKS-CONTRACTS.md C2), and the guard's
// decision. A box is a content.package (or a stock box with no record) in a mode. The world is the one the box's
// setup mode loads at the track: the facts' layers whose setup_modes hold it (the launcher scans the database the
// game reads, runtime database edits included, so Classic Seasons' swofty_f1 layer is there), narrowed to the
// box's game season when a layer names it; several worlds: the smallest room among them. Rules, in this order: Time Trial support (stock tracks and tracks made for
// 1.5 run it, D5; a record track when its record says so; none whose facts say time_trial false), garages for every
// team, grid slots for the box's cars clamped to the install's grid size, an AI folder for every car and the track's
// ai_track_difficulties for its package. A rule without facts passes and names what it lacked in no_facts. Pure:
// host tests build boxes, bases and facts by hand.

namespace swofty_core::apply::tracks {

enum class Mode { grand_prix, time_trial };

inline const char *mode_name(Mode m)
{
    return m == Mode::time_trial ? "time_trial" : "grand_prix";
}

struct Car {
    std::string team, folder, asset;
};

struct BoxInfo {
    std::string package, entrant_list, title;
    std::string setup_mode = "swofty_f1";
    std::string season;
    int teams = -1, cars = -1;
    std::vector<Car> ai;
};

enum class Rule { none, time_trial, garages, grid, ai, difficulties, offered };

struct Fit {
    bool ok = true;
    Rule rule = Rule::none;
    int have = 0, need = 0;
    std::string what, reason;
    std::vector<std::string> no_facts;
};

inline bool facts_has(const std::vector<std::string> &v, std::string_view s)
{
    return std::find(v.begin(), v.end(), s) != v.end();
}

// The worlds the box's setup mode loads, in layer order, without repeats.
inline std::vector<std::string> worlds_for(const Facts &facts, const BoxInfo &box)
{
    std::vector<const Layer *> modes;
    for (auto &l : facts.layers)
        if (facts_has(l.setup_modes, box.setup_mode))
            modes.push_back(&l);
    bool by_season = false;
    if (!box.season.empty())
        for (auto *l : modes)
            by_season = by_season || facts_has(l->seasons, box.season);
    std::vector<std::string> out;
    for (auto *l : modes)
        if ((!by_season || facts_has(l->seasons, box.season)) && !l->world.empty() && !facts_has(out, l->world))
            out.push_back(l->world);
    return out;
}

inline Fit fail(Fit f, Rule rule, int have, int need, std::string what, std::string reason)
{
    f.ok = false;
    f.rule = rule;
    f.have = have;
    f.need = need;
    f.what = std::move(what);
    f.reason = std::move(reason);
    return f;
}

// Time Trial support: a stock track or a track made for 1.5 (D5), a record track whose record says time_trial;
// never a track whose facts say it holds no world Time Trial loads. why: the facts' reason, when they give one.
inline bool runs_time_trial(const BaseTrack &track, const Facts *facts, std::string &why)
{
    const TrackFacts *tf = facts && facts->loaded ? facts->track(track.code) : nullptr;
    if (tf && !tf->time_trial) {
        why = tf->time_trial_why;
        return false;
    }
    return track.kind != Kind::record || track.time_trial;
}

inline Fit fits(const BoxInfo &box, const BaseTrack &track, Mode mode, const Facts *facts)
{
    Fit f;
    std::string tt_why;
    if (mode == Mode::time_trial && !runs_time_trial(track, facts, tt_why))
        return fail(f, Rule::time_trial, 0, 0, tt_why,
                    "no Time Trial support" + (tt_why.empty() ? std::string() : " (" + tt_why + ")"));
    if (!facts || !facts->loaded) {
        f.no_facts.push_back("no track-facts.json");
        return f;
    }
    const TrackFacts *tf = facts->track(track.code);
    if (!tf) {
        f.no_facts.push_back("no facts for " + track.code);
        return f;
    }
    std::vector<std::string> worlds = worlds_for(*facts, box);
    int garages = -1, grid = -1;
    if (worlds.empty())
        f.no_facts.push_back("no track layer for setup mode " + box.setup_mode);
    for (auto &w : worlds) {
        auto it = tf->worlds.find(w);
        if (it == tf->worlds.end()) {
            f.no_facts.push_back("no " + w + " world");
            continue;
        }
        if (it->second.garages >= 0 && (garages < 0 || it->second.garages < garages))
            garages = it->second.garages;
        if (it->second.grid >= 0 && (grid < 0 || it->second.grid < grid))
            grid = it->second.grid;
    }
    if (box.teams < 0)
        f.no_facts.push_back("the box's teams");
    else if (garages >= 0 && garages < box.teams)
        return fail(f, Rule::garages, garages, box.teams, {},
                    "its " + box.setup_mode + " world has " + std::to_string(garages) + " garages; the box has " +
                        std::to_string(box.teams) + " teams");
    int cars = box.cars;
    if (cars >= 0 && facts->grid_size > 0)
        cars = std::min(cars, facts->grid_size);
    if (cars < 0)
        f.no_facts.push_back("the box's cars");
    else if (grid >= 0 && grid < cars)
        return fail(f, Rule::grid, grid, cars, {},
                    "its " + box.setup_mode + " world has " + std::to_string(grid) + " grid slots; the box runs " +
                        std::to_string(cars) + " cars");
    for (auto &car : box.ai) {
        if (car.folder.empty() || car.asset.empty()) {
            f.no_facts.push_back("the AI folder of " + car.team);
            continue;
        }
        std::string folder = car.folder + "/" + car.asset;
        if (tf->has_ai && !facts_has(tf->ai, folder))
            return fail(f, Rule::ai, 0, 0, car.team, "no AI data for " + folder + " (" + car.team + ")");
        if (tf->has_difficulties && !facts_has(tf->ai_difficulties, car.folder))
            return fail(f, Rule::difficulties, 0, 0, car.folder, "no ai_track_difficulties for " + car.folder);
    }
    if (!tf->has_ai && !box.ai.empty())
        f.no_facts.push_back("the track's AI folders");
    return f;
}

struct Decision {
    bool ok = true;
    std::string title, body, line;
};

// The guard's answer for one confirmed track: the popup of docs/std/systems/tracks.md when it doesn't fit.
inline Decision decide(const BaseTrack *track, const BoxInfo &box, Mode mode, const Fit &fit, std::string_view track_label)
{
    Decision d;
    std::string name = !track_label.empty() ? std::string(track_label) : track && !track->name.empty() ? track->name
                                                                            : track ? track->code
                                                                                    : std::string("this track");
    std::string boxname = box.title.empty() ? box.package : box.title;
    std::string where = "[tracks] guard: " + (track ? track->id : name) + " in " + box.package + " (" + mode_name(mode) + ")";
    if (!track) {
        d.line = where + ": not in the track base; allowed";
        return d;
    }
    if (fit.ok) {
        d.line = where + ": fits" + (fit.no_facts.empty() ? std::string() : " (no facts: " + fit.no_facts.front() + ")");
        return d;
    }
    d.ok = false;
    d.title = name + " can't run in " + boxname;
    switch (fit.rule) {
    case Rule::garages:
        d.body = "Its garages hold " + std::to_string(fit.have) + " teams. " + boxname + " has " + std::to_string(fit.need) + ".";
        break;
    case Rule::grid:
        d.body = "Its grid holds " + std::to_string(fit.have) + " cars. " + boxname + " runs " + std::to_string(fit.need) + ".";
        break;
    case Rule::ai:
        d.body = "It has no AI data for the " + fit.what + " car.";
        break;
    case Rule::difficulties:
        d.body = "It has no AI difficulty data for " + fit.what + " cars.";
        break;
    case Rule::time_trial:
        d.body = "It has no Time Trial support.";
        break;
    case Rule::offered:
        d.body = boxname + " doesn't offer it.";
        break;
    case Rule::none:
        d.body = fit.reason;
        break;
    }
    d.line = where + ": refused (" + fit.reason + ")";
    return d;
}

using Value = swofty::json::Value;
using Lookup = std::function<const Value *(const std::string &registry, const std::string &id)>;
using TextRead = std::function<std::optional<std::string>(const std::string &path)>;
using CellRead =
    std::function<std::optional<int64_t>(const std::string &table, const std::string &label, const std::string &field)>;

inline constexpr const char *DB_VEHICLES = "entrants.vehicles.vehicle";
inline constexpr const char *DB_SETUP_MODES = "game_setup.game_setup_modes";

// A setup mode's stat id from its row, found by label, -1 when the database has none. stat_id is an int record, and
// a row the core adds (swofty_f1) has no [_metadata_=...] key, so a text read of that path always missed it.
inline int setup_mode_stat_of(const CellRead &read, const std::string &setup_mode)
{
    if (!read || setup_mode.empty())
        return -1;
    auto v = read(DB_SETUP_MODES, setup_mode, "stat_id");
    return v && *v >= 0 && *v <= 255 ? int(*v) : -1;
}

inline constexpr const char *DB_PACKAGES = "entrants.vehicles.vehicle_packages.vehicle_package";
inline constexpr const char *DB_LISTS = "entrants.entrant_lists.entrant_list";

inline std::string ref_key(std::string_view id)
{
    auto slash = id.rfind('/');
    return std::string(slash == std::string_view::npos ? id : id.substr(slash + 1));
}

inline std::string text_of(const Value &t)
{
    if (t.is_string())
        return std::string(t.as_string());
    if (t.is_object() && t["literal"].is_string())
        return std::string(t["literal"].as_string());
    return {};
}

inline std::string db_text(const TextRead &read, const std::string &path)
{
    if (!read)
        return {};
    auto v = read(path);
    return v ? *v : std::string();
}

// One team's car: its vehicle record's package folder and AI asset, else the database rows the vehicle's label
// (written by the content records at boot) or its donor holds.
inline Car car_of(const std::string &team, const Value &t, const Lookup &lookup, const TextRead &read)
{
    Car car{team, {}, {}};
    if (!t["vehicle"].is_string())
        return car;
    std::string vid(t["vehicle"].as_string());
    const Value *v = lookup ? lookup("vehicle", vid) : nullptr;
    std::string label = v && (*v)["label"].is_string() ? std::string((*v)["label"].as_string()) : ref_key(vid);
    std::string donor = v && (*v)["donor"].is_string() ? ref_key((*v)["donor"].as_string()) : std::string();
    if (v && (*v)["ai"]["asset_filename"].is_string() && (*v)["ai"]["asset_filename"].as_string() != "auto")
        car.asset = std::string((*v)["ai"]["asset_filename"].as_string());
    auto row = [](const std::string &l) { return std::string(DB_VEHICLES) + "[_metadata_=" + l + "]"; };
    if (car.asset.empty())
        car.asset = db_text(read, row(label) + ".asset_filename");
    if (car.asset.empty() && !donor.empty())
        car.asset = db_text(read, row(donor) + ".asset_filename");
    std::string pkg;
    if (v && (*v)["package"].is_string()) {
        std::string pid((*v)["package"].as_string());
        const Value *p = lookup ? lookup("vehicle.package", pid) : nullptr;
        if (p && (*p)["ai_package_folder"].is_string())
            car.folder = std::string((*p)["ai_package_folder"].as_string());
        pkg = ref_key(pid);
        if (car.folder.empty() && p && (*p)["donor"].is_string())
            car.folder = db_text(read, std::string(DB_PACKAGES) + "[_metadata_=" + ref_key((*p)["donor"].as_string()) +
                                           "].ai_package_folder");
    }
    if (pkg.empty())
        pkg = db_text(read, row(label) + ".vehicle_package");
    if (pkg.empty() && !donor.empty())
        pkg = db_text(read, row(donor) + ".vehicle_package");
    if (car.folder.empty() && !pkg.empty())
        car.folder = db_text(read, std::string(DB_PACKAGES) + "[_metadata_=" + pkg + "].ai_package_folder");
    return car;
}

// A box from its effective content.package record. A team with no seats listed runs two cars.
inline BoxInfo box_from_package(const std::string &id, const Value &pkg, const Lookup &lookup, const TextRead &read)
{
    BoxInfo b;
    b.package = id;
    b.entrant_list = std::string(pkg["entrant_list"].as_string());
    b.title = text_of(pkg["title"]);
    if (pkg["season"].is_int() && (b.title.empty() || b.title.rfind("lng_", 0) == 0))
        b.title = "F1 " + std::to_string(pkg["season"].as_int());
    if (pkg["setup_mode"].is_string())
        b.setup_mode = std::string(pkg["setup_mode"].as_string());
    b.season = db_text(read, std::string(DB_LISTS) + "[_metadata_=" + b.entrant_list + "].season");
    auto teams = pkg["teams"].if_array();
    if (!teams)
        return b;
    b.teams = int(teams->size());
    b.cars = 0;
    for (auto &t : *teams) {
        auto seats = t["drivers"].if_array();
        b.cars += seats && !seats->empty() ? int(seats->size()) : 2;
        b.ai.push_back(car_of(std::string(t["key"].as_string()), t, lookup, read));
    }
    return b;
}

}
