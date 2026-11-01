#pragma once

#include <cstdint>
#include <cstdio>
#include <string>
#include <string_view>

// Database keys of the track tables. A key is the FNV-1a-64 of its path, and the hash runs left to right, so a row's
// key continues its table's and a field's continues its row's. Some track tables are known only by the hash of their
// row prefix: their paths are written "{<16 hex digits>}<rest>", and path_key continues the hash from those digits.

namespace swofty_core::apply::tracks {

inline constexpr uint64_t FNV_BASIS = 0xcbf29ce484222325ull;

inline uint64_t fnv_from(uint64_t h, std::string_view s)
{
    for (unsigned char c : s) {
        h ^= c;
        h *= 0x100000001b3ull;
    }
    return h;
}

inline bool hashed_prefix(std::string_view path, uint64_t &h)
{
    if (path.size() < 18 || path[0] != '{' || path[17] != '}')
        return false;
    uint64_t v = 0;
    for (std::size_t i = 1; i < 17; i++) {
        char c = path[i];
        int d = c >= '0' && c <= '9' ? c - '0' : c >= 'a' && c <= 'f' ? c - 'a' + 10 : c >= 'A' && c <= 'F' ? c - 'A' + 10 : -1;
        if (d < 0)
            return false;
        v = v << 4 | uint64_t(d);
    }
    h = v;
    return true;
}

inline uint64_t path_key(std::string_view path)
{
    uint64_t h = 0;
    if (hashed_prefix(path, h))
        return fnv_from(h, path.substr(18));
    return fnv_from(FNV_BASIS, path);
}

inline std::string hex16(uint64_t h)
{
    char b[24];
    std::snprintf(b, sizeof b, "%016llx", static_cast<unsigned long long>(h));
    return b;
}

struct Key {
    uint64_t h = 0;
    std::string path;
};

inline Key key(std::string path)
{
    uint64_t h = fnv_from(FNV_BASIS, path);
    return {h, std::move(path)};
}

inline Key hashed(uint64_t h) { return {h, "{" + hex16(h) + "}"}; }

inline Key at(const Key &k, std::string_view suffix) { return {fnv_from(k.h, suffix), k.path + std::string(suffix)}; }
inline Key field(const Key &row, std::string_view name) { return at(row, "." + std::string(name)); }
inline Key row(const Key &table, std::size_t i) { return at(table, std::to_string(i)); }
inline Key sub_table(const Key &row, std::string_view name) { return at(row, "." + std::string(name) + "_"); }
inline Key sub_count(const Key &row, std::string_view name) { return at(row, "._" + std::string(name) + "_instancecount_"); }

}
