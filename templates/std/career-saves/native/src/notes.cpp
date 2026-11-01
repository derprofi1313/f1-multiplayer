// SPDX-License-Identifier: MIT
#include <swofty/events.hpp>
#include <swofty/mixin.hpp>
#include <swofty/registry.hpp>
#include <swofty/systems/career.hpp>

#include "notes.hpp"

using namespace swofty;

SWOFTY_ON_LOAD
{
    events::on<career::events::Loaded>().after(
        [](const career::events::Loaded &, const events::Context &) { notes::on_loaded(); });
    events::on<career::events::Saving>().before(
        [](career::events::Saving &, const events::Context &) { notes::on_saving(); });
    events::on<career::events::Missing>().after(
        [](const career::events::Missing &, const events::Context &) {
            swofty::log("[notes] careers in the index and not in the save: %s", notes::on_missing().c_str());
        });
}

SWOFTY_ADDON()
