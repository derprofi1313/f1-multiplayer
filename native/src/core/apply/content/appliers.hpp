#pragma once

#include <string>

namespace swofty_core::apply::content {

void register_appliers();
void undress_teams(const char *why);

// The DRS group the box with this entrant list runs: the label of its content.package's rules.drs group if the
// boot apply added it (empty otherwise), and the line to log.
struct DrsBox {
    std::string label, line;
};
DrsBox drs_for_box(const std::string &entrant_list);

}
