// SPDX-License-Identifier: MIT
#pragma once
#include <swofty/registry.hpp>
#include <swofty/systems/forward.hpp>

// Power: the DRS rules a Grand Prix or Time Trial box names in its rules (sessions::Rules::drs). Each rule
// becomes a DRS allowed-lap group in the game database at boot; the box's rule applies when the box is
// pressed. Suppliers are experimental (1.7.0): a record is stored and a vehicle's engine names it, but the core
// writes no entrants.engines row from it yet. Power units and component rules come later.

namespace swofty::power {

struct DrsRule {
    static constexpr Version version{1, 0};
    int race_start = 1;
    int red_flag_restart = 1;
    int safety_car_end = 1;
    json::Value extra;
};
SWOFTY_RECORD(DrsRule, race_start, red_flag_restart, safety_car_end)

struct Supplier {
    static constexpr Version version{1, 0};
    Ref<Supplier> donor;
    std::string label;
    float peak_power_kw = 0, peak_rpm = 0, limiter_rpm = 0;
    json::Value extra;
};
SWOFTY_RECORD(Supplier, donor, label, peak_power_kw, peak_rpm, limiter_rpm)

inline Registry<Supplier> suppliers{"power.supplier", {.applies = Applies::boot}};
inline Registry<DrsRule> drs_rules{"rules.drs", {.applies = Applies::boot}};

}
