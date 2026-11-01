#pragma once
#include <string>

#include "hud_drawlist.hpp"

namespace runtime::room_cover {

bool setup();
bool active();
void step(const char *what, float fraction, float ceiling = -1.0f);
void set_subtitle(const char *text);
void paint(hud::Frame &frame, uint64_t now);

struct Snapshot {
    bool active = false;
    std::string label;
    int percent = 0;
};
Snapshot snapshot();

}
