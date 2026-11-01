#pragma once
#include "json.hpp"

#include <cstdint>
#include <cstring>
#include <map>
#include <string>
#include <vector>

// Rewrites the team and driver ids of the game's outgoing UDP telemetry packets. The launcher writes
// swofty/udp_ids.json (packages/core/src/udp_ids.rs): added team id -> stock ancestor id, added driver id ->
// stock ancestor id or 255, each as a full table (`teams`, `drivers`, read by the 16-bit 2026 format) and
// as a byte table (`teams_byte`, `drivers_byte`, read by the 2024 and 2025 formats, which carry one byte).
//
// A buffer is touched only when it parses as an F1 packet: m_packetFormat (u16 at 0) is 2024, 2025 or
// 2026, m_packetId (u8 at 6) is one of the packets below, and the length is exactly that packet's size in
// that format. Fields rewritten, offsets from the packet start:
//   Participants (4), 2024: 1350 bytes; numActiveCars at 29, entries from 30, 60 bytes each, driverId at
//     entry+1, teamId at entry+3, both u8.
//   Participants (4), 2025: 1284 bytes; the same, 57 bytes each.
//   Participants (4), 2026: 1470 bytes; entries from 30, 60 bytes each, 24 cars; driverId u16 at entry+1,
//     teamId u16 at entry+5 (the game copies its internal record whole; the 2025 builder reads the same
//     record's driver id from +1 and team id from +5).
//   Lobby Info (9), 2024: 1306 bytes; numPlayers at 29, entries from 30, 58 bytes each, teamId u8 at entry+1.
//   Lobby Info (9), 2025: 954 bytes; 42 bytes each, teamId u8 at entry+1.
//   Lobby Info (9), 2026: 1062 bytes; 24 entries from 30, 43 bytes each, teamId u16 at entry+1 (the builder
//     Game_TelemetryOutputDataSystem_UpdateUDPLobbyData 0x142d033d0 starts every entry's team at 0xffff and writes
//     the team stat id as a u16 at entry+1; the name starts at entry+5, one byte later than in 2025).
// The 2026 format sends each stock team's stat id: 0-9 in a 2025-model session (the 2025 ids) and 476-486 in
// a 2026-model session (Mercedes 476, Ferrari 477, Red Bull 478, Williams 479, Aston Martin 480, Alpine 481, RB 482,
// Haas 483, McLaren 484, Audi 485, Cadillac 486; Box-Box f1_26 TEAMS), so an added team's stock 2025 ancestor id n
// is sent as 476 + n unless the last Participants or Lobby packet showed stock ids 0-9 only.
//   Time Trial (14), 2024 and 2025: 101 bytes; three data sets of 24 bytes from 29, teamId u8 at set+1.
//   Time Trial (14), 2026: 104 bytes; three data sets of 25 bytes from 29, teamId u16 at set+1 (the game keeps one
//     2026-shaped record and Game_TelemetryUdp_Prepare2025 0x144733850 copies the low byte of that u16 into the
//     2025 packet and skips the high byte).
// Sizes come from the game's own table in Game_TelemetryUdp_LogPacketSizes 0x14472a6e0 and the destination
// offsets of Game_TelemetryOutputUDPUpdateable_PreparePackets 0x144736210. No other 2026 packet carries a team or
// driver id: Final Classification (1134), Session History (1460), Car Setups (1233), Car Damage (1133), Car
// Status (1445), Car Telemetry (1448), Lap Data (1399), Lap Positions (1231), Tyre Sets (231), Motion Ex (273)
// and Event (45) index cars by carIdx only.

namespace runtime::udp_ids {

constexpr size_t HEADER = 29;
constexpr size_t MAX_PACKET = 1500;
constexpr int NETWORK_DRIVER = 255;
constexpr int STOCK_2025_LAST = 9;
constexpr int STOCK_2026_FIRST = 476;
constexpr int STOCK_2026_LAST = 486;
constexpr int MODEL_UNKNOWN = 0;
constexpr int MODEL_2025 = 25;
constexpr int MODEL_2026 = 26;

struct Map {
    int16_t team8[256];
    int16_t driver8[256];
    std::map<int, int> teams, drivers;
    std::vector<std::string> lines;
    bool any = false;
    mutable int model = MODEL_UNKNOWN;

    Map()
    {
        for (auto &v : team8)
            v = -1;
        for (auto &v : driver8)
            v = -1;
    }
};

inline bool load(const std::string &text, Map &out, std::string &why)
{
    json::Value v;
    if (!json::parse(text, v) || v.type != json::Value::Obj) {
        why = "not a JSON object";
        return false;
    }
    if (v["version"].num() != 1) {
        why = "unknown version";
        return false;
    }
    Map m;
    auto table = [&](const char *key, std::map<int, int> &full) {
        for (auto &[k, to] : v[key].o) {
            long long id = strtoll(k.c_str(), nullptr, 10), t = to.num(-1);
            if (id >= 0 && id <= 0xffff && t >= 0 && t <= 0xffff)
                full[int(id)] = int(t);
        }
    };
    table("teams", m.teams);
    table("drivers", m.drivers);
    for (auto &[k, to] : v["teams_byte"].o) {
        long long id = strtoll(k.c_str(), nullptr, 10), t = to.num(-1);
        if (id >= 0 && id < 256 && t >= 0 && t < 256)
            m.team8[id] = int16_t(t);
    }
    for (auto &[k, to] : v["drivers_byte"].o) {
        long long id = strtoll(k.c_str(), nullptr, 10), t = to.num(-1);
        if (id >= 0 && id < 256 && t >= 0 && t < 256)
            m.driver8[id] = int16_t(t);
    }
    for (auto &l : v["lines"].a)
        m.lines.push_back(l.str());
    m.any = !m.teams.empty() || !m.drivers.empty();
    out = std::move(m);
    return true;
}

struct Layout {
    int format, id;
    size_t size, stride, max_entries;
    size_t driver, team;
    bool wide;
    bool fixed_sets;
};

inline const Layout *layouts(size_t &n)
{
    static const Layout table[] = {
        {2024, 4, 1350, 60, 22, 1, 3, false, false},
        {2025, 4, 1284, 57, 22, 1, 3, false, false},
        {2026, 4, 1470, 60, 24, 1, 5, true, false},
        {2024, 9, 1306, 58, 22, 0, 1, false, false},
        {2025, 9, 954, 42, 22, 0, 1, false, false},
        {2024, 14, 101, 24, 3, 0, 1, false, true},
        {2025, 14, 101, 24, 3, 0, 1, false, true},
        {2026, 9, 1062, 43, 24, 0, 1, true, false},
        {2026, 14, 104, 25, 3, 0, 1, true, true},
    };
    n = sizeof table / sizeof *table;
    return table;
}

inline const Layout *identify(const uint8_t *p, size_t len)
{
    if (!p || len < HEADER + 1 || len > MAX_PACKET)
        return nullptr;
    int format = p[0] | p[1] << 8;
    if (format != 2024 && format != 2025 && format != 2026)
        return nullptr;
    size_t n;
    const Layout *t = layouts(n);
    for (size_t i = 0; i < n; i++)
        if (t[i].format == format && t[i].id == p[6] && t[i].size == len)
            return &t[i];
    return nullptr;
}

inline const Layout *misfit(const uint8_t *p, size_t len)
{
    if (!p || len < HEADER + 1 || len > MAX_PACKET)
        return nullptr;
    int format = p[0] | p[1] << 8;
    size_t n;
    const Layout *t = layouts(n);
    const Layout *other = nullptr;
    for (size_t i = 0; i < n; i++) {
        if (t[i].format != format || t[i].id != p[6])
            continue;
        if (t[i].size == len)
            return nullptr;
        other = &t[i];
    }
    return other;
}

inline int to_2026_team(int stock)
{
    return stock >= 0 && stock <= STOCK_2025_LAST ? STOCK_2026_FIRST + stock : stock;
}

inline void learn_model(const Map &m, const Layout &l, const uint8_t *p, size_t count, size_t first)
{
    if (!l.wide || (l.id != 4 && l.id != 9))
        return;
    bool s25 = false, s26 = false;
    for (size_t i = 0; i < count; i++) {
        const uint8_t *at = p + first + i * l.stride + l.team;
        int id = at[0] | at[1] << 8;
        if (m.teams.count(id))
            continue;
        s26 |= id >= STOCK_2026_FIRST && id <= STOCK_2026_LAST;
        s25 |= id <= STOCK_2025_LAST;
    }
    if (s26)
        m.model = MODEL_2026;
    else if (s25)
        m.model = MODEL_2025;
}

struct Change {
    bool driver;
    int from, to;
};

inline bool rewrite(const Map &m, uint8_t *p, size_t len, std::vector<Change> *changes = nullptr, int *format = nullptr,
                    int *id = nullptr)
{
    if (!m.any)
        return false;
    const Layout *l = identify(p, len);
    if (!l)
        return false;
    if (format)
        *format = l->format;
    if (id)
        *id = l->id;
    size_t count = l->max_entries;
    size_t first = HEADER;
    if (!l->fixed_sets) {
        count = p[HEADER];
        first = HEADER + 1;
        if (count > l->max_entries)
            return false;
    }
    learn_model(m, *l, p, count, first);
    bool changed = false;
    auto edit = [&](uint8_t *at, bool driver) {
        int from = l->wide ? at[0] | at[1] << 8 : at[0], to = -1;
        if (l->wide) {
            auto &t = driver ? m.drivers : m.teams;
            auto it = t.find(from);
            to = it == t.end() ? -1 : it->second;
            if (!driver && to >= 0 && m.model != MODEL_2025)
                to = to_2026_team(to);
        } else {
            to = driver ? m.driver8[from] : m.team8[from];
        }
        if (to < 0 || to == from)
            return;
        at[0] = uint8_t(to);
        if (l->wide)
            at[1] = uint8_t(to >> 8);
        changed = true;
        if (changes)
            changes->push_back({driver, from, to});
    };
    for (size_t i = 0; i < count; i++) {
        uint8_t *e = p + first + i * l->stride;
        if (l->id == 4)
            edit(e + l->driver, true);
        edit(e + l->team, false);
    }
    return changed;
}

}
