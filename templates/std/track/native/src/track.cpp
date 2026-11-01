// SPDX-License-Identifier: MIT
#include <string>

#include <swofty/registry.hpp>
#include <swofty/systems/tracks.hpp>

using namespace swofty;

SWOFTY_REGISTER
{
    // mod.hockenheim/hockenheim: a new row cloned from Austria's in every data set; the launcher places track/
    tracks::tracks.add("hockenheim", {
        .code = "hockenheim",
        .name = "Hockenheimring",
        .base = tracks::tracks["game/austria"],
        .provides = {tracks::venues["venue/hockenheimring"]},
        .names = {.title = Text::key("lng_hockenheim_title"), .gp = Text::key("lng_hockenheim_gp"), .acronym = "HOC"},
        .files = file("track/"),
        .length_metres = 4574,
        .corners = 17,
        .latitude = 49.3278f,
        .longitude = 8.5658f,
    });

    // every data set: 67 laps, its own fuel and pit lane
    tracks::data.layer(under<tracks::TrackData>("mod.hockenheim/hockenheim/"), [](tracks::TrackData &d) {
        d.laps = 67;
        d.fuel.burn_per_lap = 1.62f;
        d.pit.speed_limit = 80;
    });

    // the 2025 and 2026 sets: the 2019 compounds
    for (const char *set : {"f1_2025", "sprint_f1_2025", "f1_2026", "sprint_f1_2026"})
        tracks::data.layer(tracks::data[std::string("mod.hockenheim/hockenheim/") + set], [](tracks::TrackData &d) {
            d.tyres = {.softer = "C4", .medium = "C3", .harder = "C2"};
        });
}

SWOFTY_ADDON()
