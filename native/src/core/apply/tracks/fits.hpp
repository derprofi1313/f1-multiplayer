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
// 1.5 run it, D5; a record track when its record says so; none whose facts say time_trial false), grid slots for
// the box's cars clamped to the install's grid size, an AI folder for every car and the track's
// ai_track_difficulties for its package. Before those, the world rule: a track whose files hold none of the box's
// worlds while other tracks hold one is left out (the game loads a world the track hasn't got). A rule without facts
// passes and names what it lacked in no_facts. A box with no content.package record (an addon's era box) is made
// from the database: the teams and cars of its entrant list, the list's season and the setup mode that offers it.
// Pure: host tests build boxes, bases and facts by hand.

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
    bool derived = false;
    std::string unread;
    std::vector<Car> ai;
};

enum class Rule { none, time_trial, world, grid, ai, difficulties, offered };

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
    int grid = -1;
    size_t here = 0;
    std::string lacking;
    if (worlds.empty())
        f.no_facts.push_back("no track layer for setup mode " + box.setup_mode);
    for (auto &w : worlds) {
        auto it = tf->worlds.find(w);
        if (it == tf->worlds.end()) {
            if (lacking.empty() && facts->any_track_holds(w))
                lacking = w;
            f.no_facts.push_back("no " + w + " world");
            continue;
        }
        ++here;
        if (it->second.grid >= 0 && (grid < 0 || it->second.grid < grid))
            grid = it->second.grid;
    }
    if (!here && !lacking.empty())
        return fail(f, Rule::world, 0, 0, lacking,
                    "its files hold no " + lacking + " world, which the box's " + box.setup_mode + " layer loads");
    int cars = box.cars;
    if (box.derived && facts->grid_size > 0 && cars < 0)
        cars = facts->grid_size;
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
    case Rule::world:
        d.body = "It has no " + fit.what + " world, the one " + boxname + " loads.";
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

using IntRead = std::function<std::optional<int64_t>(const std::string &key)>;
using RowFind = std::function<std::optional<uint64_t>(const std::string &table, const std::string &label)>;

struct ListDb {
    TextRead text;
    IntRead integer;
    RowFind row;
};

inline constexpr const char *DB_LIST_TABLE = "entrants.entrant_lists";
inline constexpr const char *DB_ENTRANT_MODES = "entrants.entrant_setup_modes";
inline constexpr const char *DB_GAME_MODES = "game_setup.game_setup_modes";
inline constexpr const char *HOST_SETUP_MODE = "swofty_f1";
inline constexpr int MAX_LIST_DEPTH = 8;
inline constexpr int64_t MAX_LIST_ENTRANTS = 64, MAX_SETUP_MODES = 256;

inline std::vector<std::string> db_names(const std::string &text)
{
    std::vector<std::string> out;
    size_t at = 0;
    while (at <= text.size()) {
        size_t comma = text.find(',', at);
        std::string part = text.substr(at, comma == std::string::npos ? std::string::npos : comma - at);
        size_t b = part.find_first_not_of(" \t"), e = part.find_last_not_of(" \t");
        if (b != std::string::npos)
            out.push_back(part.substr(b, e - b + 1));
        if (comma == std::string::npos)
            break;
        at = comma + 1;
    }
    return out;
}

// A row by index, the path the game itself reads (<table>.<row>_<i>): rows the core or an addon adds have no
// [_metadata_=...] key.
inline std::string db_row_path(const char *table, const char *row, uint64_t index)
{
    return std::string(table) + "." + row + "_" + std::to_string(index);
}

inline std::optional<std::string> list_row(const ListDb &db, const std::string &list)
{
    auto i = db.row ? db.row(DB_LIST_TABLE, list) : std::nullopt;
    if (!i)
        return std::nullopt;
    return db_row_path(DB_LIST_TABLE, "entrant_list", *i);
}

// An entrant list's entrants as the game's parser reads them: its entrant_list_references first, then its own
// _entrant_instancecount_ rows. teams: each team label once. False when a list, a reference or a count can't be read.
inline bool walk_list(const ListDb &db, const std::string &list, int depth, std::vector<std::string> &seen,
                      std::vector<std::string> &teams, int &cars)
{
    if (depth > MAX_LIST_DEPTH)
        return false;
    if (facts_has(seen, list))
        return true;
    seen.push_back(list);
    auto row = list_row(db, list);
    if (!row)
        return false;
    for (auto &ref : db_names(db_text(db.text, *row + ".entrant_list_references")))
        if (!walk_list(db, ref, depth + 1, seen, teams, cars))
            return false;
    auto n = db.integer ? db.integer(*row + "._entrant_instancecount_") : std::nullopt;
    if (!n || *n < 0 || *n > MAX_LIST_ENTRANTS)
        return false;
    for (int64_t k = 0; k < *n; ++k) {
        std::string team = db_text(db.text, *row + ".entrant_" + std::to_string(k) + ".team");
        ++cars;
        if (!team.empty() && !facts_has(teams, team))
            teams.push_back(team);
    }
    return true;
}

// The game setup mode a box's entrant list runs under: a mode whose entrant setup modes list it. An addon's own entrant
// setup mode comes before the host's, which every era box's list is in (F1 2012 Season's bdx_s5_2012 runs under
// bdx_2012_mode); through the host's, the host mode itself comes first. Empty when no mode lists it.
inline std::string list_setup_mode(const ListDb &db, const std::string &list, std::string_view host = HOST_SETUP_MODE)
{
    auto n = db.integer ? db.integer(std::string(DB_GAME_MODES) + "._game_setup_mode_instancecount_") : std::nullopt;
    if (!n || *n <= 0 || *n > MAX_SETUP_MODES || !db.row)
        return {};
    std::string host_mode, via_host;
    for (int64_t i = 0; i < *n; ++i) {
        std::string row = db_row_path(DB_GAME_MODES, "game_setup_mode", uint64_t(i));
        std::string label = db_text(db.text, row + "._metadata_");
        if (label.empty())
            continue;
        for (auto &mode : db_names(db_text(db.text, row + ".entrant_setup_modes"))) {
            auto j = db.row(DB_ENTRANT_MODES, mode);
            if (!j || !facts_has(db_names(db_text(db.text, db_row_path(DB_ENTRANT_MODES, "entrant_setup_mode", *j) +
                                                                ".entrant_lists")),
                                 list))
                continue;
            if (mode != host)
                return label;
            if (label == host)
                host_mode = label;
            else if (via_host.empty())
                via_host = label;
        }
    }
    return host_mode.empty() ? via_host : host_mode;
}

// A box with no content.package record, from the database: the setup mode that offers its list, the list's season,
// and the teams and cars of its entrants. With no setup mode found the box keeps swofty_f1 and unknown counts.
inline BoxInfo box_from_list(const std::string &list, const ListDb &db)
{
    BoxInfo b;
    b.package = "game/" + list;
    b.entrant_list = list;
    b.derived = true;
    std::string mode = list_setup_mode(db, list);
    if (!mode.empty())
        b.setup_mode = mode;
    else
        b.unread = "no setup mode offers the list";
    auto row = list_row(db, list);
    if (!row) {
        b.unread += std::string(b.unread.empty() ? "" : "; ") + "no entrant list row labelled " + list;
        return b;
    }
    if (!mode.empty())
        b.season = db_text(db.text, *row + ".season");
    std::vector<std::string> seen, teams;
    int cars = 0;
    if (!walk_list(db, list, 0, seen, teams, cars)) {
        b.unread += std::string(b.unread.empty() ? "" : "; ") + "an entrant count or reference of " + list + " can't be read";
        return b;
    }
    b.teams = int(teams.size());
    b.cars = cars;
    return b;
}

}
