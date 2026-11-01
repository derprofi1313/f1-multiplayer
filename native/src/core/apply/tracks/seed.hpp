#pragma once

#include "lower.hpp"

#include "../ids.hpp"

#include <map>
#include <set>
#include <string>
#include <vector>

// What the core seeds for the tracks registries (contract C1): game/<code> for every tracks.Track row (rows without
// assets are dormant: records that slot can name, offered nowhere, providing nothing), venue/<key> for every venue
// of venues.inc with the stock track that provides it, game/<code>/<set> for every TrackData block, and the tracks
// made for 1.5 from the launcher's older-tracks.json (contract C3). Pure: the database is a Db, the store is filled
// by the caller, so host tests check every value.

namespace swofty_core::apply::tracks {

struct VenueSeed {
    const char *key, *name, *country, *stock;
};

#include "venues.inc"

inline constexpr const char *DORMANT[] = {"paul_ricard", "portimao",    "sochi",           "hanoi",    "texas_nat",
                                          "suzuka_east", "bahrain_pad", "silverstone_int", "bahrain24"};

inline bool is_dormant(std::string_view code)
{
    for (const char *d : DORMANT)
        if (code == d)
            return true;
    return false;
}

inline const VenueSeed *stock_venue(std::string_view code)
{
    if (is_dormant(code))
        return nullptr;
    for (const VenueSeed &v : VENUE_SEEDS)
        if (*v.stock && code == v.stock)
            return &v;
    return nullptr;
}

struct StockRow {
    std::string code;
    int64_t stat_id = -1;
};

struct Block {
    std::string code, set;
    Value value;
};

struct Scan {
    std::vector<StockRow> rows;
    std::vector<Block> blocks;
    std::vector<std::string> sets;
    std::map<int64_t, std::string> used;
};

inline std::optional<std::string> text_at(const Db &db, const Key &row_key, std::string_view path)
{
    return db.text(field(row_key, path).h);
}

inline std::optional<int64_t> int_at(const Db &db, const Key &row_key, std::string_view path)
{
    return db.int_of(field(row_key, path).h);
}

inline std::optional<double> number_at(const Db &db, const Key &row_key, std::string_view path)
{
    auto t = text_at(db, row_key, path);
    if (!t || t->empty())
        return std::nullopt;
    std::string s = *t;
    for (char &c : s)
        if (c == ',')
            c = '.';
    char *end = nullptr;
    double v = std::strtod(s.c_str(), &end);
    if (!end || *end)
        return std::nullopt;
    return v;
}

inline Value number_value(double v)
{
    if (v == double(int64_t(v)) && v > -1e15 && v < 1e15)
        return Value(int64_t(v));
    return Value(v);
}

// A TrackData block as a track.data value: the fields the header names, from the rows the block holds.
inline Value block_value(const Db &db, const Key &block, const std::string &track_id, const std::string &set)
{
    Value v = Value::object({{"track", Value(track_id)}, {"data_set", Value(set)}});
    if (auto laps = text_at(db, block, "TrackGeneralData_0.number_of_laps")) {
        std::string first = laps->substr(0, laps->find(','));
        char *end = nullptr;
        long long n = std::strtoll(first.c_str(), &end, 10);
        if (end && end != first.c_str() && n >= 0)
            v["laps"] = Value(int64_t(n));
    }
    if (auto fuel = number_at(db, block, "TrackGeneralData_0.fuel_burn_per_lap"))
        v["fuel"] = Value::object({{"burn_per_lap", number_value(*fuel)}});
    Value tyres = Value::object();
    for (auto [name, path] : {std::pair<const char *, const char *>{"softer", "TyreData_0.compound_softer"},
                              {"medium", "TyreData_0.compound_medium"},
                              {"harder", "TyreData_0.compound_harder"}})
        if (auto t = text_at(db, block, path))
            tyres[name] = Value(*t);
    if (tyres.size())
        v["tyres"] = std::move(tyres);
    Value pit = Value::object();
    if (auto s = int_at(db, block, "TrackPitData_0.pitSpeedLimit"))
        pit["speed_limit"] = Value(*s);
    if (auto d = number_at(db, block, "TrackPitData_0.pitStopTime_Dry"))
        pit["stop_time_dry"] = number_value(*d);
    if (auto w = number_at(db, block, "TrackPitData_0.pitStopTime_Wet"))
        pit["stop_time_wet"] = number_value(*w);
    if (pit.size())
        v["pit"] = std::move(pit);
    Value names = Value::object();
    for (auto [name, path] : {std::pair<const char *, const char *>{"title", "TrackNameData_0.title"},
                              {"gp", "TrackNameData_0.gp_long_name"},
                              {"acronym", "TrackNameData_0.acronym_name"},
                              {"city", "TrackNameData_0.city_name"}})
        if (auto t = text_at(db, block, path); t && !t->empty() && !(std::string_view(name) == "acronym" && t->size() > 8))
            names[name] = Value(*t);
    if (names.size())
        v["names"] = std::move(names);
    auto day = int_at(db, block, "TrackGeneralData_0.weekend_day");
    auto month = int_at(db, block, "TrackGeneralData_0.weekend_month");
    if (day && month && *day >= 1 && *day <= 31 && *month >= 1 && *month <= 12)
        v["date"] = Value::object({{"day", Value(*day)}, {"month", Value(*month)}});
    return v;
}

// Every tracks.Track row in row order with its stat id, and every TrackData block of every data set.
inline Scan scan(const Db &db, const Tables &t)
{
    Scan s;
    if (!t.ok)
        return s;
    Table tracks = Table::of(t.track);
    std::string label = t.track.labels.empty() ? std::string("_metadata_") : t.track.labels.front();
    std::size_t n = tracks.len(db);
    for (std::size_t i = 0; i < n; i++) {
        Key r = row(tracks.base, i);
        auto code = db.text(field(r, label).h);
        if (!code || code->empty())
            continue;
        StockRow sr{*code, db.int_of(field(r, "statId").h).value_or(-1)};
        if (sr.stat_id >= 0)
            s.used.emplace(sr.stat_id, sr.code);
        s.rows.push_back(std::move(sr));
    }
    Table sets = Table::of(t.datasets);
    std::size_t m = sets.ok ? sets.len(db) : 0;
    for (std::size_t i = 0; i < m; i++) {
        Key set = row(sets.base, i);
        std::string set_label = db.text(field(set, "_metadata_").h).value_or("");
        if (set_label.empty())
            continue;
        s.sets.push_back(set_label);
        Table data = Table::nested(set, *t.track_data.path);
        std::size_t k = data.len(db);
        for (std::size_t j = 0; j < k; j++) {
            Key block = row(data.base, j);
            auto code = db.text(field(block, "track").h);
            if (!code || code->empty())
                continue;
            s.blocks.push_back({*code, set_label, block_value(db, block, "game/" + *code, set_label)});
        }
    }
    return s;
}

struct Older {
    std::string addon, code, name, base, slot, venue;
    int64_t stat_id = -1;
};

struct OlderFile {
    std::vector<Older> tracks;
    std::vector<std::string> lines;
    bool read = false;
};

// $DATA/registry/older-tracks.json (contract C3). A file for another profile than the build's index.json names is
// ignored.
inline OlderFile parse_older(const std::string &text, const std::string &build_profile)
{
    OlderFile out;
    if (text.empty()) {
        out.lines.push_back("[tracks] no older-tracks.json: no tracks made for 1.5 recorded");
        return out;
    }
    swofty::json::Error err;
    Value doc = swofty::json::parse(text, &err);
    if (!err.message.empty() || !doc.is_object() || doc["version"].as_int() != 1) {
        out.lines.push_back("[tracks] older-tracks.json is unreadable or not version 1: no tracks made for 1.5 recorded");
        return out;
    }
    std::string profile(doc["profile"].as_string());
    if (!build_profile.empty() && profile != build_profile) {
        out.lines.push_back("[tracks] older-tracks.json is for profile " + profile + "; this build is " + build_profile +
                            ": no older tracks");
        return out;
    }
    out.read = true;
    if (auto a = doc["tracks"].if_array())
        for (auto &t : *a) {
            Older o;
            o.addon = std::string(t["addon"].as_string());
            o.code = std::string(t["code"].as_string());
            o.name = std::string(t["name"].as_string());
            o.base = std::string(t["base"].as_string());
            o.slot = std::string(t["slot"].as_string());
            o.venue = std::string(t["venue"].as_string());
            o.stat_id = t["stat_id"].is_int() ? t["stat_id"].as_int() : -1;
            if (o.addon.empty() || o.code.empty()) {
                out.lines.push_back("[tracks] older-tracks.json: an entry without addon or code; left out");
                continue;
            }
            out.tracks.push_back(std::move(o));
        }
    return out;
}

inline const Older *older_of(const std::vector<Older> &older, std::string_view code)
{
    for (auto &o : older)
        if (o.code == code)
            return &o;
    return nullptr;
}

inline Value track_value(const std::string &code, const std::string &name, const std::string &base, const std::string &slot,
                         const std::vector<std::string> &provides, bool time_trial, int64_t stat_id)
{
    Value list = Value::array();
    for (auto &p : provides)
        list.push_back(Value(p));
    Value v = Value::object({{"code", Value(code)},
                             {"name", Value(name)},
                             {"base", base.empty() ? Value() : Value(base)},
                             {"slot", slot.empty() ? Value() : Value(slot)},
                             {"provides", std::move(list)},
                             {"time_trial", Value(time_trial)}});
    if (stat_id >= 0 && stat_id <= 254)
        v["stat_id"] = Value(stat_id);
    return v;
}

struct Seed {
    std::string registry, key;
    Value value;
};

struct Seeds {
    std::vector<Seed> base;
    std::vector<std::pair<std::string, Seed>> older;
};

// The venues, which need no database: the core declares them before any record that names one is read.
inline std::vector<Seed> venue_bases()
{
    std::vector<Seed> out;
    for (const VenueSeed &v : VENUE_SEEDS)
        out.push_back({"track.venue", v.key,
                       Value::object({{"name", Value(v.name)}, {"country", Value(v.country)}, {"track", Value()}})});
    return out;
}

// The records the core declares from the database: stock rows (a row an older track holds is its, not a stock row),
// data blocks of stock rows, and the older tracks as <addon>/<code>.
inline Seeds seeds(const Scan &s, const std::vector<Older> &older)
{
    Seeds out;
    std::set<std::string> older_codes;
    for (auto &o : older)
        older_codes.insert(o.code);
    for (auto &r : s.rows) {
        if (older_codes.count(r.code) && !is_dormant(r.code))
            continue;
        std::vector<std::string> provides;
        const VenueSeed *v = stock_venue(r.code);
        if (v)
            provides.push_back(std::string("venue/") + v->key);
        out.base.push_back({"track", r.code,
                            track_value(r.code, v ? v->name : r.code, "", "", provides, !is_dormant(r.code), r.stat_id)});
    }
    for (auto &b : s.blocks)
        if (!older_codes.count(b.code) || is_dormant(b.code))
            out.base.push_back({"track.data", b.code + "/" + b.set, b.value});
    for (auto &o : older) {
        std::vector<std::string> provides;
        if (!o.venue.empty())
            provides.push_back(o.venue);
        out.older.push_back({o.addon, {"track", o.code, track_value(o.code, o.name.empty() ? o.code : o.name, o.base, o.slot,
                                                                    provides, false, o.stat_id)}});
    }
    return out;
}

// Without the database (no track content): for each venue a stock track provides, that track as a record with no
// stat id, so the venues resolve and calendars that name them stay valid. It adds nothing to the track base.
inline Seeds venue_providers()
{
    Seeds out;
    for (const VenueSeed &v : VENUE_SEEDS)
        if (*v.stock)
            out.base.push_back({"track", v.stock,
                                track_value(v.stock, v.name, "", "", {std::string("venue/") + v.key}, true, -1)});
    return out;
}

inline constexpr int64_t TRACK_STAT_CAP = 254;

// The record tracks' stat-id pool: every id from 254 down that no loaded row and no older track holds, highest first,
// so a 1.5 build's max + 1 grows away from it.
inline ids::Pool track_pool(const std::map<int64_t, std::string> &used, const std::vector<Older> &older)
{
    std::set<int64_t> held;
    for (auto &[id, code] : used)
        held.insert(id);
    for (auto &o : older)
        if (o.stat_id >= 0)
            held.insert(o.stat_id);
    ids::Pool p;
    for (int64_t id = TRACK_STAT_CAP; id >= 0; id--)
        if (!held.count(id))
            p.ranges.push_back({id, id});
    return p;
}

}
