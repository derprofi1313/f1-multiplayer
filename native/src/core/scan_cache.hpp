#pragma once
#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

namespace scancache {

constexpr int SCHEMA = 1;
constexpr const char *MAGIC = "swofty-scan-cache";

inline uint64_t fnv64(const void *p, size_t n, uint64_t h = 0xcbf29ce484222325ull)
{
    auto b = static_cast<const uint8_t *>(p);
    for (size_t i = 0; i < n; i++)
        h = (h ^ b[i]) * 0x100000001b3ull;
    return h;
}

inline std::vector<int> tokens(const std::string &pattern)
{
    std::vector<int> out;
    bool fixed = false;
    for (size_t p = 0; p < pattern.size();) {
        if (pattern[p] == ' ') {
            p++;
            continue;
        }
        size_t e = pattern.find(' ', p);
        std::string tok = pattern.substr(p, e == std::string::npos ? std::string::npos : e - p);
        p = e == std::string::npos ? pattern.size() : e;
        if (tok[0] == '?') {
            out.push_back(-1);
            continue;
        }
        char *end = nullptr;
        long v = strtol(tok.c_str(), &end, 16);
        if (*end || tok.size() > 2 || v < 0 || v > 255)
            return {};
        out.push_back(int(v));
        fixed = true;
    }
    return fixed ? out : std::vector<int>{};
}

inline bool matches(const uint8_t *at, const std::vector<int> &pat)
{
    for (size_t k = 0; k < pat.size(); k++)
        if (pat[k] >= 0 && at[k] != pat[k])
            return false;
    return !pat.empty();
}

struct Cache {
    uint64_t key = 0;
    std::unordered_map<std::string, uint64_t> rva;
    bool dirty = false;

    static std::string id(const std::string &section, const std::string &pattern) { return section + '\t' + pattern; }

    bool find(const std::string &section, const std::string &pattern, uint64_t &out) const
    {
        auto it = rva.find(id(section, pattern));
        if (it == rva.end())
            return false;
        out = it->second;
        return true;
    }

    void found(const std::string &section, const std::string &pattern, int hits, uint64_t at)
    {
        auto k = id(section, pattern);
        auto it = rva.find(k);
        if (hits == 1) {
            if (it == rva.end() || it->second != at)
                rva[k] = at, dirty = true;
        } else if (it != rva.end()) {
            rva.erase(it);
            dirty = true;
        }
    }

    std::string serialize() const
    {
        std::vector<std::pair<std::string, uint64_t>> rows(rva.begin(), rva.end());
        std::sort(rows.begin(), rows.end());
        char line[64];
        snprintf(line, sizeof line, "%s\t%d\t%016llx\n", MAGIC, SCHEMA, (unsigned long long)key);
        std::string out = line;
        for (auto &[k, v] : rows) {
            snprintf(line, sizeof line, "%llx\t", (unsigned long long)v);
            out += line + k + '\n';
        }
        return out;
    }

    bool parse(const std::string &text, uint64_t want)
    {
        rva.clear();
        key = want;
        dirty = false;
        size_t nl = text.find('\n');
        char head[64];
        snprintf(head, sizeof head, "%s\t%d\t%016llx", MAGIC, SCHEMA, (unsigned long long)want);
        if (nl == std::string::npos || text.compare(0, nl, head) != 0) {
            dirty = true;
            return false;
        }
        for (size_t p = nl + 1; p < text.size();) {
            size_t e = text.find('\n', p);
            std::string row = text.substr(p, e == std::string::npos ? std::string::npos : e - p);
            p = e == std::string::npos ? text.size() : e + 1;
            size_t t1 = row.find('\t'), t2 = t1 == std::string::npos ? t1 : row.find('\t', t1 + 1);
            if (t2 == std::string::npos || t1 == 0) {
                dirty = true;
                continue;
            }
            std::string num = row.substr(0, t1), section = row.substr(t1 + 1, t2 - t1 - 1), pattern = row.substr(t2 + 1);
            char *end = nullptr;
            unsigned long long v = strtoull(num.c_str(), &end, 16);
            if (*end || section.empty() || tokens(pattern).empty()) {
                dirty = true;
                continue;
            }
            rva[id(section, pattern)] = v;
        }
        return true;
    }
};

}
