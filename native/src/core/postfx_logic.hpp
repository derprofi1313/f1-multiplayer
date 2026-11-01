#pragma once

#include <cstdint>
#include <cstdlib>
#include <string>
#include <string_view>

// The parts of the post-process passes that are plain logic: which views a pass runs in, the key
// names a toggle option holds, texture formats and sizes, and option values from settings text.
namespace postfx {

enum View : uint32_t { UNKNOWN = 0, COCKPIT, TCAM, TCAM_OFFSET, CHASE_NEAR, CHASE_FAR, OTHER, VIEWS };
enum Flag : uint32_t { PAUSED = 1, REPLAY = 2, PHOTO = 4, MENU = 8, CUTSCENE = 16, SPECTATING = 32 };

inline constexpr std::string_view VIEW_NAMES[VIEWS] = {"unknown",    "cockpit",   "tcam", "tcam_offset",
                                                      "chase_near", "chase_far", "other"};
inline constexpr std::string_view FLAG_NAMES[] = {"paused", "replay", "photo", "menu", "cutscene", "spectating"};

inline uint32_t view_bit(std::string_view name)
{
    for (uint32_t i = 1; i < VIEWS; i++)
        if (VIEW_NAMES[i] == name)
            return 1u << i;
    return 0;
}

inline uint32_t flag_bit(std::string_view name)
{
    for (uint32_t i = 0; i < std::size(FLAG_NAMES); i++)
        if (FLAG_NAMES[i] == name)
            return 1u << i;
    return 0;
}

inline std::string flag_text(uint32_t flags)
{
    std::string out;
    for (uint32_t i = 0; i < std::size(FLAG_NAMES); i++)
        if (flags & (1u << i))
            out += (out.empty() ? "" : "+") + std::string(FLAG_NAMES[i]);
    return out.empty() ? "-" : out;
}

inline std::string_view view_text(uint32_t kind)
{
    return kind < VIEWS ? VIEW_NAMES[kind] : "?";
}

struct When {
    uint32_t views = 0, skip = 0;
};

// A pass with views runs only while the view is known and one of them; skip flags stop it while the
// view is known to have them. Without either it always runs.
inline const char *ineligible(const When &w, uint32_t kind, uint32_t flags, bool valid)
{
    if (w.views && !valid)
        return "the view is unknown";
    if (w.views && !(w.views & (1u << kind)))
        return "not in one of its views";
    if (valid && (w.skip & flags))
        return "a skipped state";
    return nullptr;
}

// Win32 virtual-key codes for a toggle option's key names; 0 for None and unknown names.
inline int key_code(std::string_view key)
{
    if (key.size() == 1 && ((key[0] >= 'A' && key[0] <= 'Z') || (key[0] >= '0' && key[0] <= '9')))
        return key[0];
    if (key.size() == 1 && key[0] >= 'a' && key[0] <= 'z')
        return key[0] - 'a' + 'A';
    if (key.size() >= 2 && key.size() <= 3 && (key[0] == 'F' || key[0] == 'f')) {
        int n = 0;
        for (char c : key.substr(1)) {
            if (c < '0' || c > '9')
                return 0;
            n = n * 10 + (c - '0');
        }
        return n >= 1 && n <= 24 ? 0x70 + n - 1 : 0;
    }
    struct Named {
        std::string_view name;
        int vk;
    };
    static constexpr Named NAMED[] = {{"Insert", 0x2D}, {"Home", 0x24},      {"End", 0x23},
                                      {"Page Up", 0x21}, {"Page Down", 0x22}, {"Scroll Lock", 0x91},
                                      {"Pause", 0x13}};
    for (auto &n : NAMED)
        if (n.name == key)
            return n.vk;
    return 0;
}

// DXGI_FORMAT values for the texture format names.
inline int format_of(std::string_view name)
{
    struct Named {
        std::string_view name;
        int dxgi;
    };
    static constexpr Named NAMED[] = {{"rgba8", 28}, {"rgba16f", 10}, {"rg16f", 34},      {"r16f", 54},
                                      {"r32f", 41},  {"rgb10a2", 24}, {"r11g11b10f", 26}};
    for (auto &n : NAMED)
        if (n.name == name)
            return n.dxgi;
    return 0;
}

struct Size {
    uint32_t w = 0, h = 0;
};

// A texture's size for a picture: its own width and height, or the picture's scaled, at least 1x1.
inline Size texture_size(double scale, uint32_t width, uint32_t height, uint32_t pic_w, uint32_t pic_h)
{
    if (width && height)
        return {width, height};
    auto at = [&](uint32_t v) {
        double s = v * scale + 0.5;
        return s < 1 ? 1u : uint32_t(s);
    };
    return {at(pic_w), at(pic_h)};
}

// An option's value from the text of its current state in the settings: true and false read as 1 and
// 0, numbers as themselves; anything else is not a value.
inline bool option_value(std::string_view text, float &out)
{
    if (text == "true" || text == "false") {
        out = text == "true";
        return true;
    }
    std::string s(text);
    char *end = nullptr;
    double v = std::strtod(s.c_str(), &end);
    if (s.empty() || end != s.c_str() + s.size())
        return false;
    out = float(v);
    return true;
}

inline uint32_t groups(uint32_t size, uint32_t threads)
{
    return threads ? (size + threads - 1) / threads : 0;
}

}
