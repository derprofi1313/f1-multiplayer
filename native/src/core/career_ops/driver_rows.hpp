#pragma once
#include <cstdint>
#include <vector>

namespace career_ops::rows {

struct Row {
    uint32_t driver = 0;
    bool teamless = false;
};

constexpr int kNone = -1;
constexpr int kAmbiguous = -2;

inline int pick(const std::vector<Row> &rows, uint32_t stat)
{
    int live = kNone, first = kNone;
    for (size_t i = 0; i < rows.size(); ++i) {
        if ((rows[i].driver >> 16) != stat || (rows[i].driver & 0xff) != 0)
            continue;
        if (first < 0)
            first = static_cast<int>(i);
        if (rows[i].teamless)
            continue;
        if (live >= 0)
            return kAmbiguous;
        live = static_cast<int>(i);
    }
    return live >= 0 ? live : first;
}

}
