#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <atomic>
#include <cstdarg>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <swofty/game.hpp>
#include <swofty/mixin.hpp>
#include "../core/json.hpp"

extern "C" _invalid_parameter_handler __cdecl _set_thread_local_invalid_parameter_handler(_invalid_parameter_handler);

using namespace swofty::game;

namespace {

enum Level : uint8_t { L_NONE, L_INFO, L_WARN, L_ERROR, L_OFF };

constexpr size_t GATE_SLOTS = 373, GATE_MAX_OPEN = 300, LINE = 1024;
constexpr const char *DEFAULT_CHANNELS =
    "*WARN*,*ERR*,*FAIL*,*ASSERT*,*FATAL*,UI:warn,UI_*:warn,*_UI*:warn,*-UI*:warn,UIRef:warn,UICom:warn,"
    "UINodeTreeMessages:warn,VM:warn,FLOW:warn,FLOW/*:warn,UI_FLOW:warn,NeUI*:warn,EGO_ERP_LOADER:warn,"
    "NeAssetGroupLoader*:warn";
constexpr const char *VERBOSE_CHANNELS =
    "UI:info,UI_*:info,*_UI*:info,*-UI*:info,UIRef:info,UICom:info,UINodeTreeMessages:info,VM:info,FLOW:info,"
    "FLOW/*:info,UI_FLOW:info,NeUI*:info,EGO_ERP_LOADER:info,NeAssetGroupLoader*:info";

const char *const ROUTE_ERRORS[] = {
    "Screen %hs failed to ",
    "No screen called %hs",
    "The screen %hs ",
    "The direction %hs is not a valid",
    "Screen %hs is not active",
    "Failed to find screen",
    "Can't Find Screen",
    "The screen group",
    "Screen group is trying",
    "No valid screen group",
    "Can not navigate screen group",
};

const char *const ERROR_WORDS[] = {"error", "fail", "assert", "fatal", "exception", "crash"};
const char *const WARN_WORDS[] = {"warn",       "invalid",   "missing",     "unable",    "can't",         "cannot",
                                  "could not",  "couldn't",  "not found",   "not exist", "doesn't exist", "unknown",
                                  "unexpected", "mismatch",  "corrupt",     "timed out", "timeout",       "overflow",
                                  "no such",    "not supported", "unsupported", "ignor", "skipping",      "too many",
                                  "rejected",   "refused",   "denied"};

struct Rule {
    std::string glob;
    Level level;
};

struct Config {
    Level level = L_WARN;
    std::string channels = DEFAULT_CHANNELS;
    uint32_t rate = 100, repeat = 100, max_lines = 20000;
    std::string source = "defaults";
};

struct Slot {
    std::atomic<uintptr_t> key;
    std::atomic<uint8_t> value;
    std::atomic<uint32_t> count;
};

template <size_t N> struct Cache {
    Slot slots[N];

    Slot *find(uintptr_t key)
    {
        size_t h = key * 0x9E3779B97F4A7C15ull >> 40;
        for (size_t i = 0; i < 64; i++) {
            Slot &s = slots[(h + i) % N];
            uintptr_t k = s.key.load(std::memory_order_acquire);
            if (k == key)
                return &s;
            if (!k) {
                uintptr_t expected = 0;
                if (s.key.compare_exchange_strong(expected, key, std::memory_order_acq_rel) || expected == key)
                    return &s;
            }
        }
        return nullptr;
    }
};

Config cfg;
std::vector<Rule> rules;
bool active;
Cache<4096> log_formats, route_formats;
Cache<2048> channel_levels;
std::atomic<uint64_t> window;
std::atomic<uint32_t> dropped, lines;
std::atomic<uint64_t> perf_calls, perf_written, perf_repeat, perf_rate;
bool perf;
std::atomic<bool> capped;
std::vector<uint32_t> opened;
HMODULE self;

using PerfProbe = uint32_t (*)(const char *, const char *);
using PerfEnter = void (*)(uint32_t);
using PerfLeave = void (*)();
PerfEnter perf_enter;
PerfLeave perf_leave;
uint32_t log_probe;

struct Timed {
    bool on = perf_enter && log_probe;
    Timed()
    {
        if (on)
            perf_enter(log_probe);
    }
    ~Timed()
    {
        if (on)
            perf_leave();
    }
};

void find_profiler()
{
    HMODULE core = GetModuleHandleW(L"swofty_core.dll");
    auto probe = core ? reinterpret_cast<PerfProbe>(reinterpret_cast<void *>(GetProcAddress(core, "SwoftyPerfProbe"))) : nullptr;
    perf_enter = core ? reinterpret_cast<PerfEnter>(reinterpret_cast<void *>(GetProcAddress(core, "SwoftyPerfEnter"))) : nullptr;
    perf_leave = core ? reinterpret_cast<PerfLeave>(reinterpret_cast<void *>(GetProcAddress(core, "SwoftyPerfLeave"))) : nullptr;
    if (probe && perf_enter && perf_leave)
        log_probe = probe("core", "game log (Ne_Log_Thunk)");
}

bool icontains(const char *s, const char *w)
{
    size_t n = strlen(w);
    for (; *s; s++)
        if (!_strnicmp(s, w, n))
            return true;
    return false;
}

bool glob(const char *p, const char *s)
{
    if (!*p)
        return !*s;
    if (*p == '*')
        return glob(p + 1, s) || (*s && glob(p, s + 1));
    return *s && (*p == '?' || toupper((unsigned char)*p) == toupper((unsigned char)*s)) && glob(p + 1, s + 1);
}

Level parse_level(std::string s, Level def)
{
    for (auto &c : s)
        c = (char)tolower((unsigned char)c);
    if (s == "off" || s == "0" || s == "false" || s == "none")
        return L_OFF;
    if (s == "error" || s == "errors")
        return L_ERROR;
    if (s == "warn" || s == "warning" || s == "warnings")
        return L_WARN;
    if (s == "info" || s == "all" || s == "verbose" || s == "1" || s == "true")
        return L_INFO;
    return def;
}

const char *level_name(Level l)
{
    return l == L_INFO ? "info" : l == L_WARN ? "warn" : l == L_ERROR ? "error" : "off";
}

void parse_rules(const std::string &spec)
{
    rules.clear();
    size_t i = 0;
    while (i <= spec.size()) {
        size_t j = spec.find(',', i);
        if (j == std::string::npos)
            j = spec.size();
        std::string item = spec.substr(i, j - i);
        while (!item.empty() && item.front() == ' ')
            item.erase(item.begin());
        while (!item.empty() && item.back() == ' ')
            item.pop_back();
        if (item == "default" || item == "verbose") {
            std::vector<Rule> mine = rules;
            parse_rules(item == "default" ? DEFAULT_CHANNELS : VERBOSE_CHANNELS);
            mine.insert(mine.end(), rules.begin(), rules.end());
            rules = mine;
        } else if (item == "all") {
            rules.push_back({"*", L_NONE});
        } else if (!item.empty()) {
            size_t colon = item.rfind(':');
            Level l = L_NONE;
            if (colon != std::string::npos && colon > 0) {
                l = parse_level(item.substr(colon + 1), L_NONE);
                if (l != L_NONE)
                    item.resize(colon);
            }
            rules.push_back({item, l});
        }
        i = j + 1;
    }
}

Level channel_threshold(const char *channel)
{
    Level best = L_OFF;
    for (auto &r : rules)
        if (glob(r.glob.c_str(), channel)) {
            Level l = r.level == L_NONE ? cfg.level : r.level;
            if (l < best)
                best = l;
        }
    return best;
}

Level classify(const char *channel, const char *fmt)
{
    for (const char *w : {"ERR", "FAIL", "ASSERT", "FATAL"})
        if (channel && icontains(channel, w))
            return L_ERROR;
    for (const char *w : ERROR_WORDS)
        if (icontains(fmt, w))
            return L_ERROR;
    if (channel && icontains(channel, "WARN"))
        return L_WARN;
    for (const char *w : WARN_WORDS)
        if (icontains(fmt, w))
            return L_WARN;
    return L_INFO;
}

void quiet_handler(const wchar_t *, const wchar_t *, const wchar_t *, unsigned, uintptr_t) {}

int bad_read(DWORD code)
{
    return code == EXCEPTION_ACCESS_VIOLATION || code == EXCEPTION_IN_PAGE_ERROR || code == EXCEPTION_DATATYPE_MISALIGNMENT
               ? EXCEPTION_EXECUTE_HANDLER
               : EXCEPTION_CONTINUE_SEARCH;
}

int guarded_vsprintf(char *out, size_t n, const char *fmt, va_list ap)
{
    __try {
        return __stdio_common_vsprintf(_CRT_INTERNAL_PRINTF_STANDARD_SNPRINTF_BEHAVIOR, out, n, fmt, nullptr, ap);
    } __except (bad_read(GetExceptionCode())) {
        return -1;
    }
}

int format(char *out, size_t n, const char *fmt, va_list ap)
{
    _invalid_parameter_handler old = _set_thread_local_invalid_parameter_handler(quiet_handler);
    int r = guarded_vsprintf(out, n, fmt, ap);
    _set_thread_local_invalid_parameter_handler(old);
    if (r < 0)
        snprintf(out, n, "(unformattable) %s", fmt);
    for (char *p = out; *p; p++)
        if (*p == '\r' || *p == '\n' || *p == '\t')
            *p = ' ';
    return r;
}

bool admit(Slot *fmt_slot, const char *fmt)
{
    if (capped.load(std::memory_order_relaxed))
        return false;
    if (fmt_slot && cfg.repeat) {
        uint32_t c = fmt_slot->count.fetch_add(1, std::memory_order_relaxed) + 1;
        if (c > cfg.repeat) {
            perf_repeat.fetch_add(1, std::memory_order_relaxed);
            if (c == cfg.repeat + 1)
                swofty::log("[game] (format \"%.80s\" logged %u times; further repeats are suppressed)", fmt, cfg.repeat);
            return false;
        }
    }
    uint64_t now = GetTickCount64() / 1000;
    uint64_t w = window.load(std::memory_order_relaxed);
    for (;;) {
        uint64_t sec = w >> 32, count = w & 0xffffffff, next;
        if (sec != now)
            next = now << 32 | 1;
        else if (cfg.rate && count >= cfg.rate) {
            dropped.fetch_add(1, std::memory_order_relaxed);
            perf_rate.fetch_add(1, std::memory_order_relaxed);
            return false;
        } else
            next = w + 1;
        if (window.compare_exchange_weak(w, next, std::memory_order_relaxed)) {
            if (sec != now) {
                uint32_t d = dropped.exchange(0, std::memory_order_relaxed);
                if (d)
                    swofty::log("[game] (%u message(s) dropped by the rate limit of %u per second)", d, cfg.rate);
            }
            break;
        }
    }
    uint32_t n = lines.fetch_add(1, std::memory_order_relaxed) + 1;
    if (cfg.max_lines && n > cfg.max_lines) {
        if (!capped.exchange(true))
            swofty::log("[game] (line cap of %u reached; no further game messages are logged)", cfg.max_lines);
        return false;
    }
    return true;
}

std::wstring self_dir()
{
    wchar_t path[MAX_PATH] = {};
    GetModuleFileNameW(self, path, MAX_PATH);
    std::wstring w(path);
    w.resize(w.find_last_of(L'\\') + 1);
    return w;
}

std::string read_file(const std::wstring &path)
{
    std::string out;
    if (FILE *f = _wfopen(path.c_str(), L"rb")) {
        char buf[16384];
        for (size_t n; (n = fread(buf, 1, sizeof buf, f));)
            out.append(buf, n);
        fclose(f);
    }
    return out;
}

std::string env(const char *name)
{
    char buf[1024];
    DWORD n = GetEnvironmentVariableA(name, buf, sizeof buf);
    return n && n < sizeof buf ? std::string(buf, n) : std::string();
}

void load_config(const std::wstring &dir)
{
    json::Value session;
    if (json::parse(read_file(dir + L"..\\..\\session.json"), session) && session["game_log"].type == json::Value::Obj) {
        const json::Value &g = session["game_log"];
        cfg.source = "session.json";
        if (g["enabled"].type == json::Value::Bool && !g["enabled"].b)
            cfg.level = L_OFF;
        cfg.level = parse_level(g["level"].str(level_name(cfg.level)), cfg.level);
        if (g["channels"].type == json::Value::Str)
            cfg.channels = g["channels"].s;
        else if (g["channels"].type == json::Value::Arr) {
            cfg.channels.clear();
            for (auto &c : g["channels"].a)
                cfg.channels += (cfg.channels.empty() ? "" : ",") + c.str();
        }
        cfg.rate = (uint32_t)g["rate"].num(cfg.rate);
        cfg.repeat = (uint32_t)g["repeat"].num(cfg.repeat);
        cfg.max_lines = (uint32_t)g["max_lines"].num(cfg.max_lines);
    }
    std::string v;
    if (!(v = env("SWOFTY_GAME_LOG")).empty())
        cfg.level = parse_level(v, cfg.level), cfg.source = "environment";
    if (!(v = env("SWOFTY_GAME_LOG_CHANNELS")).empty())
        cfg.channels = v, cfg.source = "environment";
    if (!(v = env("SWOFTY_GAME_LOG_RATE")).empty())
        cfg.rate = (uint32_t)strtoul(v.c_str(), nullptr, 0);
    if (!(v = env("SWOFTY_GAME_LOG_REPEAT")).empty())
        cfg.repeat = (uint32_t)strtoul(v.c_str(), nullptr, 0);
    if (!(v = env("SWOFTY_GAME_LOG_MAX")).empty())
        cfg.max_lines = (uint32_t)strtoul(v.c_str(), nullptr, 0);
    perf = !(v = env("SWOFTY_PERF")).empty() && v != "0";
    parse_rules(cfg.channels);
}

uint32_t exe_stamp()
{
    auto base = reinterpret_cast<const uint8_t *>(swofty::api->module_base());
    auto nt = reinterpret_cast<const IMAGE_NT_HEADERS *>(base + reinterpret_cast<const IMAGE_DOS_HEADER *>(base)->e_lfanew);
    return nt->FileHeader.TimeDateStamp;
}

size_t gate_slot(uint32_t h, size_t i)
{
    return (h % GATE_SLOTS + (13 + h % 13) * i) % GATE_SLOTS;
}

bool has_hash(const NeLogGate *gate, uint32_t h)
{
    for (size_t i = 0; i < GATE_SLOTS; i++) {
        uint32_t cur = *reinterpret_cast<const volatile uint32_t *>(&gate->channel_hashes[gate_slot(h, i)]);
        if (cur == h)
            return true;
        if (!cur)
            return false;
    }
    return false;
}

bool insert_hash(NeLogGate *gate, uint32_t h)
{
    for (size_t i = 0; i < GATE_SLOTS; i++) {
        auto *slot = reinterpret_cast<volatile LONG *>(&gate->channel_hashes[gate_slot(h, i)]);
        LONG cur = *slot;
        if ((uint32_t)cur == h)
            return true;
        if (!cur && (InterlockedCompareExchange(slot, (LONG)h, 0) == 0 || (uint32_t)*slot == h))
            return true;
    }
    return false;
}

void open_gates(const std::wstring &dir)
{
    std::string text = read_file(dir + L"log_channels.tsv");
    if (text.empty()) {
        swofty::log("[gamelog] %lslog_channels.tsv missing; no game log channel is opened (route errors are still logged)",
                    dir.c_str());
        return;
    }
    unsigned long stamp = 0;
    char sha[32] = {};
    if (sscanf(text.c_str(), "# F1_25.exe %31s %lu", sha, &stamp) != 2 || stamp != exe_stamp()) {
        swofty::log("[gamelog] log_channels.tsv is for another build (PE timestamp %lu, this exe %lu); no game log channel "
                    "is opened (route errors are still logged)",
                    stamp, (unsigned long)exe_stamp());
        return;
    }
    std::vector<std::pair<uint32_t, std::string>> want;
    size_t pos = 0;
    while (pos < text.size()) {
        size_t end = text.find('\n', pos);
        if (end == std::string::npos)
            end = text.size();
        std::string line = text.substr(pos, end - pos);
        pos = end + 1;
        if (line.empty() || line[0] == '#')
            continue;
        size_t t1 = line.find('\t'), t2 = t1 == std::string::npos ? t1 : line.find('\t', t1 + 1);
        if (t2 == std::string::npos)
            continue;
        uint32_t h = (uint32_t)strtoul(line.c_str(), nullptr, 16);
        std::string names = line.substr(t2 + 1);
        while (!names.empty() && (names.back() == '\r' || names.back() == ' '))
            names.pop_back();
        bool hit = false;
        for (size_t a = 0; a <= names.size() && !hit;) {
            size_t b = names.find(',', a);
            if (b == std::string::npos)
                b = names.size();
            std::string n = names.substr(a, b - a);
            hit = n != "-" && channel_threshold(n.c_str()) != L_OFF;
            a = b + 1;
        }
        if (hit && h)
            want.push_back({h, names});
    }
    auto *gate = g_Ne_LogGate.get();
    std::string list;
    size_t skipped = 0;
    for (auto &[h, names] : want) {
        if (opened.size() >= GATE_MAX_OPEN || !insert_hash(gate, h)) {
            skipped++;
            continue;
        }
        opened.push_back(h);
        if (list.size() < 900)
            list += (list.empty() ? "" : ", ") + names;
    }
    if (!opened.empty())
        gate->enabled = 1;
    swofty::log("[gamelog] gate g_Ne_LogGate at %p: opened %zu channel hash(es)%s: %s%s", static_cast<void *>(gate),
                opened.size(), skipped ? " (some skipped: table full)" : "", list.c_str(), list.size() >= 900 ? ", ..." : "");
}

void perf_report()
{
    static uint64_t calls0, written0, repeat0, rate0;
    uint64_t calls = perf_calls.load(std::memory_order_relaxed), written = perf_written.load(std::memory_order_relaxed),
             repeat = perf_repeat.load(std::memory_order_relaxed), rate = perf_rate.load(std::memory_order_relaxed);
    if (calls != calls0)
        swofty::log("[perf] gamelog: %llu call(s) on open channels in the last 10 s (%.0f/s): %llu written, %llu over the "
                    "repeat cap, %llu over the rate limit",
                    (unsigned long long)(calls - calls0), double(calls - calls0) / 10.0,
                    (unsigned long long)(written - written0), (unsigned long long)(repeat - repeat0),
                    (unsigned long long)(rate - rate0));
    calls0 = calls, written0 = written, repeat0 = repeat, rate0 = rate;
}

DWORD WINAPI keep_gates(LPVOID)
{
    bool warned = false;
    for (unsigned tick = 1;; tick++) {
        Sleep(2000);
        if (perf && tick % 5 == 0)
            perf_report();
        auto *gate = g_Ne_LogGate.get();
        bool lost = gate->enabled != 1;
        for (size_t i = 0; i < opened.size() && !lost; i += 16)
            lost = !has_hash(gate, opened[i]);
        if (!lost)
            continue;
        for (uint32_t h : opened)
            insert_hash(gate, h);
        gate->enabled = 1;
        if (!warned)
            swofty::log("[gamelog] the game reset g_Ne_LogGate; channels re-opened (logged once)");
        warned = true;
    }
}

bool check()
{
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       reinterpret_cast<LPCWSTR>(&check), &self);
    load_config(self_dir());
    find_profiler();
    active = cfg.level != L_OFF;
    swofty::log("[gamelog] game log to loader.log: %s (level %s, from %s), channels '%s', %u line(s)/s, %u repeat(s) per "
                "format, cap %u line(s)",
                active ? "on" : "off", level_name(cfg.level), cfg.source.c_str(), cfg.channels.c_str(), cfg.rate,
                cfg.repeat, cfg.max_lines);
    return true;
}

}

SWOFTY_OVERWRITE(0, Ne_Log_Thunk, void, const char *channel, const char *fmt, ...)
{
    if (!active || !fmt)
        return;
    Timed timed;
    if (!channel)
        channel = "?";
    Slot *cs = channel_levels.find(reinterpret_cast<uintptr_t>(channel));
    uint8_t threshold = cs ? cs->value.load(std::memory_order_acquire) : 0;
    if (!threshold) {
        threshold = channel_threshold(channel);
        if (cs)
            cs->value.store(threshold, std::memory_order_release);
    }
    if (threshold == L_OFF)
        return;
    perf_calls.fetch_add(1, std::memory_order_relaxed);
    Slot *fs = log_formats.find(reinterpret_cast<uintptr_t>(fmt));
    uint8_t level = fs ? fs->value.load(std::memory_order_acquire) : 0;
    if (!level) {
        level = classify(channel, fmt);
        if (fs)
            fs->value.store(level, std::memory_order_release);
    }
    if (level < threshold || !admit(fs, fmt))
        return;
    char text[LINE];
    va_list ap;
    va_start(ap, fmt);
    format(text, sizeof text, fmt, ap);
    va_end(ap);
    perf_written.fetch_add(1, std::memory_order_relaxed);
    swofty::log("[game] %s%s: %s", channel, level == L_ERROR ? " (error)" : level == L_WARN ? " (warn)" : "",
                text);
}

SWOFTY_SITE(site_Ne_Format_String_Entry, swofty::Context &c)
{
    auto fmt = reinterpret_cast<const char *>(c.rdx);
    if (!active || !fmt)
        return;
    Slot *fs = route_formats.find(c.rdx);
    uint8_t cls = fs ? fs->value.load(std::memory_order_acquire) : 0;
    if (!cls) {
        cls = strstr(fmt, "Solution:") ? 2 : 1;
        for (const char *p : ROUTE_ERRORS)
            if (!strncmp(fmt, p, strlen(p)))
                cls = 2;
        if (fs)
            fs->value.store(cls, std::memory_order_release);
    }
    if (cls != 2 || cfg.level == L_OFF || !admit(fs, fmt))
        return;
    uintptr_t args[16] = {c.rcx, c.rdx, c.r8, c.r9};
    memcpy(args + 4, reinterpret_cast<const void *>(c.rsp + 0x28), sizeof args - 4 * sizeof *args);
    char text[LINE];
    format(text, sizeof text, fmt, reinterpret_cast<va_list>(&args[2]));
    swofty::log("[game] UI_ROUTE (error): %s", text);
}

SWOFTY_ON_LOAD
{
    if (!active)
        return;
    open_gates(self_dir());
    if (!opened.empty())
        if (HANDLE t = CreateThread(nullptr, 0, keep_gates, nullptr, 0, nullptr))
            CloseHandle(t);
}

SWOFTY_MOD(check)
