#include <winsock2.h>
#include <ws2tcpip.h>

#include "imports.hpp"
#include "runtime.hpp"

#include <safetyhook.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cctype>
#include <cstring>
#include <mutex>
#include <set>
#include <string>

// EA guard. A modded F1 25 runs offline and must never talk to EA, but the game still reaches out on
// its own: at boot and when an online-mode career starts it downloads
// https://f125prod.ea.com/F1OS2025/ServerStatus/ServerStatus.xml and calls AccountGameService/Login with
// the player's platform id. The guard takes the exe's import slots for the name lookups and HTTP
// connects (ws2_32 getaddrinfo and gethostbyname, winhttp WinHttpConnect, wininet InternetConnectA) and
// refuses EA's and Codemasters' hosts (ea.com, easports.com, tnt-ea.com, codemasters.com and their
// subdomains) as if they did not resolve, so nothing is sent; every other host passes through. Each
// refused host is logged once. It also answers the engine setting OnlineServices.OfflineMode with true:
// the online-login orchestrator's onStart (0x143369ea0) reads it and, when set, skips the whole EA chain
// (CheckPrivilegeTask, EALoginTask, OnlineServerLogInTask) and finishes as the game's own offline mode,
// so online-mode screens such as the two-player career no longer stop on "EA-Konto erforderlich".
// SWOFTY_EA_GUARD=0 turns both off.

namespace {

using GetAddrInfo_t = INT(PCSTR, PCSTR, const ADDRINFOA *, PADDRINFOA *);
using GetHostByName_t = hostent *(const char *);
using WinHttpConnect_t = void *(void *, const wchar_t *, uint16_t, DWORD);
using InternetConnectA_t = void *(void *, const char *, uint16_t, const char *, const char *, DWORD, DWORD,
                                         DWORD_PTR);

const runtime::Sig S_SETTING_BOOL{"Ne_Get_SettingBool",
                                  "41 56 56 57 53 48 83 EC 38 44 89 CB 4C 89 C7 49 89 CE 48 89 54 24 30 48 8D 71 10 "
                                  "48 89 F1 E8 ?? ?? ?? ?? 4D 8B 76 08 4D 85 F6",
                                  0x3a43b90};
safetyhook::InlineHook h_setting_bool;
std::atomic<bool> g_offline_logged;

uint64_t h_get_setting_bool(void *node, const char *name, uint8_t *out, uint8_t def)
{
    uint64_t r = h_setting_bool.call<uint64_t>(node, name, out, def);
    if (name && out && !strcmp(name, "OnlineServices.OfflineMode")) {
        *out = 1;
        if (!g_offline_logged.exchange(true))
            swofty::log("[ea-guard] OnlineServices.OfflineMode answered true: the game skips its EA sign-in");
        return 0;
    }
    return r;
}

GetAddrInfo_t *o_getaddrinfo;
GetHostByName_t *o_gethostbyname;
WinHttpConnect_t *o_winhttp_connect;
InternetConnectA_t *o_internet_connect;

std::mutex g_seen_lock;
std::set<std::string> g_seen;
std::atomic<unsigned> g_refused;

constexpr const char *BLOCKED[] = {"ea.com", "easports.com", "tnt-ea.com", "codemasters.com"};
constexpr DWORD WINHTTP_NAME_NOT_RESOLVED = 12007, INTERNET_NAME_NOT_RESOLVED = 12007;

bool blocked(std::string host)
{
    for (auto &c : host)
        c = char(std::tolower(static_cast<unsigned char>(c)));
    while (!host.empty() && host.back() == '.')
        host.pop_back();
    for (const char *d : BLOCKED) {
        size_t n = strlen(d);
        if (host == d || (host.size() > n && host.compare(host.size() - n, n, d) == 0 && host[host.size() - n - 1] == '.'))
            return true;
    }
    return false;
}

bool refuse(const std::string &host, const char *via)
{
    if (!blocked(host))
        return false;
    g_refused++;
    std::lock_guard<std::mutex> g(g_seen_lock);
    if (g_seen.insert(host).second)
        swofty::log("[ea-guard] refused %s (%s): a modded game sends EA nothing", host.c_str(), via);
    return true;
}

std::string narrow(const wchar_t *w)
{
    std::string s;
    for (; w && *w && s.size() < 255; w++)
        s += *w < 128 ? char(*w) : '?';
    return s;
}

INT WSAAPI h_getaddrinfo(PCSTR node, PCSTR service, const ADDRINFOA *hints, PADDRINFOA *res)
{
    if (node && refuse(node, "getaddrinfo")) {
        if (res)
            *res = nullptr;
        SetLastError(WSAHOST_NOT_FOUND); // what WSASetLastError does, without linking ws2_32
        return WSAHOST_NOT_FOUND;
    }
    return o_getaddrinfo(node, service, hints, res);
}

hostent *WSAAPI h_gethostbyname(const char *name)
{
    if (name && refuse(name, "gethostbyname")) {
        SetLastError(WSAHOST_NOT_FOUND); // what WSASetLastError does, without linking ws2_32
        return nullptr;
    }
    return o_gethostbyname(name);
}

void *WINAPI h_winhttp_connect(void *session, const wchar_t *server, uint16_t port, DWORD reserved)
{
    if (server && refuse(narrow(server), "WinHttpConnect")) {
        SetLastError(WINHTTP_NAME_NOT_RESOLVED);
        return nullptr;
    }
    return o_winhttp_connect(session, server, port, reserved);
}

void *WINAPI h_internet_connect(void *inet, const char *server, uint16_t port, const char *user, const char *pass,
                                DWORD service, DWORD flags, DWORD_PTR ctx)
{
    if (server && refuse(server, "InternetConnectA")) {
        SetLastError(INTERNET_NAME_NOT_RESOLVED);
        return nullptr;
    }
    return o_internet_connect(inet, server, port, user, pass, service, flags, ctx);
}

// Points every exe import slot for `name` in `dll` at `detour`; the first slot's previous value becomes
// the original. Returns the number of slots taken.
template <class T> int take(const wchar_t *dll_w, const char *dll, const char *name, void *detour, T **orig)
{
    HMODULE lib = LoadLibraryW(dll_w);
    void *target = lib ? reinterpret_cast<void *>(GetProcAddress(lib, name)) : nullptr;
    if (!target)
        return 0;
    int n = 0;
    runtime::each_import(lib, dll, [&](const runtime::ImportSlot &s) {
        if (s.fn != target)
            return;
        void *cur = reinterpret_cast<void *>(*s.slot);
        if (cur == detour)
            return;
        if (!*orig)
            *orig = reinterpret_cast<T *>(cur);
        if (runtime::write_slot(s.slot, detour))
            n++;
    });
    if (!*orig)
        *orig = reinterpret_cast<T *>(target);
    return n;
}

}

namespace runtime {

void ea_guard_setup()
{
    char v[8] = {};
    if (GetEnvironmentVariableA("SWOFTY_EA_GUARD", v, sizeof v) && v[0] == '0') {
        swofty::log("[ea-guard] off (SWOFTY_EA_GUARD=0)");
        return;
    }
    int a = take(L"ws2_32.dll", "ws2_32.dll", "getaddrinfo", reinterpret_cast<void *>(&h_getaddrinfo), &o_getaddrinfo);
    int b = take(L"ws2_32.dll", "ws2_32.dll", "gethostbyname", reinterpret_cast<void *>(&h_gethostbyname),
                 &o_gethostbyname);
    int c = take(L"winhttp.dll", "winhttp.dll", "WinHttpConnect", reinterpret_cast<void *>(&h_winhttp_connect),
                 &o_winhttp_connect);
    int d = take(L"wininet.dll", "wininet.dll", "InternetConnectA", reinterpret_cast<void *>(&h_internet_connect),
                 &o_internet_connect);
    if (uintptr_t at = runtime::find(S_SETTING_BOOL, "ea-guard", "core"))
        if (auto r = safetyhook::InlineHook::create(reinterpret_cast<void *>(at), reinterpret_cast<void *>(&h_get_setting_bool)))
            h_setting_bool = std::move(*r);
    swofty::log("[ea-guard] on: getaddrinfo %d slot(s), gethostbyname %d, WinHttpConnect %d, InternetConnectA %d; "
                "EA and Codemasters hosts are refused; offline mode setting %s",
                a, b, c, d, h_setting_bool ? "hooked" : "not found");
}

}
