#pragma once
#include <cstring>

// Driver platform icon keys. Every row that shows a platform icon next to a driver (the HUD timing tower
// pills, the broadcast tower, the lobby lists, results, leaderboards, name plates, player cards) picks one
// of these '#overrides.' image keys from the entry's NePresenceUserID tag and resolves it through
// Vm_Resolve_RegistryNode. The AI icon (icon_platforms_ai) is not a platform and stays out of the set.
// "#overrides.default.blank" is the key the game itself writes for no icon (the grid rundown row sets
// its platform icon to it before deciding whether the driver gets one).

namespace runtime::netplay_icons {

inline constexpr const char blank[] = "#overrides.default.blank";

inline bool driver_platform_key(const char *path)
{
    static constexpr char group[] = "overrides.icons.session_crossplay_platform";
    static constexpr char large[] = "_large";
    static constexpr char leaf[] = ".icon_platforms_";
    static constexpr const char *platforms[] = {"steam", "playstation", "xbox", "ea", "generic_console", "generic_pc"};
    if (!path)
        return false;
    const char *p = path + (path[0] == '#');
    if (std::strncmp(p, group, sizeof group - 1) != 0)
        return false;
    p += sizeof group - 1;
    if (std::strncmp(p, large, sizeof large - 1) == 0)
        p += sizeof large - 1;
    if (std::strncmp(p, leaf, sizeof leaf - 1) != 0)
        return false;
    p += sizeof leaf - 1;
    for (const char *name : platforms)
        if (std::strcmp(p, name) == 0)
            return true;
    return false;
}

}
