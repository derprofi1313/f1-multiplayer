#pragma once

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <vector>

// The key the team logo archives are cached under. An install copies its archives into the game folder again before
// every session, and a rollback copies the stock ones back, so a write time changes while the bytes stay the same.
// Each archive counts by a hash of its bytes. The stamps file next to the cached archives keeps each hash under the
// size and write time it was taken at, and only an archive whose size or write time differs is read again.

namespace swofty_core::apply::content::logo_cache {

inline constexpr const char *STAMPS = "archives.stamps";

inline uint64_t stamp(uint64_t h, const std::string &s)
{
    for (unsigned char c : s)
        h = (h ^ c) * 0x100000001b3ull;
    return h * 0x100000001b3ull + 0x9e3779b97f4a7c15ull;
}

class Bytes {
public:
    void add(const char *p, size_t n)
    {
        total_ += n;
        while (have_ && n) {
            pending_[have_++] = *p++;
            n--;
            if (have_ == sizeof pending_) {
                block(pending_);
                have_ = 0;
            }
        }
        if (have_)
            return;
        for (; n >= sizeof pending_; p += sizeof pending_, n -= sizeof pending_)
            block(p);
        std::memcpy(pending_, p, n);
        have_ = n;
    }

    uint64_t done() const
    {
        uint64_t l[4] = {lane_[0], lane_[1], lane_[2], lane_[3]};
        for (size_t i = 0; i < have_; i++)
            l[i % 4] = mix(l[i % 4], uint8_t(pending_[i]) + 1);
        uint64_t h = total_ * 0x9e3779b97f4a7c15ull;
        for (auto v : l)
            h = mix(h, v);
        return h ^ (h >> 31);
    }

private:
    static uint64_t mix(uint64_t h, uint64_t v)
    {
        h = (h ^ v) * 0xff51afd7ed558ccdull;
        return h ^ (h >> 29);
    }

    void block(const char *p)
    {
        for (int i = 0; i < 4; i++) {
            uint64_t v;
            std::memcpy(&v, p + 8 * i, 8);
            lane_[i] = mix(lane_[i], v);
        }
    }

    uint64_t lane_[4] = {0x243f6a8885a308d3ull, 0x13198a2e03707344ull, 0xa4093822299f31d0ull, 0x082efa98ec4e6c89ull};
    char pending_[32];
    size_t have_ = 0;
    uint64_t total_ = 0;
};

struct Seen {
    uint64_t size = 0, time = 0, hash = 0;
    bool hashed = false;
};

using Stamps = std::map<std::string, Seen>;

inline Stamps parse(const std::string &text)
{
    Stamps out;
    size_t at = 0;
    while (at < text.size()) {
        size_t end = text.find('\n', at);
        if (end == std::string::npos)
            end = text.size();
        std::string line = text.substr(at, end - at);
        at = end + 1;
        std::vector<std::string> f;
        for (size_t from = 0;;) {
            size_t tab = line.find('\t', from);
            f.push_back(line.substr(from, tab == std::string::npos ? std::string::npos : tab - from));
            if (tab == std::string::npos)
                break;
            from = tab + 1;
        }
        if (f.size() != 4 || f[0].empty())
            continue;
        Seen s;
        char *stop = nullptr;
        s.size = std::strtoull(f[1].c_str(), &stop, 10);
        if (*stop)
            continue;
        s.time = std::strtoull(f[2].c_str(), &stop, 10);
        if (*stop)
            continue;
        if (f[3] != "-") {
            s.hash = std::strtoull(f[3].c_str(), &stop, 16);
            if (*stop || f[3].empty())
                continue;
            s.hashed = true;
        }
        out[f[0]] = s;
    }
    return out;
}

inline std::string text(const Stamps &stamps)
{
    std::string out;
    char hex[17];
    for (auto &[rel, s] : stamps) {
        std::snprintf(hex, sizeof hex, "%016llx", static_cast<unsigned long long>(s.hash));
        out += rel + "\t" + std::to_string(s.size) + "\t" + std::to_string(s.time) + "\t" + (s.hashed ? hex : "-") + "\n";
    }
    return out;
}

struct Env {
    std::function<bool(const std::string &rel, uint64_t &size, uint64_t &time)> stat;
    std::function<bool(const std::string &rel, uint64_t &hash)> hash;
};

struct Keyed {
    uint64_t key = 0;
    Stamps stamps;
    size_t hashed = 0;
    uint64_t hashed_bytes = 0;
};

inline Keyed key_archives(uint64_t key, const std::vector<std::string> &files, const Env &env, const Stamps &before)
{
    Keyed out;
    for (auto &rel : files) {
        Seen now;
        if (!env.stat(rel, now.size, now.time))
            continue;
        auto was = before.find(rel);
        if (was != before.end() && was->second.hashed && was->second.size == now.size && was->second.time == now.time) {
            now = was->second;
        } else if (env.hash(rel, now.hash)) {
            now.hashed = true;
            out.hashed++;
            out.hashed_bytes += now.size;
        }
        char hex[17];
        std::snprintf(hex, sizeof hex, "%016llx", static_cast<unsigned long long>(now.hash));
        key = stamp(key, rel + "|" + std::to_string(now.size) + "|" + (now.hashed ? std::string("h") + hex : "t" + std::to_string(now.time)));
        out.stamps[rel] = now;
    }
    out.key = key;
    return out;
}

}
