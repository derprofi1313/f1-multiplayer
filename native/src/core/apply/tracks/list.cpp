#include "box.hpp"

#include "../../era_logic.hpp"
#include "../content/gvdb.hpp"
#include "../../registry/store.hpp"
#include "../../registry/table.hpp"
#include "../../runtime.hpp"
#include "../scheduler.hpp"

#include <swofty/mixin.hpp>

#include <cstdio>
#include <map>
#include <mutex>

// The box track lists at runtime: track-facts.json (C2) read once from the core's registry folder, a box made from
// its effective content.package record and the live database (box_from_package), an addon's era box with no record
// made from its entrant list in the live database (box_from_list), its per_package entries along its
// variant chain, its list against the effective base (C4), catalog.json's "tracks" object rewritten at each box
// press (with the build's profile and an id of its registry/index.json), and the guard's decision for a confirmed track.

namespace runtime {
std::vector<era::Box> era_boxes();
}

namespace swofty_core::apply::tracks {

namespace {

using swofty::json::Value;

std::mutex mu;
Active current;

std::string registry_dir()
{
    const std::wstring &state = runtime::state_root();
    return runtime::narrow((state.empty() ? runtime::swofty_root() : state) + L"registry");
}

struct Build {
    std::string profile, id;
};

const Build &this_build()
{
    static Build b;
    static std::once_flag once;
    std::call_once(once, [] {
        std::string text = runtime::read_file(runtime::under_swofty("registry/index.json"), size_t(16) << 20);
        if (text.empty())
            return;
        swofty::json::Error e;
        Value doc = swofty::json::parse(text, &e);
        if (e.ok())
            b.profile = std::string(doc["profile"].as_string());
        char hex[17];
        std::snprintf(hex, sizeof hex, "%016llx", static_cast<unsigned long long>(content::gvdb::fnv64(text)));
        b.id = hex;
    });
    return b;
}

TextRead db_read()
{
    const SwoftyDb *db = runtime::db_table();
    if (!db || !db->available || !db->string_at || !db->available())
        return {};
    return [db](const std::string &path) -> std::optional<std::string> {
        const char *s = nullptr;
        if (db->string_at(path.c_str(), &s) != SWOFTY_ASSET_OK || !s)
            return std::nullopt;
        return std::string(s);
    };
}

ListDb db_list()
{
    const SwoftyDb *db = runtime::db_table();
    if (!db || !db->available || !db->row || !db->int_at || !db->available())
        return {};
    ListDb out;
    out.text = db_read();
    out.integer = [db](const std::string &key) -> std::optional<int64_t> {
        int64_t v = 0;
        if (db->int_at(key.c_str(), &v) != SWOFTY_ASSET_OK)
            return std::nullopt;
        return v;
    };
    out.row = [db](const std::string &table, const std::string &label) -> std::optional<uint64_t> {
        SwoftyDbCursor c{};
        if (db->row(table.c_str(), label.c_str(), &c) != SWOFTY_ASSET_OK)
            return std::nullopt;
        return c.index;
    };
    return out;
}

bool era_list(const std::string &list)
{
    for (auto &b : runtime::era_boxes())
        if (b.entrant_list == list || b.time_trial == list)
            return true;
    return false;
}

CellRead db_cell_read()
{
    const SwoftyDb *db = runtime::db_table();
    if (!db || !db->available || !db->row || !db->cell || !db->available())
        return {};
    return [db](const std::string &table, const std::string &label, const std::string &field) -> std::optional<int64_t> {
        SwoftyDbCursor c{};
        SwoftyDbValue v{};
        if (db->row(table.c_str(), label.c_str(), &c) != SWOFTY_ASSET_OK ||
            db->cell(&c, field.c_str(), &v) != SWOFTY_ASSET_OK || v.tag == SWOFTY_DB_STRING || v.tag == SWOFTY_DB_FLOAT)
            return std::nullopt;
        return v.i;
    };
}

}

const Facts &facts()
{
    static Facts f;
    static std::once_flag once;
    std::call_once(once, [] {
        std::string path = registry_dir() + "\\track-facts.json";
        std::string text = runtime::read_file(runtime::wide(path), size_t(64) << 20);
        if (text.empty()) {
            swofty::log("[tracks] no track-facts.json in %s: every fit rule passes", registry_dir().c_str());
            return;
        }
        std::string why, line;
        Facts parsed = parse_facts(text, &why);
        if (!parsed.loaded) {
            swofty::log("[tracks] track-facts.json unreadable (%s): every fit rule passes", why.c_str());
            return;
        }
        std::string profile = registry::store().context().profile;
        if (!facts_profile_matches(parsed, profile == "default" ? std::string() : profile, &line)) {
            swofty::log("%s", line.c_str());
            return;
        }
        f = std::move(parsed);
        swofty::log("[tracks] track-facts.json: %zu track(s), %zu layer(s), grid size %d", f.tracks.size(), f.layers.size(),
                    f.grid_size);
    });
    return f;
}

int setup_mode_stat(const std::string &setup_mode)
{
    return setup_mode_stat_of(db_cell_read(), setup_mode);
}

BoxInfo box_info(registry::Store &store, const std::string &package, const std::string &entrant_list)
{
    BoxInfo b;
    b.package = package.empty() ? "game/" + entrant_list : package;
    b.entrant_list = entrant_list;
    if (package.empty()) {
        if (entrant_list.empty() || !era_list(entrant_list))
            return b;
        ListDb db = db_list();
        if (!db.text)
            return b;
        b = box_from_list(entrant_list, db);
        swofty::log("[tracks] box %s: no content.package record; from its entrant list: setup mode %s, %s teams, %s cars",
                    b.package.c_str(), b.setup_mode.c_str(), b.teams < 0 ? "unknown" : std::to_string(b.teams).c_str(),
                    b.cars < 0 ? "unknown" : std::to_string(b.cars).c_str());
        if (!b.unread.empty())
            swofty::log("[tracks] box %s: not fully read (%s); the garage and grid rules assume the grid's size", b.package.c_str(),
                        b.unread.c_str());
        return b;
    }
    registry::Record r = store.get("content.package", package);
    if (r.status != registry::Status::ok)
        return b;
    std::map<std::string, Value> keep;
    Lookup lookup = [&](const std::string &reg, const std::string &id) -> const Value * {
        std::string key = reg + "|" + id;
        auto it = keep.find(key);
        if (it != keep.end())
            return &it->second;
        registry::Record rec = store.get(reg, id);
        if (rec.status != registry::Status::ok)
            return nullptr;
        return &keep.emplace(key, std::move(rec.value)).first->second;
    };
    return box_from_package(package, r.value, lookup, db_read());
}

Entries box_entries(registry::Store &store, const std::vector<std::string> &chain)
{
    std::vector<std::pair<std::string, Value>> all;
    for (auto &id : store.list("track.per_package", "")) {
        registry::Record r = store.get("track.per_package", id);
        if (r.status == registry::Status::ok)
            all.push_back({id, std::move(r.value)});
    }
    return resolve_entries(base(), chain_records(all, chain));
}

List box_list(registry::Store &store, const BoxInfo &box, Mode mode, const Entries &entries)
{
    std::vector<Round> rounds;
    registry::Record pkg = store.get("content.package", box.package);
    if (pkg.status == registry::Status::ok && pkg.value["calendar"].is_string()) {
        registry::Record cal = store.get("career.calendar", std::string(pkg.value["calendar"].as_string()));
        if (cal.status == registry::Status::ok)
            rounds = rounds_of(cal.value);
    }
    return build_list(base(), &facts(), box, mode, std::move(rounds), entries);
}

void write_catalog(const std::string &package, Mode mode, const List &list)
{
    std::string path = registry_dir() + "\\catalog.json";
    std::lock_guard g(mu);
    std::string text = runtime::read_file(runtime::wide(path), size_t(64) << 20);
    swofty::json::Error e;
    Value doc = text.empty() ? Value::object() : swofty::json::parse(text, &e);
    if (!e.ok() || !doc.is_object())
        doc = Value::object();
    const Build &b = this_build();
    put_catalog(doc, b.profile, b.id, package, mode, list_json(list));
    std::string tmp = path + ".tmp";
    if (!runtime::write_file(runtime::wide(tmp), swofty::json::dump(doc, 1)) ||
        !MoveFileExW(runtime::wide(tmp).c_str(), runtime::wide(path).c_str(), MOVEFILE_REPLACE_EXISTING))
        swofty::log("[tracks] catalog.json could not be written in %s", registry_dir().c_str());
}

void set_active(Active a)
{
    std::lock_guard g(mu);
    current = std::move(a);
}

void clear_active()
{
    std::lock_guard g(mu);
    current = Active();
}

Active active()
{
    std::lock_guard g(mu);
    return current;
}

Mode mode_named(std::string_view mode)
{
    return mode == "time_trial" ? Mode::time_trial : Mode::grand_prix;
}

Decision guard_package(const std::string &package, const BaseTrack *track, Mode mode, std::string_view label)
{
    Active a = active();
    BoxInfo box;
    Entries entries;
    if (a.set && a.package == package) {
        box = a.box;
        entries = a.entries;
    } else {
        registry::Store &st = registry::store();
        box = box_info(st, package.rfind("game/", 0) == 0 ? std::string() : package,
                       package.rfind("game/", 0) == 0 ? package.substr(5) : std::string());
        entries = box_entries(st, st.chain("content.package", package));
    }
    Fit fit;
    const Entry *e = track ? entries.of(track->id) : nullptr;
    if (e && !e->offered) {
        fit.ok = false;
        fit.rule = Rule::offered;
        fit.reason = "not offered in this box";
    } else if (track) {
        fit = fits(box, *track, mode, &facts());
    }
    return decide(track, box, mode, fit, label);
}

}

namespace runtime::tracks {

Decision guard(std::string_view track_code, uint8_t stat_id, std::string_view box_entrant_list, bool time_trial)
{
    namespace t = swofty_core::apply::tracks;
    const t::Base &b = t::base();
    const t::BaseTrack *track = b.ready ? b.by_code(track_code) : nullptr;
    if (!track && b.ready && stat_id != 0xff)
        track = b.by_stat_id(stat_id);
    std::string package = swofty_core::apply::scheduler().find_package(std::string(box_entrant_list));
    if (package.empty())
        package = "game/" + std::string(box_entrant_list);
    t::Decision d = t::guard_package(package, track, time_trial ? t::Mode::time_trial : t::Mode::grand_prix, track_code);
    swofty::log("%s", d.line.c_str());
    return {d.ok, d.title, d.body};
}

}
