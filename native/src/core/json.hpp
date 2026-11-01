#pragma once
#include <cstdlib>
#include <cstring>
#include <string>
#include <tuple>
#include <utility>
#include <vector>

namespace json {

struct Value {
    enum Type { Null, Bool, Num, Str, Arr, Obj } type = Null;
    bool b = false;
    double n = 0;
    std::string s;
    std::vector<Value> a;
    std::vector<std::pair<std::string, Value>> o;

    const Value &operator[](const char *key) const
    {
        static const Value none;
        for (auto &[k, v] : o)
            if (k == key)
                return v;
        return none;
    }
    const Value &field(const char *key, const char *legacy) const
    {
        const Value &v = (*this)[key];
        return v.type == Null ? (*this)[legacy] : v;
    }
    std::string str(const char *def = "") const { return type == Str ? s : def; }
    long long num(long long def = 0) const
    {
        return type == Num ? (long long)n : type == Str ? strtoll(s.c_str(), nullptr, 0) : def;
    }
};

struct Parser {
    const char *p;

    void ws()
    {
        while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r')
            p++;
    }

    bool hex4(unsigned &cp)
    {
        cp = 0;
        for (int i = 0; i < 4; i++, p++) {
            char c = *p;
            int d = c >= '0' && c <= '9' ? c - '0' : c >= 'a' && c <= 'f' ? c - 'a' + 10 : c >= 'A' && c <= 'F' ? c - 'A' + 10 : -1;
            if (d < 0)
                return false;
            cp = cp << 4 | unsigned(d);
        }
        return true;
    }

    static void utf8(std::string &out, unsigned cp)
    {
        if (cp < 0x80)
            out += char(cp);
        else if (cp < 0x800)
            out += char(0xc0 | cp >> 6), out += char(0x80 | (cp & 0x3f));
        else if (cp < 0x10000)
            out += char(0xe0 | cp >> 12), out += char(0x80 | (cp >> 6 & 0x3f)), out += char(0x80 | (cp & 0x3f));
        else
            out += char(0xf0 | cp >> 18), out += char(0x80 | (cp >> 12 & 0x3f)), out += char(0x80 | (cp >> 6 & 0x3f)),
                out += char(0x80 | (cp & 0x3f));
    }

    bool string(std::string &out)
    {
        if (*p++ != '"')
            return false;
        while (*p && *p != '"') {
            char c = *p++;
            if (c == '\\') {
                if (!*p)
                    return false;
                c = *p++;
                if (c == 'u') {
                    unsigned cp;
                    if (!hex4(cp))
                        return false;
                    if (cp >= 0xd800 && cp < 0xdc00 && p[0] == '\\' && p[1] == 'u') {
                        const char *save = p;
                        unsigned lo;
                        p += 2;
                        if (hex4(lo) && lo >= 0xdc00 && lo < 0xe000)
                            cp = 0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00);
                        else
                            p = save;
                    }
                    if (!cp || (cp >= 0xd800 && cp < 0xe000))
                        cp = 0xfffd;
                    utf8(out, cp);
                    continue;
                }
                c = c == 'n' ? '\n' : c == 't' ? '\t' : c == 'r' ? '\r' : c == 'b' ? '\b' : c == 'f' ? '\f' : c;
            }
            out += c;
        }
        return *p++ == '"';
    }

    bool value(Value &v)
    {
        ws();
        if (*p == '{') {
            v.type = Value::Obj;
            p++;
            ws();
            if (*p == '}')
                return p++, true;
            do {
                ws();
                std::string k;
                Value x;
                if (!string(k))
                    return false;
                ws();
                if (*p++ != ':' || !value(x))
                    return false;
                v.o.emplace_back(std::move(k), std::move(x));
                ws();
            } while (*p == ',' && p++);
            return *p++ == '}';
        }
        if (*p == '[') {
            v.type = Value::Arr;
            p++;
            ws();
            if (*p == ']')
                return p++, true;
            do {
                Value x;
                if (!value(x))
                    return false;
                v.a.push_back(std::move(x));
                ws();
            } while (*p == ',' && p++);
            return *p++ == ']';
        }
        if (*p == '"') {
            v.type = Value::Str;
            return string(v.s);
        }
        for (auto [word, type, b] : {std::tuple{"true", Value::Bool, true}, {"false", Value::Bool, false}, {"null", Value::Null, false}})
            if (!strncmp(p, word, strlen(word))) {
                p += strlen(word);
                v.type = type;
                v.b = b;
                return true;
            }
        char *end;
        v.n = strtod(p, &end);
        if (end == p)
            return false;
        v.type = Value::Num;
        v.s.assign(p, size_t(end - p));
        p = end;
        return true;
    }
};

inline bool parse(const std::string &text, Value &out)
{
    Parser ps{text.c_str()};
    if (!ps.value(out))
        return false;
    ps.ws();
    return !*ps.p;
}

}
