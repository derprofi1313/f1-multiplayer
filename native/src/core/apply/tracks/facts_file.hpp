#pragma once

#include <swofty/json.hpp>

#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

// The launcher's per-track facts (contract C2, work/std16/TRACKS-CONTRACTS.md): $DATA/registry/track-facts.json,
// written by every successful build. Per track code and world variant the grid slots, garages and pit boxes after
// the build's grid extension, the AI folders the track holds and the packages of its ai_track_difficulties; the
// built database's track layers in row order. Pure: the core reads the file at the first box press and host tests
// parse fixtures.

namespace swofty_core::apply::tracks {

struct World {
    int grid = -1, garages = -1, pit_boxes = -1;
    std::string stand_in;
};

struct TrackFacts {
    std::map<std::string, World> worlds;
    std::vector<std::string> ai, ai_difficulties, ui_lists;
    bool has_ai = false, has_difficulties = false;
    bool time_trial = true;
    std::string time_trial_why;
};

struct Layer {
    std::string label, world;
    std::vector<std::string> setup_modes, seasons, weekend_formats;
};

struct Facts {
    bool loaded = false;
    int version = 0;
    std::string profile, inputs;
    int grid_size = 0;
    std::vector<Layer> layers;
    std::map<std::string, TrackFacts> tracks;

    const TrackFacts *track(std::string_view code) const
    {
        auto it = tracks.find(std::string(code));
        return it == tracks.end() ? nullptr : &it->second;
    }

    bool any_track_holds(const std::string &world) const
    {
        for (auto &[code, t] : tracks)
            if (t.worlds.count(world))
                return true;
        return false;
    }
};

inline std::vector<std::string> facts_strings(const swofty::json::Value &v)
{
    std::vector<std::string> out;
    if (auto a = v.if_array())
        for (auto &e : *a)
            if (e.is_string())
                out.emplace_back(e.as_string());
    return out;
}

inline int facts_int(const swofty::json::Value &v)
{
    return v.is_number() ? int(v.as_int(-1)) : -1;
}

inline Facts parse_facts(std::string_view text, std::string *why = nullptr)
{
    Facts f;
    swofty::json::Error e;
    swofty::json::Value doc = swofty::json::parse(text, &e);
    if (!e.ok() || !doc.is_object()) {
        if (why)
            *why = e.ok() ? std::string("not a JSON object") : e.message;
        return f;
    }
    f.version = int(doc["version"].as_int(0));
    if (f.version != 1) {
        if (why)
            *why = "version " + std::to_string(f.version) + " (this core reads version 1)";
        return f;
    }
    f.profile = std::string(doc["profile"].as_string());
    f.inputs = std::string(doc["inputs"].as_string());
    f.grid_size = doc["grid_size"].is_number() ? int(doc["grid_size"].as_int(0)) : 0;
    if (auto layers = doc["layers"].if_array())
        for (auto &l : *layers) {
            if (!l.is_object())
                continue;
            Layer layer;
            layer.label = std::string(l["label"].as_string());
            layer.world = std::string(l["world"].as_string());
            layer.setup_modes = facts_strings(l["setup_modes"]);
            layer.seasons = facts_strings(l["seasons"]);
            layer.weekend_formats = facts_strings(l["weekend_formats"]);
            f.layers.push_back(std::move(layer));
        }
    if (auto tracks = doc["tracks"].if_object())
        for (auto &[code, t] : *tracks) {
            if (!t.is_object())
                continue;
            TrackFacts tf;
            if (auto worlds = t["worlds"].if_object())
                for (auto &[suffix, w] : *worlds) {
                    if (!w.is_object())
                        continue;
                    World world;
                    world.grid = facts_int(w["grid"]);
                    world.garages = facts_int(w["garages"]);
                    world.pit_boxes = facts_int(w["pit_boxes"]);
                    world.stand_in = std::string(w["stand_in"].as_string());
                    tf.worlds[std::string(suffix)] = world;
                }
            tf.has_ai = t["ai"].is_array();
            tf.has_difficulties = t["ai_difficulties"].is_array();
            tf.ai = facts_strings(t["ai"]);
            tf.ai_difficulties = facts_strings(t["ai_difficulties"]);
            tf.ui_lists = facts_strings(t["ui_lists"]);
            tf.time_trial = t["time_trial"].as_bool(true);
            tf.time_trial_why = std::string(t["time_trial_why"].as_string());
            f.tracks[std::string(code)] = std::move(tf);
        }
    f.loaded = true;
    return f;
}

// The profile rule of C2: a file made for another profile than the build's index names gives no facts.
inline bool facts_profile_matches(const Facts &f, std::string_view index_profile, std::string *line)
{
    if (index_profile.empty() || f.profile.empty() || f.profile == index_profile)
        return true;
    if (line)
        *line = "[tracks] track-facts.json is for profile " + f.profile + "; this build is " + std::string(index_profile) +
                ": no facts";
    return false;
}

}
