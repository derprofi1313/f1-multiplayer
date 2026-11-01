#pragma once

#include <cstdint>

namespace runtime::thrustmaster_lcd {

constexpr int LOG_FIRST = 4;

inline bool record_missing(uint64_t record)
{
    return record == 0;
}

struct Skips {
    int count = 0;

    int note()
    {
        return ++count;
    }

    bool logged(int n) const
    {
        return n <= LOG_FIRST;
    }

    bool last(int n) const
    {
        return n == LOG_FIRST;
    }
};

}
