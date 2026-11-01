#include "runtime.hpp"

#include <swofty/game.hpp>
#include <swofty/mem.hpp>
#include <swofty/mixin.hpp>

#include <algorithm>
#include <atomic>
#include <cstddef>
#include <cstdio>
#include <cstring>
#include <iterator>
#include <mutex>
#include <string>
#include <vector>

#include "apply/content/appliers.hpp"
#include "apply/moments.hpp"
#include "era_logic.hpp"

std::string settings_addon(const void *from, std::string &dll);

// Era picker addon row (swofty/era.hpp). The Grand Prix, Time Trial and split-screen category picker
// (the DynamicEraSelectTabViewModel, VM dynamic_era_select) shows its stock boxes in the VM tiles list;
// the launcher's UI build gives the screen a second row (scene slot addon_categories_row) whose spawner
// lists the VM addon_tiles list, selects into selected_addon_tile and raises addon_select_pressed.
// Addons register boxes with add_box; each names an entrant list of the host setup mode swofty_f1, which
// the core's own database rows add to every F1 mode and addons list-append their entrant lists onto.
//
// Build (site_FeTile_EraTilesBuilt, rsi = the view model, after the last stock box): the stock tiles stay
// in vm+0x690. The core keeps a second tile set laid out like it (era::Set) whose two handles hold the
// addon_tiles list (Vm_GetOrCreate_ListChild under the VM root) and the selected_addon_tile reference
// (Vm_Set_NodeRef). Each handle sits in its node's holder list and holds a reference. The Grand Prix and
// Time Trial each build their own picker view model, and before every build FeTile_Clear_EraTileSet_Thunk
// clears the set wherever its handles still hold nodes, another view model's too: it empties that
// addon_tiles list and unlinks and releases both handles. A handle overwritten while still linked would
// leave the old node's holder list running on into the new node's, so the game's teardown of the old view
// model would release the new nodes as well, and tearing the new picker down then made a pure call on its
// freed selected_addon_tile. A view model the game tears down unlinks and releases the handles itself.
// FeTile_Add_EraTile adds each box while vm+0x698 points at addon_tiles, so its VM node lands in the
// addon row and its Tile in the stock vector after the stock tiles, which the game frees with them.
// Boxes whose entrant list the screen's mode does not offer under swofty_f1 are skipped with one log line.
// On a Time Trial picker (the mode's name, *(*(mode+0x88)+0x278), is one the core reports as time_trial)
// a box with its own Time Trial entrant list gets that list as its tile's command, so its press selects
// and narrows to it; a box without one keeps its own list. Either way the box is skipped there, with one
// log line, when that list is not offered under swofty_f1 or holds more cars than a Time Trial session
// holds (era_logic.hpp); the car count is the list object's entrant records.
// The VM bool addon_tiles_visible says whether the addon row holds a box; the UI only moves Down from the
// stock row to the addon row while it is true.
//
// Press (FeTile_Press_EraTile, wrapped): addon_select_pressed runs the stock select_pressed path with
// vm+0x698, vm+0x6b8 and the vector start pointed at the addon row, so the pressed box raises
// entrant_list_selected with its entrant list exactly like a stock box. The stock handler does not
// bounds-check the child index against the vector, so the wrap refuses a press unless the vector still
// holds the stock and addon tiles of the last build of that view model.
//
// Narrowing (Game_F1WorldActivityContainer_SelectEntrantList, wrapped): every addon list shares the one
// host setup mode, whose entrant setup modes list all of them. Time Trial loads every car of every list
// the entrant setup mode offers and the Grand Prix takes its first list, so a press on an addon box
// leaves only that box's list in each swofty_f1 entrant setup mode until another box is pressed, and
// every press clears the setup's selected list, which the setup refill takes from the entrant setup
// mode's first list only while it is empty. A build of the boxes puts every list back for the
// availability checks, then narrows again to the addon list the setup has selected (setup +0xe18), or
// to the last addon box pressed while nothing is selected: a session can start from a rebuilt picker
// without a new press, and a Time Trial that loads all nine Classic Seasons lists overflows the fixed
// 43 MiB AUDIO_BUNDLE heap with their engine sound bundles.
//
// DRS per box: a narrowing also points every swofty_f1 game setup mode's DRS group (+0x1e0, a label holder) at the
// group the core added at boot for the box's rules.drs (content.package rules.drs). A box without a rule, a stock
// box press and a build that narrows to no addon list put the game's group back. The DRS system reads the current
// mode's group at session load, at a full safety car end and at a red flag restart; every other mode keeps its own.
//
// The three hooks are one core mixin set, offered at the first mixin commit after a box was registered,
// so a game without registered boxes runs none of them.
//
// Time Trial guard (era_logic.hpp): a stock box pressed on a Time Trial picker leaves swofty_f1 offering
// only its lists of at most 23 cars; era_session_load (StateLoadSession, any mode) puts back what it held
// and, in Time Trial, logs the setup's resolved list and keeps a list past 23 cars out of the session;
// era_session_end (StateUnloadSession) and every press put it back. The session vehicle guard is a
// separate set wrapping Sess_Setup_VehicleComponents, offered whether or not a box is registered.
//
// Setup mode guard: Sess_Apply_ContentAvailability rebuilds the game setup modes (it frees the old objects), so a
// set wrapping its entry drops the narrowing, DRS swaps and Time Trial hold the core keeps without writing them back.

namespace {

namespace mem = swofty::mem;
namespace logic = runtime::era;
using swofty::game::FeTile_Add_EraTile_t;
using swofty::game::FeTile_Clear_EraTileSet_Thunk_t;
using swofty::game::FeTile_Is_EraScenarioAvailable_Thunk_t;
using swofty::game::FeTile_Press_EraTile_t;
using swofty::game::Game_F1WorldActivityContainer_SelectEntrantList_t;
using swofty::game::Vm_GetOrCreate_ListChild_t;
using swofty::game::Vm_Set_Bool_t;
using swofty::game::Vm_Set_NodeRef_t;

constexpr const char *MIXIN = "swofty_core.dll (era picker)";
constexpr uint32_t LOG_FIRST = 6;

constexpr size_t MODE_NAME = 0x278;
constexpr size_t VM_ROOT = 0x10;
constexpr size_t VM_ALLOCATOR = 0x270;
constexpr size_t VM_MODE = 0x670;
constexpr size_t VM_TILES = 0x690;
constexpr size_t VM_TILES_NODE = 0x698;
constexpr size_t VM_SELECTED_NODE = 0x6b8;
constexpr size_t VM_TILE_BEGIN = 0x6d8;
constexpr size_t VM_TILE_END = 0x6e0;
constexpr size_t TILE_ENTRY = 0x10;

constexpr size_t CONTAINER_MODE = 0x2f0;
constexpr size_t MODE_SETUP = 0x88;
constexpr size_t SETUP_MODES = 0x260;
constexpr size_t SETUP_MODES_VFUNC = 0x1d0;
constexpr size_t SELECTED_LIST = 0xe18;
constexpr size_t SETUP_ENTRANT_SETUP = 0xe10;
constexpr size_t SETUP_GAME_SETUP_MODE = 0x2b0;
constexpr size_t ENTRANT_SETUP_MAX_ENTRANTS = 0x268;
constexpr size_t SETUP_MODE_NAME = 0xd8;
constexpr size_t ENTRANT_SETUPS = 0x10;
constexpr size_t ENTRANT_SETUP_COUNT = 0x70;
constexpr size_t LISTS = 0x130;
constexpr size_t LIST_COUNT = 0x230;
constexpr size_t LIST_SLOTS = 32;
constexpr size_t LIST_NAME = 0x28;
constexpr int64_t ENTRANT_SETUP_SLOTS = 12;

using logic::Handle;

struct Set {
    void *vt0;
    Handle list;
    void *vt1;
    Handle selected;
    void *allocator;
    uint8_t *begin, *end, *cap;
};
static_assert(sizeof(Set) == 0x60);
static_assert(offsetof(Set, list) == VM_TILES_NODE - VM_TILES && offsetof(Set, selected) == VM_SELECTED_NODE - VM_TILES &&
              offsetof(Set, begin) == VM_TILE_BEGIN - VM_TILES);

struct Bound {
    uintptr_t site, press, select, add, available, list_child, node_ref, set_bool, clear, registry;
} b;

struct Built {
    uint8_t *vm = nullptr;
    size_t stock = 0, boxes = 0;
    std::vector<std::string> lists;
};

struct Narrowed {
    uint8_t *setup;
    int64_t count, kept;
    void *lists[LIST_SLOTS];
    void *kept_lists[LIST_SLOTS];
};

std::mutex lock;
std::vector<logic::Box> boxes;
std::vector<std::string> skipped;
std::atomic<bool> offered, settled, live, off;
std::string off_why;
Set set;
Built built;
Narrowed narrowed[4];
size_t narrowed_count;
std::string last_pick, last_press;
uint32_t logged_renarrow;
FeTile_Press_EraTile_t *press_orig;
Game_F1WorldActivityContainer_SelectEntrantList_t *select_orig;
uint32_t logged_builds, logged_narrow, logged_refused, logged_detached;
constexpr size_t BOX_TIME_TRIAL = offsetof(SwoftyEraBox, time_trial_entrant_list);
static_assert(BOX_TIME_TRIAL == 0x28);
logic::DrsSwaps drs;
std::string last_drs_line;

template <class T> T at(const void *base, size_t off)
{
    return *reinterpret_cast<const T *>(static_cast<const uint8_t *>(base) + off);
}

template <class T> T &ref(void *base, size_t off)
{
    return *reinterpret_cast<T *>(static_cast<uint8_t *>(base) + off);
}

const char *name_of(const void *object, size_t off)
{
    auto *holder = object ? at<const uint8_t *>(object, off) : nullptr;
    return holder ? at<const char *>(holder, 0x10) : nullptr;
}

uint8_t *setup_owner(void *mode)
{
    auto *owner = mode ? at<uint8_t *>(mode, MODE_SETUP) : nullptr;
    return owner ? at<uint8_t *>(owner, SETUP_MODES) : nullptr;
}

template <class F> void for_each_host_setup(void *mode, F f)
{
    auto *modes = setup_owner(mode);
    if (!modes)
        return;
    auto get = reinterpret_cast<uint8_t *(*)(void *)>((*reinterpret_cast<void ***>(modes))[SETUP_MODES_VFUNC / 8]);
    uint8_t *vec = get(modes);
    if (!vec)
        return;
    for (auto **p = at<uint8_t **>(vec, 8), **e = at<uint8_t **>(vec, 0x10); p != e; ++p) {
        const char *name = name_of(*p, SETUP_MODE_NAME);
        if (!name || strcmp(name, logic::HOST))
            continue;
        int64_t n = at<int64_t>(*p, ENTRANT_SETUP_COUNT);
        auto **setups = reinterpret_cast<uint8_t **>(*p + ENTRANT_SETUPS);
        for (int64_t i = 0; i < n && i < ENTRANT_SETUP_SLOTS; ++i)
            f(setups[i]);
    }
}

template <class F> void for_each_host_mode(void *mode, F f)
{
    auto *modes = setup_owner(mode);
    if (!modes)
        return;
    auto get = reinterpret_cast<uint8_t *(*)(void *)>((*reinterpret_cast<void ***>(modes))[SETUP_MODES_VFUNC / 8]);
    uint8_t *vec = get(modes);
    if (!vec)
        return;
    for (auto **p = at<uint8_t **>(vec, 8), **e = at<uint8_t **>(vec, 0x10); p != e; ++p) {
        const char *name = name_of(*p, SETUP_MODE_NAME);
        if (name && !strcmp(name, logic::HOST))
            f(*p);
    }
}

void log_drs(const std::string &line)
{
    if (line == last_drs_line)
        return;
    last_drs_line = line;
    swofty::log("[drs] %s", line.c_str());
}

void apply_drs(void *mode, const char *list)
{
    auto box = swofty_core::apply::content::drs_for_box(list);
    if (box.label.empty()) {
        drs.put_back();
        std::string own;
        for_each_host_mode(mode, [&](uint8_t *m) {
            auto *holder = at<uint8_t *>(m, logic::MODE_DRS_GROUP);
            const char *text = holder && mem::readable(holder + logic::HOLDER_TEXT, 8) ? logic::holder_text(holder) : nullptr;
            if (own.empty() && text && mem::readable(text, 1))
                own = std::string(text, strnlen(text, logic::DRS_LABEL_MAX));
        });
        log_drs(box.line + (own.empty() ? std::string() : " (" + own + ")"));
        return;
    }
    logic::DrsHolder *holder = drs.holder(box.label);
    size_t modes = 0;
    for_each_host_mode(mode, [&](uint8_t *m) {
        drs.swap(m, holder);
        modes++;
    });
    log_drs(holder && modes ? box.line : box.line + "; not applied (no " + std::string(logic::HOST) + " mode found)");
}

void put_back_drs()
{
    if (size_t n = drs.put_back())
        log_drs(std::string(logic::HOST) + "'s DRS group put back on " + std::to_string(n) + " mode(s)");
}

void restore_lists()
{
    for (size_t i = 0; i < narrowed_count; ++i) {
        Narrowed &n = narrowed[i];
        if (at<int64_t>(n.setup, LIST_COUNT) != n.kept ||
            memcmp(n.setup + LISTS, n.kept_lists, size_t(n.kept) * sizeof(void *)))
            continue;
        memcpy(n.setup + LISTS, n.lists, size_t(n.count) * sizeof(void *));
        ref<int64_t>(n.setup, LIST_COUNT) = n.count;
    }
    narrowed_count = 0;
}

void restore()
{
    restore_lists();
    put_back_drs();
}

void narrow(void *mode, const char *list)
{
    restore_lists();
    for_each_host_setup(mode, [&](uint8_t *setup) {
        int64_t count = at<int64_t>(setup, LIST_COUNT);
        if (count <= 1 || count > int64_t(LIST_SLOTS) || narrowed_count == std::size(narrowed))
            return;
        auto **lists = reinterpret_cast<void **>(setup + LISTS);
        void *hit = nullptr;
        for (int64_t i = 0; i < count && !hit; ++i) {
            const char *name = name_of(lists[i], LIST_NAME);
            if (name && !strcmp(name, list))
                hit = lists[i];
        }
        if (!hit)
            return;
        Narrowed &n = narrowed[narrowed_count++];
        n.setup = setup;
        n.count = count;
        n.kept = 1;
        n.kept_lists[0] = hit;
        memcpy(n.lists, lists, size_t(count) * sizeof(void *));
        lists[0] = hit;
        ref<int64_t>(setup, LIST_COUNT) = 1;
        if (logged_narrow++ < LOG_FIRST)
            swofty::log("[era] %s offers only %s (of %lld lists)", logic::HOST, list, (long long)count);
    });
    apply_drs(mode, list);
}

bool time_trial(void *mode)
{
    uintptr_t holder = 0, name = 0;
    std::string label;
    if (!mode || !mem::read(reinterpret_cast<uintptr_t>(mode) + MODE_SETUP, holder) || !holder ||
        !mem::read(holder + MODE_NAME, name) || !mem::c_string(name, label, 64))
        return false;
    return logic::time_trial_mode(label);
}

int64_t cars_of(void *mode, const std::string &list)
{
    int64_t cars = -1;
    for_each_host_setup(mode, [&](uint8_t *setup) {
        int64_t count = at<int64_t>(setup, LIST_COUNT);
        auto **lists = reinterpret_cast<uint8_t **>(setup + LISTS);
        for (int64_t i = 0; i < count && i < int64_t(LIST_SLOTS) && cars < 0; ++i) {
            const char *name = name_of(lists[i], LIST_NAME);
            uintptr_t begin = 0, end = 0;
            if (!name || strcmp(name, list.c_str()) ||
                !mem::read(reinterpret_cast<uintptr_t>(lists[i]) + logic::LIST_ENTRANTS_BEGIN, begin) ||
                !mem::read(reinterpret_cast<uintptr_t>(lists[i]) + logic::LIST_ENTRANTS_END, end))
                continue;
            cars = logic::entrant_count(begin, end);
        }
    });
    return cars;
}

logic::ListCars list_cars(const void *list)
{
    logic::ListCars out;
    uintptr_t holder = 0, name = 0, begin = 0, end = 0;
    auto at_list = reinterpret_cast<uintptr_t>(list);
    if (!list || !mem::read(at_list + LIST_NAME, holder) || !holder || !mem::read(holder + 0x10, name) ||
        !mem::c_string(name, out.name, 64))
        out.name.clear();
    if (list && mem::read(at_list + logic::LIST_ENTRANTS_BEGIN, begin) && mem::read(at_list + logic::LIST_ENTRANTS_END, end))
        out.cars = logic::entrant_count(begin, end);
    return out;
}

std::vector<logic::ListCars> lists_of(const uint8_t *entrant_setup, std::vector<void *> &ptrs)
{
    std::vector<logic::ListCars> out;
    int64_t count = 0;
    if (!entrant_setup || !mem::read(reinterpret_cast<uintptr_t>(entrant_setup) + LIST_COUNT, count) || count <= 0 ||
        count > int64_t(LIST_SLOTS))
        return out;
    for (int64_t i = 0; i < count; ++i) {
        uintptr_t list = 0;
        if (!mem::read(reinterpret_cast<uintptr_t>(entrant_setup) + LISTS + size_t(i) * 8, list))
            list = 0;
        ptrs.push_back(reinterpret_cast<void *>(list));
        out.push_back(list_cars(reinterpret_cast<void *>(list)));
    }
    return out;
}

struct Held {
    uint8_t *esm;
    int32_t limit_was;
    size_t slot;
    void *first;
    void *fit;
};
std::mutex held_lock;
Held held;
bool holding;

void put_back()
{
    if (!holding)
        return;
    holding = false;
    if (held.limit_was && at<int32_t>(held.esm, ENTRANT_SETUP_MAX_ENTRANTS) == logic::time_trial_limit(held.limit_was))
        ref<int32_t>(held.esm, ENTRANT_SETUP_MAX_ENTRANTS) = held.limit_was;
    if (held.first && held.slot < LIST_SLOTS && at<void *>(held.esm, LISTS) == held.fit &&
        at<void *>(held.esm, LISTS + held.slot * 8) == held.first) {
        ref<void *>(held.esm, LISTS + held.slot * 8) = at<void *>(held.esm, LISTS);
        ref<void *>(held.esm, LISTS) = held.first;
    }
}

constexpr const char *VEHICLE_MIXIN = "swofty_core.dll (session vehicle guard)";
constexpr const char *VEHICLE_FN = "Sess_Setup_VehicleComponents";
constexpr size_t MODULE_VEHICLES_BEGIN = 0x28;
constexpr size_t MODULE_VEHICLES_END = 0x30;
constexpr size_t MODULE_MODE_CONTEXT = 0x80;
using SetupVehicles_t = void(void *module);
SetupVehicles_t *vehicles_orig;
std::atomic<bool> vehicles_offered, vehicles_settled, vehicles_bound;
uint32_t logged_vehicles;

std::string session_list(uintptr_t ctx)
{
    uintptr_t holder = 0, setup = 0, list = 0, gsm = 0, gsm_holder = 0, gsm_name = 0;
    std::string mode = "?";
    if (!ctx || !mem::read(ctx + MODE_SETUP, holder) || !holder || !mem::read(holder + SETUP_MODES, setup) || !setup)
        return "an unread list";
    if (mem::read(setup + SETUP_GAME_SETUP_MODE, gsm) && gsm && mem::read(gsm + SETUP_MODE_NAME, gsm_holder) && gsm_holder &&
        mem::read(gsm_holder + 0x10, gsm_name))
        mem::c_string(gsm_name, mode, 64);
    logic::ListCars cars = mem::read(setup + SELECTED_LIST, list) && list ? list_cars(reinterpret_cast<void *>(list))
                                                                         : logic::ListCars{};
    return "list " + (cars.name.empty() ? std::string("?") : cars.name) + " (" + std::to_string(cars.cars) +
           " cars) of setup mode " + mode;
}

void vehicles_entry(void *module)
{
    auto *m = static_cast<uint8_t *>(module);
    uintptr_t begin = at<uintptr_t>(m, MODULE_VEHICLES_BEGIN), end = at<uintptr_t>(m, MODULE_VEHICLES_END);
    uintptr_t within = logic::vehicles_end_within_table(begin, end);
    if (within == end)
        return vehicles_orig(module);
    if (logged_vehicles++ < LOG_FIRST)
        swofty::log("[era] the session holds %zu vehicles and its vehicle table %zu; setting up the first %zu (the "
                    "session's %s)",
                    logic::session_vehicles(begin, end), logic::TT_TABLE, logic::TT_TABLE,
                    session_list(at<uintptr_t>(m, MODULE_MODE_CONTEXT)).c_str());
    ref<uintptr_t>(m, MODULE_VEHICLES_END) = within;
    vehicles_orig(module);
    if (at<uintptr_t>(m, MODULE_VEHICLES_END) == within)
        ref<uintptr_t>(m, MODULE_VEHICLES_END) = end;
}

std::vector<std::string> kept_out;
uint32_t logged_kept;

bool keep_time_trial_lists(void *mode)
{
    bool any = false;
    for_each_host_setup(mode, [&](uint8_t *setup) {
        std::vector<void *> ptrs;
        auto info = lists_of(setup, ptrs);
        auto keep = logic::time_trial_keep(info);
        if (info.empty() || keep.size() == info.size() || keep.empty() || narrowed_count == std::size(narrowed))
            return;
        Narrowed &n = narrowed[narrowed_count++];
        n.setup = setup;
        n.count = int64_t(info.size());
        n.kept = int64_t(keep.size());
        memcpy(n.lists, ptrs.data(), info.size() * sizeof(void *));
        for (size_t i = 0; i < keep.size(); ++i)
            n.kept_lists[i] = ptrs[keep[i]];
        memcpy(setup + LISTS, n.kept_lists, keep.size() * sizeof(void *));
        ref<int64_t>(setup, LIST_COUNT) = n.kept;
        any = true;
        for (size_t i = 0; i < info.size(); ++i) {
            if (!logic::oversized(info[i].cars) ||
                std::find(kept_out.begin(), kept_out.end(), info[i].name) != kept_out.end())
                continue;
            kept_out.push_back(info[i].name);
            if (logged_kept++ < LOG_FIRST * 4)
                swofty::log("[era] %s keeps %s out of Time Trial while a stock box is picked: it holds %lld cars; Time "
                            "Trial holds at most %zu (the game adds %zu cars of its own to a table of %zu)",
                            logic::HOST, info[i].name.c_str(), (long long)info[i].cars, logic::TT_MAX_CARS,
                            logic::TT_EXTRA_CARS, logic::TT_TABLE);
        }
    });
    return any;
}

bool skip_once(const std::string &key)
{
    std::lock_guard<std::mutex> g(lock);
    if (std::find(skipped.begin(), skipped.end(), key) != skipped.end())
        return false;
    skipped.push_back(key);
    return true;
}

bool is_ours(const char *list)
{
    return list && std::find(built.lists.begin(), built.lists.end(), list) != built.lists.end();
}

bool renarrow(void *mode)
{
    auto *modes = setup_owner(mode);
    if (!modes)
        return false;
    const char *selected = name_of(at<void *>(modes, SELECTED_LIST), LIST_NAME);
    std::string list = is_ours(selected) ? selected : selected ? std::string() : last_pick;
    if (list.empty() || !is_ours(list.c_str()))
        return false;
    narrow(mode, list.c_str());
    if (logged_renarrow++ < LOG_FIRST)
        swofty::log("[era] picker built with %s still selected (%s): %s narrowed to it again", list.c_str(),
                    selected ? "the setup's selected list" : "the last addon box pressed", logic::HOST);
    return true;
}

void *registry()
{
    return *reinterpret_cast<void **>(b.registry);
}

uint8_t *addon_list(uint8_t *vm)
{
    auto *root = at<uint8_t *>(vm, VM_ROOT);
    if (!root)
        return nullptr;
    uint64_t key = logic::ADDON_TILES;
    auto **h = reinterpret_cast<Vm_GetOrCreate_ListChild_t *>(b.list_child)(registry(), root + 8, reinterpret_cast<const uint32_t *>(&key));
    return h ? static_cast<uint8_t *>(*h) : nullptr;
}

uint8_t *addon_selection(uint8_t *vm)
{
    auto *root = at<uint8_t *>(vm, VM_ROOT);
    if (!root)
        return nullptr;
    Handle empty{};
    auto **h = reinterpret_cast<Vm_Set_NodeRef_t *>(b.node_ref)(registry(), root + 8, logic::SELECTED_ADDON_TILE, &empty, 1);
    return h ? static_cast<uint8_t *>(*h) : nullptr;
}

bool reset_set(uint8_t *vm, uint8_t *list)
{
    if ((set.list.node || set.selected.node) && (set.list.node != list || built.vm != vm) && logged_detached++ < LOG_FIRST)
        swofty::log("[era] detached the addon row from a previous view model (its boxes there cleared, its two nodes "
                    "unlinked and released)");
    logic::relink(set, list, [](Set &s) { reinterpret_cast<FeTile_Clear_EraTileSet_Thunk_t *>(b.clear)(&s); });
    uint8_t *selected = addon_selection(vm);
    if (!selected)
        return false;
    logic::link(set.selected, selected);
    return true;
}

void site_entry(SwoftyContext *c)
{
    auto *vm = reinterpret_cast<uint8_t *>(c->rsi);
    if (!vm || !live || off)
        return;
    restore_lists();
    void *mode = at<void *>(vm, VM_MODE);
    void *alloc = at<void *>(vm, VM_ALLOCATOR);
    auto *begin = at<uint8_t *>(vm, VM_TILE_BEGIN);
    auto *end = at<uint8_t *>(vm, VM_TILE_END);
    size_t stock = end > begin ? size_t(end - begin) / TILE_ENTRY : 0;
    uint8_t *list = addon_list(vm);
    if (!list || !reset_set(vm, list)) {
        put_back_drs();
        built = {};
        swofty::log("[era] the view model's addon_tiles or selected_addon_tile node could not be made; no addon row");
        return;
    }
    std::vector<logic::Box> order;
    {
        std::lock_guard<std::mutex> g(lock);
        order = logic::ordered(boxes);
    }
    auto available = reinterpret_cast<FeTile_Is_EraScenarioAvailable_Thunk_t *>(b.available);
    auto add = reinterpret_cast<FeTile_Add_EraTile_t *>(b.add);
    uint8_t *tiles_node = at<uint8_t *>(vm, VM_TILES_NODE);
    bool tt = time_trial(mode);
    built = {vm, stock, 0, {}};
    ref<uint8_t *>(vm, VM_TILES_NODE) = list;
    for (const auto &box : order) {
        if (built.boxes >= logic::MAX_BOXES)
            break;
        if (!mode || !available(mode, logic::HOST, box.entrant_list.c_str())) {
            if (skip_once(box.entrant_list))
                swofty::log("[era] box %s from %s skipped: this screen's mode does not offer the entrant list under %s "
                            "(it must exist and be in %s's entrant lists)",
                            box.entrant_list.c_str(), box.owner.c_str(), logic::HOST, logic::HOST);
            continue;
        }
        const std::string &list = logic::list_for(box, tt);
        if (tt && list != box.entrant_list && !available(mode, logic::HOST, list.c_str())) {
            if (skip_once("time_trial " + box.entrant_list))
                swofty::log("[era] box %s from %s skipped in Time Trial: its Time Trial entrant list %s is not offered "
                            "under %s (it must exist and be in %s's entrant lists)",
                            box.entrant_list.c_str(), box.owner.c_str(), list.c_str(), logic::HOST, logic::HOST);
            continue;
        }
        if (tt) {
            int64_t cars = cars_of(mode, list);
            std::string why = logic::time_trial_refusal(box, cars);
            if (!why.empty()) {
                if (skip_once("time_trial " + box.entrant_list))
                    swofty::log("[era] box %s from %s skipped in Time Trial: %s", box.entrant_list.c_str(),
                                box.owner.c_str(), why.c_str());
                continue;
            }
            if (cars < 0 && skip_once("time_trial count " + list))
                swofty::log("[era] box %s from %s: the car count of its Time Trial list %s could not be read; not checked",
                            box.entrant_list.c_str(), box.owner.c_str(), list.c_str());
        }
        char node[0x20];
        snprintf(node, sizeof node, "select_tile_%zu", built.boxes);
        std::string image = logic::image_of(box);
        add(vm + VM_TILES, alloc, uint8_t(built.boxes), node, box.entrant_list.c_str(), 1, box.title.c_str(),
            list.c_str(), image.c_str(), 0);
        built.lists.push_back(list);
        built.boxes++;
    }
    ref<uint8_t *>(vm, VM_TILES_NODE) = tiles_node;
    if (!renarrow(mode)) {
        put_back_drs();
        if (tt)
            keep_time_trial_lists(mode);
    }
    if (auto *root = at<uint8_t *>(vm, VM_ROOT)) {
        uint8_t visible = built.boxes > 0;
        reinterpret_cast<Vm_Set_Bool_t *>(b.set_bool)(registry(), root + 8, logic::ADDON_TILES_VISIBLE, &visible, 1);
    }
    if (logged_builds++ < LOG_FIRST)
        swofty::log("[era] era select%s: %zu stock box(es), %zu addon box(es) of %zu registered",
                    tt ? " (Time Trial)" : "", stock, built.boxes, order.size());
}

char *press_entry(void *vm, void *command)
{
    auto *cmd = command ? *static_cast<const uint32_t **>(command) : nullptr;
    if (!cmd || !mem::readable(cmd, 4) || *cmd != logic::ADDON_PRESS)
        return press_orig(vm, command);
    auto *v = static_cast<uint8_t *>(vm);
    auto *begin = at<uint8_t *>(v, VM_TILE_BEGIN);
    auto *end = at<uint8_t *>(v, VM_TILE_END);
    size_t count = end > begin ? size_t(end - begin) / TILE_ENTRY : 0;
    std::string why = logic::press_refusal(v == built.vm, set.list.node && set.selected.node, count, built.stock,
                                           built.boxes);
    if (!why.empty()) {
        if (logged_refused++ < LOG_FIRST)
            swofty::log("[era] addon box press refused: %s", why.c_str());
        return nullptr;
    }
    uint8_t *tiles_node = at<uint8_t *>(v, VM_TILES_NODE), *selected_node = at<uint8_t *>(v, VM_SELECTED_NODE);
    ref<uint8_t *>(v, VM_TILES_NODE) = set.list.node;
    ref<uint8_t *>(v, VM_SELECTED_NODE) = set.selected.node;
    ref<uint8_t *>(v, VM_TILE_BEGIN) = begin + built.stock * TILE_ENTRY;
    uint32_t select = logic::SELECT_PRESS;
    const void *fake[4] = {&select, nullptr, nullptr, nullptr};
    char *r = press_orig(vm, fake);
    ref<uint8_t *>(v, VM_TILES_NODE) = tiles_node;
    ref<uint8_t *>(v, VM_SELECTED_NODE) = selected_node;
    ref<uint8_t *>(v, VM_TILE_BEGIN) = begin;
    return r;
}

void select_entry(void *container, const char *entrant_list)
{
    {
        std::lock_guard<std::mutex> g(held_lock);
        put_back();
    }
    void *mode = container ? at<void *>(container, CONTAINER_MODE) : nullptr;
    bool ours = entrant_list && std::find(built.lists.begin(), built.lists.end(), entrant_list) != built.lists.end();
    if (mode && ours)
        narrow(mode, entrant_list);
    else
        restore();
    if (mode && !ours && time_trial(mode))
        keep_time_trial_lists(mode);
    last_press = entrant_list ? entrant_list : "";
    if (ours && !mode && logged_narrow++ < LOG_FIRST)
        swofty::log("[era] %s pressed but the container has no mode; %s is not narrowed", entrant_list, logic::HOST);
    last_pick = ours ? entrant_list : "";
    swofty_core::apply::moments::box_picked(entrant_list, mode);
    if (auto *modes = setup_owner(mode))
        ref<void *>(modes, SELECTED_LIST) = nullptr;
    select_orig(container, entrant_list);
}

uintptr_t need(const char *id, std::string &missing)
{
    uintptr_t a = swofty::api->resolve(id);
    if (!a)
        missing += std::string(missing.empty() ? "" : ", ") + id;
    return a;
}

bool bind_all(std::string &why)
{
    std::string missing;
    b.site = need("site_FeTile_EraTilesBuilt", missing);
    b.press = need("FeTile_Press_EraTile", missing);
    b.select = need("Game_F1WorldActivityContainer_SelectEntrantList", missing);
    b.add = need("FeTile_Add_EraTile", missing);
    b.available = need("FeTile_Is_EraScenarioAvailable_Thunk", missing);
    b.list_child = need("Vm_GetOrCreate_ListChild", missing);
    b.node_ref = need("Vm_Set_NodeRef", missing);
    b.set_bool = need("Vm_Set_Bool", missing);
    b.clear = need("FeTile_Clear_EraTileSet_Thunk", missing);
    b.registry = need("g_Vm_Registry", missing);
    if (missing.empty())
        return true;
    why = "needs " + missing + ", which this build does not resolve";
    return false;
}

std::string caller(const void *from)
{
    std::string dll, id = settings_addon(from, dll);
    return id.empty() ? (dll.empty() ? std::string("unknown module") : dll) : id;
}

std::string text(const char *s)
{
    return s ? s : "";
}

int api_available()
{
    return !off.load();
}

int api_add_box(const SwoftyEraBox *desc)
{
    std::string who = caller(__builtin_return_address(0));
    if (!desc || desc->size < BOX_TIME_TRIAL) {
        swofty::log("[era] box from %s refused: no descriptor, or its size %u is below %zu", who.c_str(),
                    desc ? desc->size : 0, BOX_TIME_TRIAL);
        return SWOFTY_ASSET_INVALID;
    }
    logic::Box box{text(desc->entrant_list), text(desc->title), text(desc->image), desc->order, who, 0, {}};
    if (desc->size >= sizeof(SwoftyEraBox))
        box.time_trial = text(desc->time_trial_entrant_list);
    std::lock_guard<std::mutex> g(lock);
    std::string why = logic::check(box, boxes);
    if (!why.empty()) {
        swofty::log("[era] box %s from %s refused: %s", box.entrant_list.c_str(), who.c_str(), why.c_str());
        return SWOFTY_ASSET_INVALID;
    }
    box.seq = boxes.size();
    boxes.push_back(box);
    std::string state = off ? "off: " + off_why : live ? "live from the next build of the picker" : "queued for the next mixin commit";
    std::string tt = box.time_trial.empty() ? std::string() : ", Time Trial list " + box.time_trial;
    swofty::log("[era] box %s from %s registered (title %s, image %s, order %d%s): %s", box.entrant_list.c_str(),
                who.c_str(), box.title.c_str(), logic::image_of(box).c_str(), box.order, tt.c_str(), state.c_str());
    return off ? SWOFTY_ASSET_UNAVAILABLE : SWOFTY_ASSET_OK;
}

int api_count()
{
    std::lock_guard<std::mutex> g(lock);
    return int(boxes.size());
}

const SwoftyEra table = {api_available, api_add_box, api_count};

}

extern "C" __declspec(dllexport) const SwoftyEra *SwoftyCoreEra(void)
{
    return &table;
}

namespace runtime {

const SwoftyEra *era_table()
{
    return &table;
}

std::vector<era::Box> era_boxes()
{
    std::lock_guard<std::mutex> g(lock);
    return logic::ordered(boxes);
}

void era_session_load()
{
    static uint32_t logged, swapped;
    static std::string last_seen;
    std::lock_guard<std::mutex> g(held_lock);
    put_back();
    uintptr_t slot = swofty::api ? swofty::api->resolve("g_Game_Manager") : 0, mgr = 0, ctx = 0, setup = 0;
    if (!slot || !mem::read(slot, mgr) || !mgr || !mem::read(mgr + offsetof(swofty::game::GameManager, mode_context), ctx) ||
        !ctx || !time_trial(reinterpret_cast<void *>(ctx)))
        return;
    uintptr_t holder = 0, gsm = 0, gsm_holder = 0, gsm_name = 0, entrant_setup = 0, list = 0;
    if (!mem::read(ctx + MODE_SETUP, holder) || !holder || !mem::read(holder + SETUP_MODES, setup) || !setup)
        return;
    std::string setup_mode = "?";
    if (mem::read(setup + SETUP_GAME_SETUP_MODE, gsm) && gsm && mem::read(gsm + SETUP_MODE_NAME, gsm_holder) && gsm_holder &&
        mem::read(gsm_holder + 0x10, gsm_name))
        mem::c_string(gsm_name, setup_mode, 64);
    if (!mem::read(setup + SETUP_ENTRANT_SETUP, entrant_setup) || !entrant_setup || !mem::read(setup + SELECTED_LIST, list))
        return;
    std::vector<void *> ptrs;
    auto lists = lists_of(reinterpret_cast<const uint8_t *>(entrant_setup), ptrs);
    logic::ListCars selected = list ? list_cars(reinterpret_cast<void *>(list)) : logic::ListCars{};
    logic::ListCars first = lists.empty() ? logic::ListCars{} : lists[0];
    std::string seen = setup_mode + "/" + selected.name + "/" + std::to_string(selected.cars);
    if (seen != last_seen && logged++ < LOG_FIRST * 2)
        swofty::log("[era] Time Trial session loads list %s (%lld cars) of setup mode %s (box pressed: %s)",
                    selected.name.empty() ? "?" : selected.name.c_str(), (long long)selected.cars, setup_mode.c_str(),
                    last_press.empty() ? "none" : last_press.c_str());
    last_seen = seen;
    if (!logic::oversized(selected.cars) && !logic::oversized(first.cars))
        return;
    logic::ListCars bad = logic::oversized(selected.cars) ? selected : first;
    std::vector<std::string> prefer{last_press};
    {
        std::lock_guard<std::mutex> b(lock);
        for (const auto &box : boxes)
            if (box.entrant_list == last_press)
                prefer.push_back(logic::list_for(box, true));
    }
    int64_t i = logic::time_trial_fallback(lists, prefer);
    auto *esm = reinterpret_cast<uint8_t *>(entrant_setup);
    held = {esm, 0, 0, nullptr, nullptr};
    if (i >= 0) {
        void *fit = ptrs[size_t(i)];
        ref<void *>(reinterpret_cast<void *>(setup), SELECTED_LIST) = fit;
        if (logic::oversized(first.cars) && i > 0) {
            held.first = ptrs[0];
            held.fit = fit;
            held.slot = size_t(i);
            ref<void *>(esm, LISTS + size_t(i) * 8) = ptrs[0];
            ref<void *>(esm, LISTS) = fit;
        }
    }
    int32_t limit = 0;
    if (mem::read(entrant_setup + ENTRANT_SETUP_MAX_ENTRANTS, limit) && logic::time_trial_limit(limit) != limit) {
        held.limit_was = limit;
        ref<int32_t>(esm, ENTRANT_SETUP_MAX_ENTRANTS) = logic::time_trial_limit(limit);
    }
    holding = held.first || held.limit_was;
    if (swapped++ < LOG_FIRST * 2)
        swofty::log("[era] %s", logic::time_trial_swap(bad.name, bad.cars, setup_mode, i >= 0 ? lists[size_t(i)].name : "",
                                                       held.limit_was)
                                    .c_str());
}

constexpr const char *AVAILABILITY_MIXIN = "swofty_core.dll (setup mode guard)";
constexpr const char *AVAILABILITY_FN = "Sess_Apply_ContentAvailability";
using ContentAvailability_t = void(void *self, void *availability);
ContentAvailability_t *availability_orig;
std::atomic<bool> availability_bound, logged_dropped;

void drop_setup_pointers()
{
    logic::SetupPointers p;
    {
        std::lock_guard<std::mutex> g(held_lock);
        p.held = holding;
        holding = false;
        held = {};
    }
    p.narrowed = narrowed_count;
    narrowed_count = 0;
    p.drs = drs.forget();
    last_drs_line.clear();
    std::string line = logic::availability_drop_line(p);
    if (!line.empty() && !logged_dropped.exchange(true))
        swofty::log("[era] %s", line.c_str());
}

void availability_entry(void *self, void *availability)
{
    drop_setup_pointers();
    availability_orig(self, availability);
}

std::vector<CoreMixins> tt_guard_mixins()
{
    std::vector<CoreMixins> out;
    if (vehicles_offered || !code_ready())
        return out;
    vehicles_offered = true;
    if (!swofty::api->resolve(VEHICLE_FN)) {
        swofty::log("[era] session vehicle guard off: needs %s, which this build does not resolve", VEHICLE_FN);
    } else {
        vehicles_bound = true;
        out.push_back(
            {VEHICLE_MIXIN, {{VEHICLE_FN, SWOFTY_MIXIN_WRAP, 100, (void *)&vehicles_entry, (void **)&vehicles_orig}}});
    }
    if (!swofty::api->resolve(AVAILABILITY_FN)) {
        swofty::log("[era] setup mode guard off: needs %s, which this build does not resolve", AVAILABILITY_FN);
    } else {
        availability_bound = true;
        out.push_back({AVAILABILITY_MIXIN,
                       {{AVAILABILITY_FN, SWOFTY_MIXIN_WRAP, 100, (void *)&availability_entry, (void **)&availability_orig}}});
    }
    return out;
}

void tt_guard_commit()
{
    if (!vehicles_offered || vehicles_settled)
        return;
    vehicles_settled = true;
    if (vehicles_bound)
        swofty::log("[era] session vehicle guard %s",
                    core_mixins_active(VEHICLE_MIXIN)
                        ? "armed: a session with more vehicles than its table of 26 sets up the first 26 instead of writing past it"
                        : "did not install (see the [core] lines)");
    if (availability_bound)
        swofty::log("[era] setup mode guard %s",
                    core_mixins_active(AVAILABILITY_MIXIN)
                        ? "armed: when the game rebuilds its setup modes the core drops the pointers it keeps into them"
                        : "did not install (see the [core] lines)");
}

void era_session_end()
{
    std::lock_guard<std::mutex> g(held_lock);
    put_back();
}

std::vector<CoreMixins> era_mixins()
{
    std::vector<CoreMixins> out;
    if (offered || off || !code_ready())
        return out;
    {
        std::lock_guard<std::mutex> g(lock);
        if (boxes.empty())
            return out;
    }
    offered = true;
    std::string why;
    if (!bind_all(why)) {
        off_why = why;
        off = true;
        swofty::log("[era] addon row off: %s", why.c_str());
        return out;
    }
    out.push_back({MIXIN,
                   {{"site_FeTile_EraTilesBuilt", SWOFTY_MIXIN_SITE, 100, (void *)&site_entry, nullptr},
                    {"FeTile_Press_EraTile", SWOFTY_MIXIN_WRAP, 100, (void *)&press_entry, (void **)&press_orig},
                    {"Game_F1WorldActivityContainer_SelectEntrantList", SWOFTY_MIXIN_WRAP, 100, (void *)&select_entry,
                     (void **)&select_orig}}});
    swofty::log("[era] %d box(es) registered; offering the era picker hooks", api_count());
    return out;
}

void era_commit()
{
    if (!offered || settled || off)
        return;
    settled = true;
    if (core_mixins_active(MIXIN)) {
        live = true;
        swofty::log("[era] era picker hooks installed; addon boxes show from the next build of the picker");
        return;
    }
    off_why = "the era picker hooks did not install (see the [core] lines for " + std::string(MIXIN) + ")";
    off = true;
    swofty::log("[era] addon row off: %s", off_why.c_str());
}

}
