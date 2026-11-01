#pragma once
#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <map>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

#include "apply/facts.hpp"

// Pure logic of the era picker addon row (era_picker.cpp): the box check, the box order, the image
// default, the Time Trial list and its car limit, the press guard, the links of the row's two node handles
// and the DRS group swap of the host setup mode. No game calls, so the unit test runs on the host.
//
// Time Trial: a session holds the cars of the entrant list it was picked with plus three cars of its own,
// and the session system that walks them (0x142f45960, called from Game_GameModule_OnSessionEnter) keeps
// them in a table of 26 at +0xb0 with its count right behind it at +0x180. A 22-car list gives 25; a
// 24-car list gives 27 and overwrites the count. A list object (Sess_Load_AvailableEntrantLists, copied
// by 0x1424df240) holds its entrants as 0x88-byte records from +0x10 to +0x18.
//
// The guard holds for any list, whichever box or addon made the setup select it. On a Time Trial picker,
// a press on a stock box leaves swofty_f1 offering only the lists of at most 23 cars, and when a Time Trial
// session loads (StateLoadSession) the core reads the setup's resolved list (setup +0xe18, the setup being
// *(*(mode context+0x88)+0x260), its entrant setup mode +0xe10, its game setup mode +0x2b0) and, when that
// list or the entrant setup mode's first list holds more than 23 cars, selects a list of at most 23 cars
// from the same entrant setup mode instead (the box pressed, then its Time Trial list, then the first that
// fits) and holds the entrant setup mode's max_entrants (+0x268, the selector's limit, research grid24-cap
// §2 step 6) at 23 until the session ends or a box is pressed. As the last net, a wrap of 0x142f45960
// (Sess_Setup_VehicleComponents) that sees more vehicles than the table's 26 hands the game a vector of the
// first 26 for that call and logs the list, so the table is never written past its end.

namespace runtime::era {

constexpr const char *HOST = "swofty_f1";
constexpr const char *SELECTED_ADDON_TILE = "selected_addon_tile";
constexpr const char *ADDON_TILES_VISIBLE = "addon_tiles_visible";
constexpr const char *IMAGE_ARRAY = "#overrides.home_screen.tiles_solo_6x6";
constexpr size_t MAX_BOXES = 64;
constexpr size_t LIST_NAME_MAX = 63;
constexpr size_t TEXT_MAX = 255;
constexpr size_t NODE_REFS = 0x24;
constexpr size_t NODE_HOLDERS = 0x28;
constexpr size_t HOLDER_PREV = 0x10;
constexpr size_t TT_TABLE = 26;
constexpr size_t TT_EXTRA_CARS = 3;
constexpr size_t TT_MAX_CARS = TT_TABLE - TT_EXTRA_CARS;
constexpr size_t LIST_ENTRANTS_BEGIN = 0x10;
constexpr size_t LIST_ENTRANTS_END = 0x18;
constexpr size_t ENTRANT_RECORD = 0x88;
constexpr size_t MAX_LIST_ENTRANTS = 64;

constexpr uint32_t djb2(const char *s)
{
    uint32_t h = 5381;
    while (*s)
        h = h * 33 + uint8_t(*s++);
    return h;
}

constexpr uint32_t ADDON_TILES = djb2("addon_tiles");
constexpr uint32_t ADDON_PRESS = djb2("addon_select_pressed");
constexpr uint32_t SELECT_PRESS = djb2("select_pressed");
static_assert(ADDON_TILES == 0xf2614e2b && ADDON_PRESS == 0x8fb68dff && SELECT_PRESS == 0x2b0357a);

struct Box {
    std::string entrant_list, title, image;
    int32_t order = 0;
    std::string owner;
    size_t seq = 0;
    std::string time_trial;
};

inline bool plain_name(const std::string &s)
{
    if (s.empty() || s.size() > LIST_NAME_MAX)
        return false;
    return std::all_of(s.begin(), s.end(), [](char c) {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_';
    });
}

inline std::string check(Box &box, const std::vector<Box> &have)
{
    if (!plain_name(box.entrant_list))
        return "entrant_list must be 1 to 63 letters, digits or underscores";
    if (box.time_trial == box.entrant_list)
        box.time_trial.clear();
    if (!box.time_trial.empty() && !plain_name(box.time_trial))
        return "time_trial_entrant_list must be empty or 1 to 63 letters, digits or underscores";
    if (box.title.empty())
        box.title = "lng_" + box.entrant_list;
    if (box.title.size() > TEXT_MAX || box.image.size() > TEXT_MAX)
        return "title and image must be at most 255 characters";
    if (box.title.find('\0') != std::string::npos || box.image.find('\0') != std::string::npos)
        return "title and image must not hold NUL bytes";
    for (const auto &h : have)
        if (h.entrant_list == box.entrant_list)
            return "a box for this entrant list is already registered by " + h.owner;
    if (have.size() >= MAX_BOXES)
        return "the core holds at most 64 boxes";
    return {};
}

inline std::string image_of(const Box &box)
{
    return box.image.empty() ? std::string(IMAGE_ARRAY) + "[" + box.entrant_list + "]" : box.image;
}

inline bool time_trial_mode(std::string_view label)
{
    return swofty_core::apply::facts::mode_of(label) == "time_trial";
}

inline const std::string &list_for(const Box &box, bool time_trial)
{
    return time_trial && !box.time_trial.empty() ? box.time_trial : box.entrant_list;
}

inline int64_t entrant_count(uintptr_t begin, uintptr_t end)
{
    if (!begin || end < begin || (end - begin) % ENTRANT_RECORD || (end - begin) / ENTRANT_RECORD > MAX_LIST_ENTRANTS)
        return -1;
    return int64_t((end - begin) / ENTRANT_RECORD);
}

inline std::string time_trial_refusal(const Box &box, int64_t cars)
{
    if (cars < 0 || size_t(cars) <= TT_MAX_CARS)
        return {};
    std::string list = list_for(box, true);
    std::string why = "its Time Trial list " + list + " holds " + std::to_string(cars) + " cars; Time Trial holds at most " +
                      std::to_string(TT_MAX_CARS) + " (the game adds " + std::to_string(TT_EXTRA_CARS) +
                      " cars of its own to a table of " + std::to_string(TT_TABLE) + ")";
    if (box.time_trial.empty())
        why += ". Give the box a Time Trial entrant list of at most " + std::to_string(TT_MAX_CARS) + " cars";
    return why;
}

// The Time Trial guard. A list whose car count was read and is past TT_MAX_CARS is oversized; a count that
// could not be read (-1) is kept, so the guard never drops what it cannot see.
struct ListCars {
    std::string name;
    int64_t cars = -1;
};

inline bool oversized(int64_t cars)
{
    return cars >= 0 && size_t(cars) > TT_MAX_CARS;
}

inline std::vector<size_t> time_trial_keep(const std::vector<ListCars> &lists)
{
    std::vector<size_t> keep;
    for (size_t i = 0; i < lists.size(); ++i)
        if (!oversized(lists[i].cars))
            keep.push_back(i);
    return keep;
}

inline int64_t time_trial_fallback(const std::vector<ListCars> &lists, const std::vector<std::string> &prefer)
{
    auto fits = [&](size_t i) { return lists[i].cars >= 0 && !oversized(lists[i].cars); };
    for (const auto &want : prefer)
        for (size_t i = 0; i < lists.size(); ++i)
            if (!want.empty() && lists[i].name == want && fits(i))
                return int64_t(i);
    for (size_t i = 0; i < lists.size(); ++i)
        if (fits(i))
            return int64_t(i);
    return -1;
}

inline int32_t time_trial_limit(int32_t max_entrants)
{
    return max_entrants < 1 || size_t(max_entrants) > TT_MAX_CARS ? int32_t(TT_MAX_CARS) : max_entrants;
}

inline std::string time_trial_swap(const std::string &list, int64_t cars, const std::string &setup_mode,
                                   const std::string &instead, int32_t limit_was)
{
    std::string why = "the Time Trial session's list " + list + " (setup mode " + setup_mode + ") holds " +
                      std::to_string(cars) + " cars; Time Trial holds at most " + std::to_string(TT_MAX_CARS) +
                      " (the game adds " + std::to_string(TT_EXTRA_CARS) + " cars of its own to a table of " +
                      std::to_string(TT_TABLE) + ")";
    if (!instead.empty())
        why += "; loading " + instead + " instead";
    else
        why += "; its entrant setup mode offers no list of at most " + std::to_string(TT_MAX_CARS) + " cars";
    if (limit_was)
        why += "; the session draws at most " + std::to_string(TT_MAX_CARS) + " cars (max_entrants " +
               std::to_string(limit_was) + " -> " + std::to_string(TT_MAX_CARS) + " until the session ends)";
    return why;
}

constexpr size_t VEHICLE_SLOT = 8;

inline size_t session_vehicles(uintptr_t begin, uintptr_t end)
{
    return end > begin ? size_t(((end - begin) / VEHICLE_SLOT) & 0xff) : 0;
}

inline uintptr_t vehicles_end_within_table(uintptr_t begin, uintptr_t end)
{
    return session_vehicles(begin, end) > TT_TABLE ? begin + TT_TABLE * VEHICLE_SLOT : end;
}

inline std::vector<Box> ordered(std::vector<Box> boxes)
{
    std::stable_sort(boxes.begin(), boxes.end(), [](const Box &a, const Box &b) {
        return a.order != b.order ? a.order < b.order : a.seq < b.seq;
    });
    return boxes;
}

struct Handle {
    uint8_t *node;
    void *next, *prev;
};
static_assert(offsetof(Handle, prev) == HOLDER_PREV);

inline void link(Handle &h, uint8_t *node)
{
    h = {node, *reinterpret_cast<void **>(node + NODE_HOLDERS), nullptr};
    if (h.next)
        *reinterpret_cast<void **>(static_cast<uint8_t *>(h.next) + HOLDER_PREV) = &h;
    *reinterpret_cast<void **>(node + NODE_HOLDERS) = &h;
    __atomic_fetch_add(reinterpret_cast<int16_t *>(node + NODE_REFS), int16_t(1), __ATOMIC_SEQ_CST);
}

// clear (FeTile_Clear_EraTileSet_Thunk in game) unlinks and releases both handles from the nodes they still
// hold, whichever view model those belong to, before the list handle is linked to this build's node.
template <class Set, class Clear> void relink(Set &set, uint8_t *list, Clear clear)
{
    if (set.list.node || set.selected.node)
        clear(set);
    set = {};
    link(set.list, list);
}

inline std::string press_refusal(bool same_view_model, bool nodes, size_t vector_tiles, size_t stock, size_t boxes)
{
    if (!same_view_model)
        return "the picker that raised it is not the one the addon row was built for";
    if (!nodes || !boxes)
        return "the addon row has no boxes";
    if (vector_tiles < stock + boxes)
        return "the tile vector holds " + std::to_string(vector_tiles) + " tiles, fewer than the " +
               std::to_string(stock) + " stock and " + std::to_string(boxes) + " addon tiles of the last build";
    return {};
}

// A box's DRS rule (content.package rules.drs) applies through the host setup mode's DRS group. A game setup mode
// object keeps each string field as a holder whose +0x10 is the char * label (re/db structs/GameSetupMode.h); its
// +0x1e0 holder names the drs.drs_allowed_lap group the DRS system looks up at session load, at a full safety car
// end and at a red flag restart. The swap points that field at a holder of the rule's group label and remembers the
// game's holder; put_back restores it wherever the field still holds the swap's holder.
constexpr size_t MODE_NAME = 0xd8;
constexpr size_t MODE_DRS_GROUP = 0x1e0;
constexpr size_t HOLDER_TEXT = 0x10;
constexpr size_t DRS_LABEL_MAX = 63;

struct DrsHolder {
    void *unused[2] = {};
    const char *text = nullptr;
    uint8_t pad[0x40] = {};
};
static_assert(offsetof(DrsHolder, text) == HOLDER_TEXT);

class DrsSwaps {
public:
    // A holder per label, at a stable address for the life of the process.
    DrsHolder *holder(const std::string &label)
    {
        if (label.empty() || label.size() > DRS_LABEL_MAX || label.find('\0') != std::string::npos)
            return nullptr;
        auto &h = holders_[label];
        if (!h) {
            h = std::make_unique<Entry>();
            h->label = label;
            h->holder.text = h->label.c_str();
        }
        return &h->holder;
    }

    void swap(uint8_t *mode, DrsHolder *holder)
    {
        if (!mode || !holder)
            return;
        void *&field = *reinterpret_cast<void **>(mode + MODE_DRS_GROUP);
        if (field == holder)
            return;
        bool known = false;
        for (auto &s : swaps_)
            known = known || s.mode == mode;
        if (!known)
            swaps_.push_back({mode, field});
        field = holder;
    }

    size_t put_back()
    {
        size_t n = 0;
        for (auto &s : swaps_) {
            void *&field = *reinterpret_cast<void **>(s.mode + MODE_DRS_GROUP);
            if (!owns(field))
                continue;
            field = s.original;
            n++;
        }
        swaps_.clear();
        return n;
    }

    // Drops the swaps without reading or writing the modes, which the game may have freed.
    size_t forget()
    {
        size_t n = swaps_.size();
        swaps_.clear();
        return n;
    }

    size_t active() const { return swaps_.size(); }

    bool owns(const void *holder) const
    {
        for (auto &[label, e] : holders_)
            if (&e->holder == holder)
                return true;
        return false;
    }

private:
    struct Entry {
        std::string label;
        DrsHolder holder;
    };
    struct Swap {
        uint8_t *mode;
        void *original;
    };
    std::map<std::string, std::unique_ptr<Entry>> holders_;
    std::vector<Swap> swaps_;
};

inline const char *holder_text(const void *holder)
{
    return holder ? *reinterpret_cast<const char *const *>(static_cast<const uint8_t *>(holder) + HOLDER_TEXT) : nullptr;
}

// Sess_Apply_ContentAvailability calls Sess_Load_GameSetupModes (through the thunk 0x142a223f0), which frees every
// GameSetupMode object before it parses the rows again. The pointers the picker keeps into the old objects (its
// narrowed entrant setups, its DRS group swaps, the Time Trial hold) are dropped at that function's entry and never
// written back; the next press or picker build applies the box to the new objects.
struct SetupPointers {
    size_t narrowed = 0, drs = 0;
    bool held = false;
};

inline std::string availability_drop_line(const SetupPointers &p)
{
    std::string what;
    auto add = [&](const std::string &part) { what += (what.empty() ? "" : ", ") + part; };
    if (p.narrowed)
        add(std::to_string(p.narrowed) + " narrowed entrant setup mode(s)");
    if (p.drs)
        add(std::to_string(p.drs) + " DRS group swap(s)");
    if (p.held)
        add("the Time Trial hold");
    if (what.empty())
        return {};
    return "the game rebuilds its setup modes (Sess_Apply_ContentAvailability): dropped " + what +
           " without writing them back; the next box press or picker build applies the box again";
}

}
