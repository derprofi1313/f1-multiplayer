#pragma once
#include <cstdint>

// One participant step of Game_ErsSystem_HandleVehicleNewLap's search, as the core's site hook sees it:
// the participant entry, its vehicle component (entry+0x28) and that component's vehicle id (+0xe8, or -1
// when it can't be read), against the message's vehicle id. skip: no vehicle, so the guard sends the
// search to the next entry; lap: the car whose lap changed, which the search stops at, so session.lap
// fires there once per message; other: the game's own compare moves on.

namespace runtime::ers {

enum class Step { skip, other, lap };

inline Step step(uint64_t entry, uint64_t vehicle, int vehicle_id, uint8_t car)
{
    if (!entry || !vehicle || vehicle_id < 0)
        return Step::skip;
    return uint8_t(vehicle_id) == car ? Step::lap : Step::other;
}

}
