#pragma once

#include <string>

#include "direct_start_logic.hpp"
#include "json.hpp"

namespace runtime::direct_start::tt {

// The Time Trial for a session request with "mode": "time_trial" ({box, track, team, setup_mode}), polled on
// the flow thread while StateFTUE is held, until it answers done or failed.
Setup setup(const json::Value &request, std::string &why);

}
