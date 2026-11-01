#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

using FeHome_Add_CareerTile_t = void(TileList *list, uint8_t index);
inline constexpr fn<"FeHome_Add_CareerTile", FeHome_Add_CareerTile_t> FeHome_Add_CareerTile{}; // L3 stable
using FeHome_Handle_Command_t = uint64_t(FEHomeViewModel *vm, NeStr **cmd);
inline constexpr fn<"FeHome_Handle_Command", FeHome_Handle_Command_t> FeHome_Handle_Command{}; // L3 stable, game::FEHomeViewModel::onCommandRaised
inline constexpr site<"site_FeHome_AfterCareerCall"> site_FeHome_AfterCareerCall{}; // L3 stable

}
