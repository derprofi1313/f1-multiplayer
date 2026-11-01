#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>

#include "imports.hpp"
#include "runtime.hpp"
#include "udp_ids_logic.hpp"

#include <swofty/mixin.hpp>

#include <atomic>
#include <cstdio>
#include <string>

// UDP telemetry reports the stat ids of added teams and drivers, which overlays that know EA's ids show
// as unknown. The game sends its finished telemetry packets with sendto through its ws2_32 import slots
// (both import descriptors, like the other socket hooks). This hook copies a buffer that parses as an F1
// packet (udp_ids_logic.hpp), replaces the ids swofty/udp_ids.json maps, and sends the copy; every other
// call passes through with its arguments. Datagrams to the LAN game port 21414 and to 10.77.0.0/16 are
// never looked at. SWOFTY_UDP_STOCK_IDS=0 turns it off.

namespace {

namespace ids = runtime::udp_ids;

constexpr uint16_t GAME_PORT = 21414;
constexpr size_t LINES = 12;

using SendToFn = int(WSAAPI *)(SOCKET, const char *, int, int, const sockaddr *, int);

void *detour_target;
void *orig_sendto;
ids::Map map;
std::atomic<bool> armed;
std::atomic<unsigned> said;
std::atomic<int> seen_formats;
std::atomic<uint64_t> odd_sizes[4];
std::atomic<unsigned> trial_said;

bool lan_traffic(const sockaddr *to, int tolen)
{
    if (!to || tolen < int(sizeof(sockaddr_in)) || to->sa_family != AF_INET)
        return false;
    auto *in = reinterpret_cast<const sockaddr_in *>(to);
    uint16_t port = __builtin_bswap16(in->sin_port);
    uint32_t ip = __builtin_bswap32(in->sin_addr.s_addr);
    return port == GAME_PORT || (ip & 0xffff0000) == 0x0a4d0000;
}

void first(int format, int id, const std::vector<ids::Change> &changes)
{
    unsigned bit = 1u << (format - 2024 & 3);
    if (said.fetch_or(bit) & bit)
        return;
    DWORD err = GetLastError();
    std::string what;
    for (size_t i = 0; i < changes.size() && i < 4; i++)
        what += (i ? ", " : "") + std::string(changes[i].driver ? "driver " : "team ") + std::to_string(changes[i].from) +
                " -> " + std::to_string(changes[i].to);
    swofty::log("[udp] first telemetry packet rewritten: format %d, packet %d, %s model: %s", format, id,
                map.model == ids::MODEL_2025 ? "2025" : map.model == ids::MODEL_2026 ? "2026" : "unknown", what.c_str());
    SetLastError(err);
}

void odd(const ids::Layout *l, int len)
{
    int slot = l->format - 2024;
    if (slot < 0 || slot > 2 || l->id > 63)
        return;
    uint64_t bit = uint64_t(1) << l->id;
    if (odd_sizes[slot].fetch_or(bit) & bit)
        return;
    DWORD err = GetLastError();
    swofty::log("[udp] format %d packet %d is %d bytes, not the %d the id rewrite expects; left alone", l->format, l->id,
                len, int(l->size));
    SetLastError(err);
}

void trial(const ids::Layout *l, const uint8_t *p, bool changed)
{
    unsigned bit = 1u << (l->format - 2024 & 3);
    if (l->id != 14 || (trial_said.fetch_or(bit) & bit))
        return;
    DWORD err = GetLastError();
    std::string teams;
    for (size_t i = 0; i < l->max_entries; i++) {
        const uint8_t *at = p + ids::HEADER + i * l->stride + l->team;
        teams += (i ? ", " : "") + std::to_string(l->wide ? at[0] | at[1] << 8 : at[0]);
    }
    swofty::log("[udp] first time trial packet: format %d, team ids %s, %s", l->format, teams.c_str(),
                changed ? "rewritten" : "not rewritten");
    SetLastError(err);
}

int WSAAPI h_sendto(SOCKET s, const char *buf, int len, int flags, const sockaddr *to, int tolen)
{
    auto real = reinterpret_cast<SendToFn>(orig_sendto);
    if (!armed.load(std::memory_order_relaxed) || !buf || len < int(ids::HEADER) + 1 || len > int(ids::MAX_PACKET) ||
        lan_traffic(to, tolen))
        return real(s, buf, len, flags, to, tolen);
    if (!ids::identify(reinterpret_cast<const uint8_t *>(buf), size_t(len))) {
        if (const ids::Layout *l = ids::misfit(reinterpret_cast<const uint8_t *>(buf), size_t(len)))
            odd(l, len);
        return real(s, buf, len, flags, to, tolen);
    }
    uint8_t copy[ids::MAX_PACKET];
    memcpy(copy, buf, size_t(len));
    std::vector<ids::Change> changes;
    int format = 0, id = 0;
    const ids::Layout *l = ids::identify(copy, size_t(len));
    if (!ids::rewrite(map, copy, size_t(len), &changes, &format, &id)) {
        trial(l, reinterpret_cast<const uint8_t *>(buf), false);
        return real(s, buf, len, flags, to, tolen);
    }
    trial(l, reinterpret_cast<const uint8_t *>(buf), true);
    first(format, id, changes);
    return real(s, reinterpret_cast<const char *>(copy), len, flags, to, tolen);
}

bool wanted_off()
{
    char v[16] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_UDP_STOCK_IDS", v, sizeof v);
    if (!n || n >= sizeof v || (strcmp(v, "0") && strcmp(v, "off")))
        return false;
    swofty::log("[udp] stock ids off (SWOFTY_UDP_STOCK_IDS=%s)", v);
    return true;
}

void setup()
{
    if (wanted_off())
        return;
    std::string text = runtime::read_file(runtime::swofty_root() + L"udp_ids.json", size_t(4) << 20);
    if (text.empty())
        return;
    std::string why;
    if (!ids::load(text, map, why)) {
        swofty::log("[udp] swofty\\udp_ids.json is unusable (%s); telemetry ids stay as the game sends them", why.c_str());
        return;
    }
    if (!map.any)
        return;
    HMODULE ws = LoadLibraryW(L"ws2_32.dll");
    detour_target = ws ? reinterpret_cast<void *>(GetProcAddress(ws, "sendto")) : nullptr;
    int slots = 0;
    if (ws)
        runtime::each_import(ws, "ws2_32.dll", [&](const runtime::ImportSlot &s) {
            if (!s.fn || s.fn != detour_target)
                return;
            void *cur = reinterpret_cast<void *>(*s.slot);
            if (orig_sendto && orig_sendto != cur) {
                swofty::log("[udp] sendto table %d slot exe+0x%llx holds 0x%llx, not 0x%llx; left alone", s.table,
                            (unsigned long long)s.rva, (unsigned long long)uintptr_t(cur),
                            (unsigned long long)uintptr_t(orig_sendto));
                return;
            }
            void *was = orig_sendto;
            orig_sendto = cur;
            if (!runtime::write_slot(s.slot, reinterpret_cast<void *>(&h_sendto))) {
                orig_sendto = was;
                return;
            }
            slots++;
        });
    if (!slots) {
        swofty::log("[udp] no game import slot for sendto; telemetry ids stay as the game sends them");
        return;
    }
    armed = true;
    std::string head = std::to_string(map.teams.size()) + " team id(s), " + std::to_string(map.drivers.size()) +
                       " driver id(s)";
    swofty::log("[udp] stock ids on: %s, %d sendto slot(s)", head.c_str(), slots);
    std::string line;
    size_t shown = 0;
    for (auto &l : map.lines) {
        if (shown++ == LINES)
            break;
        line += (line.empty() ? "" : "; ") + l;
    }
    if (!line.empty())
        swofty::log("[udp] stock ids: %s%s", line.c_str(), map.lines.size() > LINES ? "; ..." : "");
}

}

namespace runtime {

void udp_ids_setup()
{
    setup();
}

}
