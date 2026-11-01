// SPDX-License-Identifier: MIT
#pragma once
#include <swofty/registry.hpp>
#include <swofty/systems/forward.hpp>

// Tyres, first slice (experimental, 1.7.0): physical compounds with their tyre group, visual compounds, tyre
// classes, allocations and rules. The records are stored and nothing in them is applied: the core writes no
// tyre rows from them yet. What reaches the car today is the tyre group, which the launcher writes into a
// handling file's Chassis TyrePhysicsGroup at build, from Vehicle::tyre_group (or from
// cars::Chassis::tyre_physics_group in the handling record), one group per team's car.

namespace swofty::tyres {

struct Physical {
    static constexpr Version version{1, 0};
    std::string group;
    std::string code;
    Ref<Class> cls;
    float qualifying_grip = 1, race_grip = 1;
    json::Value extra;
};
SWOFTY_RECORD(Physical, group, code, cls, qualifying_grip, race_grip)

struct Visual {
    static constexpr Version version{1, 0};
    std::string label;
    Colour colour;
    Ref<Physical> physical;
    json::Value extra;
};
SWOFTY_RECORD(Visual, label, colour, physical)

struct Class {
    static constexpr Version version{1, 0};
    std::string name;
    std::vector<Ref<Physical>> compounds;
    json::Value extra;
};
SWOFTY_RECORD(Class, name, compounds)

struct Allocation {
    static constexpr Version version{1, 0};
    Ref<Class> cls;
    int dry_sets = 0, intermediate_sets = 0, wet_sets = 0;
    json::Value extra;
};
SWOFTY_RECORD(Allocation, cls, dry_sets, intermediate_sets, wet_sets)

struct Rules {
    static constexpr Version version{1, 0};
    Ref<Allocation> allocation;
    int compounds_per_race = 2;
    bool mandatory_stop = true;
    json::Value extra;
};
SWOFTY_RECORD(Rules, allocation, compounds_per_race, mandatory_stop)

inline Registry<Physical> physical{"tyre.physical", {.applies = Applies::session}};
inline Registry<Visual> visuals{"tyre.visual", {.applies = Applies::session}};
inline Registry<Class> classes{"tyre.class", {.applies = Applies::session}};
inline Registry<Allocation> allocations{"tyre.allocation", {.applies = Applies::session}};
inline Registry<Rules> rules{"tyre.rules", {.applies = Applies::session}};

}
