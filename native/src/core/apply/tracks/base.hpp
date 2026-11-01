#pragma once

#include <cstdint>
#include <map>
#include <string>
#include <string_view>
#include <vector>

// The effective track base after boot (contract C4, work/std16/TRACKS-CONTRACTS.md): every track in the base with
// its id, code, stat id, base, provides and time_trial, the dormant stock rows, and each venue's resolved track.
// The boot pass fills it once, after the track records are lowered and before the first box press; everything
// after reads it. The lookups are pure, so host tests build a Base by hand.

namespace swofty_core::apply::tracks {

enum class Kind : uint8_t { stock, record, older };

inline const char *kind_name(Kind k)
{
    switch (k) {
    case Kind::stock:
        return "stock";
    case Kind::record:
        return "record";
    default:
        return "older";
    }
}

struct BaseTrack {
    std::string id;
    std::string code;
    int stat_id = -1;
    std::string base;
    std::string slot;
    std::vector<std::string> provides;
    bool time_trial = false;
    Kind kind = Kind::stock;
    std::string addon;
    std::string name;
};

struct Base {
    bool ready = false;
    std::vector<BaseTrack> tracks;
    std::vector<std::string> dormant;
    std::map<std::string, std::string> filled;
    std::map<std::string, std::string> venues;

    const BaseTrack *by_id(std::string_view id) const
    {
        for (const BaseTrack &t : tracks)
            if (t.id == id)
                return &t;
        return nullptr;
    }

    const BaseTrack *by_code(std::string_view code) const
    {
        for (const BaseTrack &t : tracks)
            if (t.code == code)
                return &t;
        return nullptr;
    }

    const BaseTrack *by_stat_id(int stat_id) const
    {
        if (stat_id < 0)
            return nullptr;
        for (const BaseTrack &t : tracks)
            if (t.stat_id == stat_id)
                return &t;
        return nullptr;
    }

    const BaseTrack *venue(std::string_view venue_id) const
    {
        auto it = venues.find(std::string(venue_id));
        return it == venues.end() ? nullptr : by_id(it->second);
    }

    const BaseTrack *resolve(std::string_view ref) const
    {
        if (!ref.empty() && ref.back() == '?')
            ref.remove_suffix(1);
        while (!ref.empty()) {
            size_t bar = ref.find('|');
            std::string_view one = ref.substr(0, bar);
            const BaseTrack *t = one.substr(0, 6) == "venue/" ? venue(one) : by_id(one);
            if (t)
                return t;
            if (bar == std::string_view::npos)
                break;
            ref.remove_prefix(bar + 1);
        }
        return nullptr;
    }
};

inline Base &base()
{
    static Base b;
    return b;
}

}
