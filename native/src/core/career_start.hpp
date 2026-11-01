#pragma once
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <string>
#include <string_view>
#include <vector>

#include "swofty_api.h"

// Front-end routes that start or load a career (routes_frontend_general.uir, groups fe_career,
// fe_career_dlc_select, career_continue_* and career_wizard_career_type_select). A route name maps to
// a career mode and to what the player picked: the mode alone (the season or save choice comes
// next), a new career, or an existing save.

namespace runtime::career_start {

struct Route {
    const char *name;
    uint32_t mode, kind;
};

inline constexpr Route ROUTES[] = {
    {"career_dlc_select_my_team", SWOFTY_CAREER_MY_TEAM, SWOFTY_CAREER_START_PICK},
    {"career_dlc_select_driver", SWOFTY_CAREER_DRIVER, SWOFTY_CAREER_START_PICK},
    {"career_wizard_my_team_career", SWOFTY_CAREER_MY_TEAM, SWOFTY_CAREER_START_NEW},
    {"career_wizard_my_team_season_length", SWOFTY_CAREER_MY_TEAM, SWOFTY_CAREER_START_NEW},
    {"career_wizard_driver_career", SWOFTY_CAREER_DRIVER, SWOFTY_CAREER_START_NEW},
    {"career_wizard_driver_career_to_f1", SWOFTY_CAREER_DRIVER, SWOFTY_CAREER_START_NEW},
    {"solo_career_continue_my_team", SWOFTY_CAREER_MY_TEAM, SWOFTY_CAREER_START_CONTINUE},
    {"solo_career_continue_driver", SWOFTY_CAREER_DRIVER, SWOFTY_CAREER_START_CONTINUE},
    {"to_manage_career_my_team", SWOFTY_CAREER_MY_TEAM, SWOFTY_CAREER_START_CONTINUE},
    {"to_manage_career_driver", SWOFTY_CAREER_DRIVER, SWOFTY_CAREER_START_CONTINUE},
};

inline const Route *classify(std::string_view name)
{
    for (const Route &r : ROUTES)
        if (name == r.name)
            return &r;
    return nullptr;
}

struct Block {
    uint64_t owner;
    std::string by;
    uint32_t mode;
    std::string title, body;
};

class Blocks {
public:
    bool set(uint64_t owner, const std::string &by, uint32_t mode, const std::string &title, const std::string &body)
    {
        if (mode != SWOFTY_CAREER_MY_TEAM && mode != SWOFTY_CAREER_DRIVER)
            return false;
        for (auto &b : list_)
            if (b.owner == owner && b.mode == mode) {
                b.title = title, b.body = body, b.by = by;
                return true;
            }
        list_.push_back({owner, by, mode, title, body});
        return true;
    }

    bool clear(uint64_t owner, uint32_t mode)
    {
        for (size_t i = 0; i < list_.size(); i++)
            if (list_[i].owner == owner && list_[i].mode == mode) {
                list_.erase(list_.begin() + long(i));
                return true;
            }
        return false;
    }

    void drop_owner(uint64_t owner)
    {
        for (size_t i = list_.size(); i-- > 0;)
            if (list_[i].owner == owner)
                list_.erase(list_.begin() + long(i));
    }

    const Block *find(uint32_t mode) const
    {
        for (auto &b : list_)
            if (b.mode == mode)
                return &b;
        return nullptr;
    }

    size_t size() const { return list_.size(); }

private:
    std::vector<Block> list_;
};

inline SwoftyCareerStart make_start(const Route &r, bool blocked)
{
    SwoftyCareerStart s{};
    s.mode = r.mode;
    s.kind = r.kind;
    s.blocked = blocked;
    snprintf(s.route, sizeof s.route, "%s", r.name);
    return s;
}

inline bool due(uint64_t now_ms, uint64_t &last_ms, uint64_t gap_ms)
{
    if (last_ms && now_ms - last_ms < gap_ms)
        return false;
    last_ms = now_ms;
    return true;
}

}
