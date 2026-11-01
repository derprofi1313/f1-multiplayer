#pragma once

#include <cstdint>
#include <string>

// The virtual pad: XInputGetState and XInputGetCapabilities in xinput1_4.dll (the game imports them by
// ordinal 2 and 4) report a connected pad in slot 0 while someone has claimed it (the test agent, the title
// skip). Buttons that are set replace a real pad's state while they are held; otherwise a real pad passes
// through.
namespace runtime::pad {

void setup();
const std::string &why();
void claim(bool on);
void set(uint16_t buttons, uint8_t lt = 0, uint8_t rt = 0);
void release();

constexpr uint16_t A = 0x1000;

}
