#pragma once
#include <cstdint>

namespace runtime::track_record {

struct Seen {
    bool bits[256] = {};

    bool first(uint8_t track)
    {
        if (bits[track])
            return false;
        bits[track] = true;
        return true;
    }
};

}
