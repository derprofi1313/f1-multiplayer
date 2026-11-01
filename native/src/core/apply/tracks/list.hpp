#pragma once

#include "fits.hpp"
#include "per_package.hpp"

#include <swofty/json.hpp>

#include <algorithm>
#include <map>
#include <string>
#include <vector>

// A box's track list in one mode (tracks::TrackList): the season's real calendar in round order (each round's
// venue resolved through the base, a round with no providing track dropped, a track or missing venue a later round
// repeats listed and counted once), then "More tracks": every other base
// track in base order, with a per_package order N placing its track at place N. offered = false and a failed fit
// leave a track out with the reason. Pure: the core builds it at each box press for catalog.json and the
// [tracks] box line; host tests build it from hand-made bases.

namespace swofty_core::apply::tracks {

struct Round {
    int round = 0;
    std::string venue;
};

struct LeftOut {
    std::string track, reason;
};

struct List {
    std::vector<std::string> calendar, more;
    std::vector<LeftOut> left_out;
    std::vector<std::string> missing, no_facts;
    int rounds = 0;
};

inline bool listed(const std::vector<std::string> &v, const std::string &id)
{
    return std::find(v.begin(), v.end(), id) != v.end();
}

inline List build_list(const Base &base, const Facts *facts, const BoxInfo &box, Mode mode, std::vector<Round> rounds,
                       const Entries &entries)
{
    List out;
    std::stable_sort(rounds.begin(), rounds.end(), [](const Round &a, const Round &b) { return a.round < b.round; });
    std::map<std::string, bool> judged;
    auto judge = [&](const BaseTrack &t) {
        auto it = judged.find(t.id);
        if (it != judged.end())
            return it->second;
        bool ok = true;
        const Entry *e = entries.of(t.id);
        if (e && !e->offered) {
            out.left_out.push_back({t.id, "not offered in this box"});
            ok = false;
        } else {
            Fit f = fits(box, t, mode, facts);
            for (auto &n : f.no_facts)
                if (!listed(out.no_facts, t.code + ": " + n))
                    out.no_facts.push_back(t.code + ": " + n);
            if (!f.ok) {
                out.left_out.push_back({t.id, f.reason});
                ok = false;
            }
        }
        judged[t.id] = ok;
        return ok;
    };
    std::vector<std::string> seen;
    for (auto &r : rounds) {
        const BaseTrack *t = r.venue.empty() ? nullptr : base.resolve(r.venue);
        std::string v = r.venue.empty() ? "round " + std::to_string(r.round) : r.venue;
        if (!v.empty() && v.back() == '?')
            v.pop_back();
        std::string key = t ? t->id : v;
        if (listed(seen, key))
            continue;
        seen.push_back(key);
        out.rounds++;
        if (!t) {
            out.missing.push_back(v);
            continue;
        }
        if (judge(*t))
            out.calendar.push_back(t->id);
    }
    std::vector<std::pair<int, std::string>> placed;
    for (const BaseTrack &t : base.tracks) {
        if (listed(out.calendar, t.id) || !judge(t))
            continue;
        const Entry *e = entries.of(t.id);
        if (e && e->order > 0)
            placed.push_back({e->order, t.id});
        else
            out.more.push_back(t.id);
    }
    std::stable_sort(placed.begin(), placed.end(), [](auto &a, auto &b) { return a.first < b.first; });
    for (auto &[order, id] : placed)
        out.more.insert(out.more.begin() + std::min<std::ptrdiff_t>(order - 1, std::ptrdiff_t(out.more.size())), id);
    return out;
}

inline std::string left_out_text(const std::vector<LeftOut> &left)
{
    std::vector<std::pair<std::string, std::vector<std::string>>> by_reason;
    for (auto &l : left) {
        auto it = std::find_if(by_reason.begin(), by_reason.end(), [&](auto &p) { return p.first == l.reason; });
        if (it == by_reason.end())
            by_reason.push_back({l.reason, {l.track}});
        else
            it->second.push_back(l.track);
    }
    std::string out;
    for (auto &[reason, ids] : by_reason) {
        out += out.empty() ? "" : ", ";
        out += ids.size() == 1 ? ids.front() : std::to_string(ids.size()) + " tracks";
        out += " (" + reason + ")";
    }
    return out;
}

inline std::string list_line(const std::string &package, Mode mode, const List &l)
{
    std::string line = "[tracks] box " + package + " (" + mode_name(mode) + "): ";
    if (l.rounds) {
        line += "calendar " + std::to_string(l.calendar.size()) + " of " + std::to_string(l.rounds) + " rounds";
        if (!l.missing.empty()) {
            line += " (";
            for (size_t i = 0; i < l.missing.size(); ++i)
                line += (i ? ", " : "") + l.missing[i] + ": no track";
            line += ")";
        }
        line += ", more tracks " + std::to_string(l.more.size());
    } else {
        line += std::to_string(l.more.size()) + " tracks";
    }
    if (!l.left_out.empty())
        line += "; left out " + left_out_text(l.left_out);
    return line;
}

inline swofty::json::Value list_json(const List &l)
{
    using swofty::json::Value;
    Value calendar = Value::array(), more = Value::array(), left = Value::array();
    for (auto &id : l.calendar)
        calendar.push_back(Value(id));
    for (auto &id : l.more)
        more.push_back(Value(id));
    for (auto &o : l.left_out)
        left.push_back(Value::object({{"track", Value(o.track)}, {"reason", Value(o.reason)}}));
    return Value::object({{"calendar", std::move(calendar)}, {"more", std::move(more)}, {"left_out", std::move(left)}});
}

// catalog.json holds the box lists of one build: a box press under another profile or build starts its "tracks" again.
inline void put_catalog(swofty::json::Value &doc, const std::string &profile, const std::string &build,
                        const std::string &package, Mode mode, swofty::json::Value list)
{
    using swofty::json::Value;
    if (!doc.is_object())
        doc = Value::object();
    const Value &was = doc;
    if (was["profile"].as_string() != profile || was["build"].as_string() != build || !was["tracks"].is_object())
        doc["tracks"] = Value::object();
    doc["profile"] = Value(profile);
    doc["build"] = Value(build);
    doc["tracks"][package][mode_name(mode)] = std::move(list);
}

inline std::vector<Round> rounds_of(const swofty::json::Value &calendar)
{
    std::vector<Round> out;
    if (auto rounds = calendar["rounds"].if_array())
        for (auto &r : *rounds)
            if (r.is_object())
                out.push_back({int(r["round"].as_int(0)), std::string(r["venue"].as_string())});
    return out;
}

}
