// SPDX-License-Identifier: MIT
#pragma once
#include <string>

#include <swofty/systems/career.hpp>

namespace notes {

namespace saves = swofty::career::saves;
namespace json = swofty::json;

inline constexpr const char *kAddon = "mod.career-notes";

// Version 1 stored {"loads": n}; version 2 adds the last round the index had.
inline constexpr uint32_t kVersion = 2;

struct Notes {
    int loads = 0;
    int last_round = 0;
};

inline Notes read(const std::string &career)
{
    Notes n;
    auto stored = saves::data(kAddon).get(career);
    if (!stored)
        return n;
    n.loads = static_cast<int>(stored->value["loads"].as_int());
    if (stored->version >= 2)
        n.last_round = static_cast<int>(stored->value["last_round"].as_int());
    return n;
}

inline swofty::career::Result write(const std::string &career, const Notes &n)
{
    return saves::data(kAddon).set(
        career, json::Value::object({{"loads", json::Value(int64_t(n.loads))},
                                     {"last_round", json::Value(int64_t(n.last_round))}}),
        kVersion);
}

inline void on_loaded()
{
    std::string career = saves::current();
    if (career.empty())
        return;
    Notes n = read(career);
    n.loads++;
    write(career, n);
    saves::need(kAddon, "^1.0.0", "its visit counter");
}

inline void on_saving()
{
    std::string career = saves::current();
    auto entry = saves::find(career);
    if (career.empty() || !entry)
        return;
    Notes n = read(career);
    n.last_round = entry->round;
    write(career, n);
}

inline std::string on_missing()
{
    std::string text;
    for (auto &gone : saves::missing())
        text += (text.empty() ? "" : ", ") + gone.career;
    return text;
}

}
