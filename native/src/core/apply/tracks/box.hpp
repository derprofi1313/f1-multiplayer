#pragma once

#include "list.hpp"

#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace swofty_core::registry {
class Store;
}

// The core's runtime side of the box track lists (list.cpp, per_package.cpp, data_session.cpp, track_select.cpp):
// the facts file read once, the box a package id names, its track.per_package entries, the active box set at each
// box press, catalog.json's "tracks" object and the guard's decision for one confirmed track.

namespace swofty_core::apply::tracks {

struct Active {
    bool set = false;
    std::string package;
    Mode mode = Mode::grand_prix;
    BoxInfo box;
    Entries entries;
};

const Facts &facts();
BoxInfo box_info(registry::Store &store, const std::string &package, const std::string &entrant_list);
Entries box_entries(registry::Store &store, const std::vector<std::string> &chain);
List box_list(registry::Store &store, const BoxInfo &box, Mode mode, const Entries &entries);
void write_catalog(const std::string &package, Mode mode, const List &list);
void set_active(Active a);
void clear_active();
Active active();
Mode mode_named(std::string_view mode);
Decision guard_package(const std::string &package, const BaseTrack *track, Mode mode, std::string_view label);
int put_back_box_fuel(const char *why);
int write_box_fuel_now(const Active &a, int mode_stat);
int setup_mode_stat(const std::string &setup_mode);
void register_box_appliers();

}

namespace runtime::tracks {

struct Decision {
    bool ok = true;
    std::string title, body;
};

Decision guard(std::string_view track_code, uint8_t stat_id, std::string_view box_entrant_list, bool time_trial);

}
