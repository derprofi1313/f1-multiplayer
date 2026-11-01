#pragma once
#include <windows.h>

#include <string>

namespace runtime::dred {

void arm(const std::wstring &swofty_dir, void (*log)(const char *, ...), HMODULE self);

// The "GPU device removal" section of a crash report: DRED breadcrumbs and page fault data when a device that
// the game created reports removal, and a note when only the registers show it. `registers_show_removal` is
// true when a register held a DXGI device-removed HRESULT. Empty when neither applies.
std::string section(bool registers_show_removal);

}
