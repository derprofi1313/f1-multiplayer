#include "virtual_pad.hpp"

#include "runtime.hpp"

#include <safetyhook.hpp>

#include <atomic>
#include <cstring>
#include <mutex>

namespace {

struct XPad {
    uint16_t buttons;
    uint8_t lt, rt;
    int16_t lx, ly, rx, ry;
};
struct XState {
    DWORD packet;
    XPad pad;
};
struct XCaps {
    uint8_t type, subtype;
    uint16_t flags;
    XPad pad;
    uint16_t vib_l, vib_r;
};

std::once_flag once;
std::atomic<int> claims{0};
std::atomic<uint32_t> pad_word{0};
std::atomic<bool> pad_held{false};
std::atomic<DWORD> pad_packet{1};
safetyhook::InlineHook h_xget, h_xcaps;
std::string pad_why = "not set up";

DWORD WINAPI x_get_state(DWORD user, XState *s)
{
    DWORD r = h_xget.call<DWORD>(user, s);
    if (user != 0 || !s || claims.load() <= 0)
        return r;
    if (pad_held.load() || r != ERROR_SUCCESS) {
        uint32_t w = pad_word.load();
        std::memset(s, 0, sizeof *s);
        s->packet = pad_packet.load();
        s->pad.buttons = uint16_t(w);
        s->pad.lt = uint8_t(w >> 16);
        s->pad.rt = uint8_t(w >> 24);
    }
    return ERROR_SUCCESS;
}

DWORD WINAPI x_get_caps(DWORD user, DWORD flags, XCaps *c)
{
    DWORD r = h_xcaps.call<DWORD>(user, flags, c);
    if (user != 0 || !c || claims.load() <= 0 || r == ERROR_SUCCESS)
        return r;
    std::memset(c, 0, sizeof *c);
    c->type = 1;
    c->subtype = 1;
    c->pad.buttons = 0xf3ff;
    c->pad.lt = c->pad.rt = 0xff;
    return ERROR_SUCCESS;
}

void pad_setup()
{
    HMODULE x = LoadLibraryW(L"xinput1_4.dll");
    void *get = x ? reinterpret_cast<void *>(GetProcAddress(x, MAKEINTRESOURCEA(2))) : nullptr;
    void *caps = x ? reinterpret_cast<void *>(GetProcAddress(x, MAKEINTRESOURCEA(4))) : nullptr;
    if (!get || !caps) {
        pad_why = "xinput1_4.dll or its XInputGetState/XInputGetCapabilities exports were not found";
        return;
    }
    auto g = safetyhook::InlineHook::create(get, reinterpret_cast<void *>(x_get_state));
    auto c = safetyhook::InlineHook::create(caps, reinterpret_cast<void *>(x_get_caps));
    if (!g || !c) {
        pad_why = "hooking xinput1_4.dll's exports failed";
        return;
    }
    h_xget = std::move(*g);
    h_xcaps = std::move(*c);
    pad_why.clear();
}

}

namespace runtime::pad {

void setup()
{
    std::call_once(once, pad_setup);
}

const std::string &why()
{
    return pad_why;
}

void claim(bool on)
{
    claims.fetch_add(on ? 1 : -1);
}

void set(uint16_t buttons, uint8_t lt, uint8_t rt)
{
    pad_word = uint32_t(buttons) | uint32_t(lt) << 16 | uint32_t(rt) << 24;
    pad_held = buttons || lt || rt;
    pad_packet.fetch_add(1);
}

void release()
{
    set(0);
}

}
