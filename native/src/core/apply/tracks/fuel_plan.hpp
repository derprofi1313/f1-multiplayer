#pragma once

#include "base.hpp"
#include "per_package.hpp"
#include "../../career_ops/fuel_logic.hpp"

#include <cstdint>
#include <cstdio>
#include <string>
#include <string_view>
#include <vector>

// The fuel writes of a box's track.per_package entries and of track.data records, planned over the track data sets
// the fuel writer reads (career_ops/fuel_logic.hpp): the box's burn per lap on its tracks in every set its setup
// mode can read and the sets the session's query picks, a track.data record's burn per lap in its named set. A track is written only when its fuel reads as
// fuel (one to three race rows) and differs from the planned value; each write keeps the bytes it found, so the core
// puts them back in reverse order. Pure.

namespace swofty_core::apply::tracks {

namespace fuel = ::career_ops::fuel;

struct FuelWrite {
    uintptr_t object = 0;
    std::string track, entry;
    fuel::Fuel before, after;
};

struct Query {
    uint32_t mode = 0, season = 0;
    bool season_used = false;
};

// The sets the session's query reads: for each weekend format the game knows (default, sprint), the first set in
// row order whose masks hold the mode, the season when it is used, and the format.
inline std::vector<const fuel::Set *> query_sets(const std::vector<fuel::Set> &sets, const Query &q)
{
    auto has = [](uint32_t mask, uint32_t bit) { return bit < 32 && ((mask >> bit) & 1u); };
    std::vector<const fuel::Set *> out;
    for (uint32_t format : {fuel::kFormatDefault, fuel::kFormatSprint})
        for (auto &s : sets)
            if (has(s.modes, q.mode) && (!q.season_used || has(s.seasons, q.season)) && has(s.formats, format)) {
                bool seen = false;
                for (auto *o : out)
                    seen = seen || o == &s;
                if (!seen)
                    out.push_back(&s);
                break;
            }
    return out;
}

// The sets a box's fuel goes into: every set whose mode mask holds the box's setup mode stat id, whatever season
// and weekend format it carries (a session in that mode reads one of them, and the season the query carries can
// change between the box press and the race), then the sets the session's own query picks when it is known.
inline std::vector<const fuel::Set *> box_sets(const std::vector<fuel::Set> &sets, uint32_t mode, const Query *q)
{
    std::vector<const fuel::Set *> out;
    auto add = [&out](const fuel::Set *s) {
        for (auto *o : out)
            if (o == s)
                return;
        out.push_back(s);
    };
    for (auto &s : sets)
        if (mode < 32 && ((s.modes >> mode) & 1u))
            add(&s);
    if (q)
        for (auto *s : query_sets(sets, *q))
            add(s);
    return out;
}

inline std::string set_masks(const std::vector<const fuel::Set *> &sets)
{
    std::string out;
    char b[64];
    for (auto *s : sets) {
        std::snprintf(b, sizeof b, "%smodes %#x seasons %#x formats %#x", out.empty() ? "" : "; ", s->modes, s->seasons,
                      s->formats);
        out += b;
    }
    return out;
}

inline void plan_burn(std::vector<FuelWrite> &out, const fuel::Set &set, const std::string &code, float burn,
                      const std::string &entry)
{
    const fuel::Track *t = fuel::find(set, code);
    if (!t || t->fuel.count < 1 || t->fuel.count > fuel::kMaxRows)
        return;
    fuel::Fuel after = fuel::with_burn(t->fuel, burn);
    if (fuel::same(after, t->fuel))
        return;
    out.push_back({t->object, code, entry, t->fuel, after});
}

inline std::vector<FuelWrite> plan_box_fuel(const Base &base, const Entries &entries, const std::vector<const fuel::Set *> &sets,
                                            std::string_view session_code)
{
    std::vector<FuelWrite> out;
    for (auto &[id, e] : entries.by_track) {
        const BaseTrack *t = base.by_id(id);
        if (!e.burn || !t || (!session_code.empty() && t->code != session_code))
            continue;
        if (!(*e.burn > 0.0f && *e.burn < 20.0f))
            continue;
        for (auto *s : sets)
            plan_burn(out, *s, t->code, *e.burn, e.id);
    }
    return out;
}

}
