#pragma once

#include <cstdint>

namespace runtime::leaderboard_guard {

constexpr int LOG_FIRST = 4;

enum class Say { none, line, last };

inline bool board_missing(uint64_t board)
{
    return board == 0;
}

inline bool tab_child_missing(uint64_t child)
{
    return child == 0;
}

inline Say say(int occurrence)
{
    if (occurrence < 1 || occurrence > LOG_FIRST)
        return Say::none;
    return occurrence == LOG_FIRST ? Say::last : Say::line;
}

inline bool switch_off(const char *value)
{
    if (!value)
        return false;
    return value[0] == '0' && value[1] == 0 ? true : value[0] == 'o' && value[1] == 'f' && value[2] == 'f' && value[3] == 0;
}

}
