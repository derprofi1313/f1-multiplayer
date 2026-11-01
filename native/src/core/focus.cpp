#include "runtime.hpp"

#include <swofty/mixin.hpp>

#include <atomic>

// SWOFTY_KEEP_FOCUS=1 keeps the game running when its window loses focus, for headless test runs. The
// game's window procedure (NeWindowWin::winProc) calls setOnFocus(false) on WM_KILLFOCUS and reacts to
// WM_ACTIVATE(WA_INACTIVE) and WM_ACTIVATEAPP(FALSE); a race then stops at the pause menu. The core
// subclasses that window and drops those three messages, and hands the game one WM_SETFOCUS when the
// window starts without focus.

namespace {

std::atomic<WNDPROC> original;
std::atomic<unsigned> dropped[3];

LRESULT CALLBACK keep_focus_proc(HWND wnd, UINT msg, WPARAM wp, LPARAM lp)
{
    int kind = msg == WM_KILLFOCUS                                        ? 0
               : msg == WM_ACTIVATE && LOWORD(wp) == WA_INACTIVE          ? 1
               : msg == WM_ACTIVATEAPP && !wp                             ? 2
                                                                          : -1;
    if (kind >= 0) {
        static const char *names[] = {"WM_KILLFOCUS", "WM_ACTIVATE(WA_INACTIVE)", "WM_ACTIVATEAPP(FALSE)"};
        if (dropped[kind]++ == 0)
            swofty::log("[focus] dropped %s; the game keeps running without focus (SWOFTY_KEEP_FOCUS)", names[kind]);
        return 0;
    }
    return CallWindowProcW(original.load(), wnd, msg, wp, lp);
}

struct Find {
    uintptr_t lo, hi;
    HWND found;
};

BOOL CALLBACK find_window(HWND wnd, LPARAM lp)
{
    auto *f = reinterpret_cast<Find *>(lp);
    DWORD pid = 0;
    GetWindowThreadProcessId(wnd, &pid);
    if (pid != GetCurrentProcessId())
        return TRUE;
    auto proc = static_cast<uintptr_t>(GetWindowLongPtrW(wnd, GWLP_WNDPROC));
    if (proc < f->lo || proc >= f->hi)
        return TRUE;
    f->found = wnd;
    return FALSE;
}

DWORD WINAPI watch(void *)
{
    auto base = reinterpret_cast<uintptr_t>(GetModuleHandleW(nullptr));
    auto *dos = reinterpret_cast<const IMAGE_DOS_HEADER *>(base);
    auto *nt = reinterpret_cast<const IMAGE_NT_HEADERS *>(base + dos->e_lfanew);
    Find f{base, base + nt->OptionalHeader.SizeOfImage, nullptr};
    for (int i = 0; i < 1200 && !f.found; i++) {
        EnumWindows(find_window, reinterpret_cast<LPARAM>(&f));
        if (!f.found)
            Sleep(100);
    }
    if (!f.found) {
        swofty::log("[focus] SWOFTY_KEEP_FOCUS: no window with the game's window procedure appeared in 120 s");
        return 0;
    }
    WNDPROC game = reinterpret_cast<WNDPROC>(GetWindowLongPtrW(f.found, GWLP_WNDPROC));
    original.store(game);
    SetWindowLongPtrW(f.found, GWLP_WNDPROC, reinterpret_cast<LONG_PTR>(&keep_focus_proc));
    bool focused = GetForegroundWindow() == f.found;
    swofty::log("[focus] SWOFTY_KEEP_FOCUS: window %p (procedure %p) subclassed; WM_KILLFOCUS, "
                "WM_ACTIVATE(WA_INACTIVE) and WM_ACTIVATEAPP(FALSE) are dropped; the window is %s",
                static_cast<void *>(f.found), reinterpret_cast<void *>(game),
                focused ? "in the foreground" : "not in the foreground, so the game gets one WM_SETFOCUS");
    if (!focused)
        SendMessageW(f.found, WM_SETFOCUS, 0, 0);
    return 0;
}

}

namespace runtime {

void focus_setup()
{
    char v[8] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_KEEP_FOCUS", v, sizeof v);
    if (!n || n >= sizeof v || v[0] != '1')
        return;
    HANDLE t = CreateThread(nullptr, 0, watch, nullptr, 0, nullptr);
    if (t)
        CloseHandle(t);
}

}
