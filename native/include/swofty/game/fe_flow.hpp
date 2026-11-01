#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

struct FlowState {
    uint8_t _pad_0x0[0x20];
    uint64_t name_size; // L2 experimental
    char *name; // L2 experimental
    uint8_t _pad_0x30[0x80];
    uint32_t name_hash; // L2 experimental
    uint8_t _pad_0xb4[0x4];
    int32_t status; // L2 experimental
};
static_assert(offsetof(FlowState, name_size) == 0x20);
static_assert(offsetof(FlowState, name) == 0x28);
static_assert(offsetof(FlowState, name_hash) == 0xb0);
static_assert(offsetof(FlowState, status) == 0xb8);

inline constexpr address<"FeFlow_Set_StateStatus"> FeFlow_Set_StateStatus{}; // L2 experimental
inline constexpr site<"site_FeFlow_SetStateStatus"> site_FeFlow_SetStateStatus{}; // L2 experimental

}
