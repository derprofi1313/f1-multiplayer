#include "runtime.hpp"

#include <safetyhook.hpp>
#include <swofty/game.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <cstring>
#include <mutex>
#include <string>

#include "json.hpp"
#include "netplay_drive_logic.hpp"
#include "netplay_room.hpp"
#include "room_cover.hpp"

// Room set-up with no LAN tile and no main menu (research/multiplayer/ns-resolve.md, carcat-direct.md).
//
// The network subsystem comes from the game manager's services map, the LAN mode switch is the tile's
// getAction made by hand (game mode online_custom, then changeMPMode(eLan)), and:
//
//  - a joiner writes the pending join request and waits until it has joined;
//  - a host sets the front end's transition state to custom_game (a LAN game, so no online login), lets the
//    direct start release the flow, and sends open_create_custom to the f1_world_activity_container the
//    moment it exists, which opens the Car Category tab for the player to pick a box on;
//  - SWOFTY_ROOM_HOST=headless keeps the old host that writes the Car Category values and calls CustomCreate
//    itself.
//
// The rest of this comment is the headless path (headless-trace.md, gamesessionui.md), which a joiner
// and the headless host still use. Headless LAN host and join (research/multiplayer/headless-trace.md, gamesessionui.md).
//
// netplay_drive reaches a room the way a player does: Local Multiplayer, the LAN tile, the LAN Game dialog,
// then the Car Category screen for a host or the Join via IP prompt for a client. With the game on its
// main menu and the private LAN item initialised (the probe in netplay_drive.cpp), the same game state can
// be written and the same game functions called with none of those screens:
//
//  - the mode switch is the LAN tile's getAction on the private item (online_custom, then eLan);
//  - a host writes what the Car Category confirm writes (the DLC domain, the cluster id) and calls
//    GameSessionUI's CustomCreate (0x1430ae3f0) on *(ns+0x460), which sets up the session and hands a
//    SessionCreateCustom invite to the invite manager;
//  - a client fills the pending request record the Join via IP prompt fills; the pump at 0x1443dca90 hands
//    it to the invite manager as a SessionInvite.
//
// Either invite raises an interrupt the game routes to the multiplayer screen's onInviteJoinNotification,
// which asks the invite manager to join. With no screen that routing may not happen. The run watches the
// invite manager's join delegate (+0xb78); if the invite is there and the delegate is still empty after
// INTERRUPT_MS it calls onInviteJoinNotification(gsui, 0) itself, and logs that it did. The game's
// StateOnlineLobby then ends the run.
//
// Everything runs on the flow thread, one step per runtime::netplay_tick, and the game is never touched
// before the whole plan has checked out: a missing precondition makes set_up return false so the caller can
// take the screen-driven path instead.

namespace runtime::netplay_room::bind {
void *network();
void *registry();
bool on_home();
bool in_lobby();
bool armed();
bool provide_context(void *registry, void *context);
}

namespace runtime {
bool car_category_choose(const std::string &value, std::string &why);
std::string hook_error(const safetyhook::MidHook::Error &e);
}

bool ui_vm_set_path(const std::string &path, int type, const void *value);

namespace {

namespace room = runtime::netplay_room;

const runtime::Sig S_CUSTOM_CREATE{"game.network_custom_create", "56 48 81 EC 70 01 00 00 48 89 CE E8", 0x30ae3f0};
const runtime::Sig S_INVITE_NOTIFY{"game.game_session_ui_on_invite_join_notification",
                                   "56 57 53 48 83 EC 40 48 89 CE 80 3D ?? ?? ?? ?? 01 75 66 B8 70 01 00 00", 0x30acca0};
const runtime::Sig S_REQUEST_KEY{"game.join_request_owner_key",
                                 "48 8D 15 ?? ?? ?? ?? 48 89 F9 FF 50 28 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 18", 0x6cb8278,
                                 runtime::Sig::riprel, 0, 3, 7};

const char *const LOOKUP_SITE =
    "48 8D 05 ?? ?? ?? ?? 48 89 44 24 28 48 8D 54 24 28 48 89 D9 E8 ?? ?? ?? ?? 48 C1 E0 04 48 03 43 18 48 3B 43 20 0F 84 ?? "
    "?? ?? ?? 48 8B 40 08 48 89 86 50 01 00 00 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 07 48 8D 15 ?? ?? ?? ?? 48 89 F9 FF 50 28 "
    "48 85 C0 0F 84";
const runtime::Sig S_MANAGER{"game.manager",
                             "48 8B B8 98 01 00 00 48 85 FF 0F 84 ?? ?? ?? ?? 48 8D 4C 24 70 E8 ?? ?? ?? ?? 48 8B 05",
                             0x604c2c8, runtime::Sig::riprel, 26, 29, 7};
const runtime::Sig S_SERVICE_FIND{"game.service_map_find", LOOKUP_SITE, 0x30ffd0, runtime::Sig::rel32, 0x14, 0x15, 5};
const runtime::Sig S_NS_KEY{"game.net_subsystem_key", LOOKUP_SITE, 0x60bed80, runtime::Sig::riprel, 0, 3, 7};
const runtime::Sig S_NS_VTABLE{"game.net_subsystem_vtable",
                                "48 8D 05 ?? ?? ?? ?? 48 89 06 48 8D 05 ?? ?? ?? ?? 48 89 46 20 48 8D 05 ?? ?? ?? ?? 48 89 46 28 C5 C8 57 F6",
                                0x5814050, runtime::Sig::riprel, 0, 3, 7};
const runtime::Sig S_SET_MODE{"game.game_set_current_game_mode", "41 56 56 57 55 53 48 81 EC 80 00 00 00 48 8B 81 88 00 00 00", 0x7d07a0};
const runtime::Sig S_NAME_INDEX{"game.game_modes_name_index", "E9 ?? ?? ?? ?? EF 7B F2 64 0F 84 ?? ?? ?? ?? 2B CA", 0x9763b0};
const runtime::Sig S_STR_VTABLE{"game.ne_inline_str_vtable",
                                "48 8D 05 ?? ?? ?? ?? 48 89 44 24 28 48 8D 54 24 28 E8 ?? ?? ?? ?? 48 8B 4B 58 48 8B", 0x5612ea8,
                                runtime::Sig::riprel, 0, 3, 7};
const runtime::Sig S_MP_MODE{"game.network_subsystem_change_mp_mode",
                             "41 57 41 56 41 54 56 57 55 53 48 81 EC 20 02 00 00 C5 F8 29 B4 24 10 02 00 00", 0x30ed330};
const runtime::Sig S_PRESENCE{"game.network_mox_session_generic",
                              "56 57 48 83 EC 28 48 89 CE 80 3D ?? ?? ?? ?? 01 75 60 B8 00 01 00 00 48 8D 0D ?? ?? ?? ?? "
                              "66 90 8B 14 81 81 FA BB 5E FE A7 74 34 85 D2 74 43 83 C0 19 48 69 D0 43 43 66 5F 48 C1 EA "
                              "20 41 89 C0 41 29 D0 41 D1 E8 41 01 D0 41 C1 E8 08 41 69 D0 75 01 00 00 29 D0 3D 00 01 00 "
                              "00 75 C3 EB 13 48 8D 0D ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B 46 40",
                              0x3083570};
const runtime::Sig S_ADD_TABS{"game.f1_world_container_add_tabs",
                              "41 57 41 56 41 55 41 54 56 57 55 53 48 81 EC F8 00 00 00 C5 F8 29 B4 24 E0 00 00 00 49 89 CF B1",
                              0x1a76130};

using CustomCreate_t = void(void *gsui);
using Find_t = uint64_t(void *map, const uint64_t *key);
using SetMode_t = uint64_t(void *modes, void *mode);
using NameIndex_t = uint64_t(void *names, void *name);
using Presence_t = void(void *ns);
using MpMode_t = void *(void *ns, uint8_t *lan, uint32_t what);
using Send_t = void(void *sink, swofty::game::NeStr **name, void *data);
using InviteNotify_t = void(void *gsui, uint32_t what);
using Lookup_t = void *(void *registry, const void *key);

constexpr size_t REGISTRY_LOOKUP_SLOT = 5;

std::mutex lock;
room::Role role = room::Role::host;
room::Category category;
room::Run run;
room::HostPlan host_plan;
room::JoinPlan join_plan;
room::Status state;
std::string error_now;
std::string join_ip;
uintptr_t a_create, a_notify, a_key, a_manager, a_find, a_ns_key, a_ns_vtable, a_set_mode, a_name_index, a_str_vtable, a_mp_mode,
    a_presence, a_add_tabs;
uint64_t ns_cached;
std::string network_note;
bool resolved, asked, host_headless, host_ui, ui_started;
std::atomic<bool> ui_rewrite{false};
room::HostUi host_ui_run;
std::string ui_logged_error;
room::UiStep ui_logged_step = room::UiStep::idle;
safetyhook::MidHook h_add_tabs;
std::atomic<void *> container{nullptr}, container_vt{nullptr};
room::Role asked_role = room::Role::host, request_role = room::Role::host;
std::string request_ip;
bool requested;

bool read_memory(uint64_t address, void *out, size_t n, void *)
{
    if (!runtime::readable(reinterpret_cast<const void *>(address), n))
        return false;
    std::memcpy(out, reinterpret_cast<const void *>(address), n);
    return true;
}

room::Memory memory()
{
    return {read_memory, nullptr};
}

bool code_pointer(const void *p)
{
    auto a = reinterpret_cast<uintptr_t>(p);
    uintptr_t base = swofty::api->module_base();
    return p && a >= base && a < base + 0x20000000 && runtime::readable(p, 1);
}

uint64_t service_find(void *, uint64_t map, uint64_t key)
{
    uint64_t k = key;
    return reinterpret_cast<Find_t *>(a_find)(reinterpret_cast<void *>(map), &k);
}

uint64_t manager()
{
    if (!a_manager || !runtime::readable(reinterpret_cast<void *>(a_manager), 8))
        return 0;
    return *reinterpret_cast<uint64_t *>(a_manager);
}

void note_network(const std::string &what)
{
    if (what == network_note)
        return;
    network_note = what;
    swofty::log("[netplay] room: %s", what.c_str());
}

void *resolve_network(std::string &why)
{
    if (ns_cached && room::net_subsystem_check(memory(), ns_cached, a_ns_vtable).empty())
        return reinterpret_cast<void *>(ns_cached);
    ns_cached = 0;
    if (!a_manager || !a_find || !a_ns_key || !a_ns_vtable) {
        why = "the game manager, the services map lookup, the network subsystem key or the network subsystem vtable did not "
              "resolve on this build";
    } else {
        uint64_t ns = 0;
        why = room::find_net_subsystem(memory(), manager(), a_ns_key, a_ns_vtable, service_find, nullptr, ns);
        if (why.empty()) {
            ns_cached = ns;
            char b[96];
            std::snprintf(b, sizeof b, "network subsystem %p from the game manager's services map",
                          reinterpret_cast<void *>(ns));
            note_network(b);
            return reinterpret_cast<void *>(ns);
        }
    }
    if (void *over = room::bind::network()) {
        note_network("network subsystem from the LAN item the caller provided (override)");
        return over;
    }
    return nullptr;
}

void *network_now()
{
    std::string why;
    return resolve_network(why);
}

int net_mode_now()
{
    void *ns = network_now();
    return ns ? room::net_mode(memory(), reinterpret_cast<uint64_t>(ns)) : -1;
}

int session_role_now()
{
    void *ns = network_now();
    return ns ? room::session_role(memory(), reinterpret_cast<uint64_t>(ns)) : -1;
}

void resolve_once()
{
    if (resolved)
        return;
    resolved = true;
    a_create = runtime::find(S_CUSTOM_CREATE, "netplay", "netplay");
    a_notify = runtime::find(S_INVITE_NOTIFY, "netplay", "netplay");
    a_key = runtime::find(S_REQUEST_KEY, "netplay", "netplay");
    a_manager = runtime::find(S_MANAGER, "game manager", "netplay");
    a_find = runtime::find(S_SERVICE_FIND, "services map lookup", "netplay");
    a_ns_key = runtime::find(S_NS_KEY, "network subsystem key", "netplay");
    a_ns_vtable = runtime::find(S_NS_VTABLE, "network subsystem vtable", "netplay");
    a_set_mode = runtime::direct_start_set_mode();
    if (!a_set_mode)
        a_set_mode = runtime::find(S_SET_MODE, "SetCurrentGameMode", "netplay");
    a_name_index = runtime::find(S_NAME_INDEX, "game mode name index", "netplay");
    a_str_vtable = runtime::find(S_STR_VTABLE, "inline string vtable", "netplay");
    a_mp_mode = runtime::find(S_MP_MODE, "changeMPMode", "netplay");
    a_presence = runtime::find(S_PRESENCE, "declareActivePresence", "netplay");
}

struct InlineStr {
    void *vtable;
    uint64_t capacity;
    char *chars;
    char inline_chars[0x28];
};

std::string switch_to_lan()
{
    void *ns = network_now();
    if (!ns)
        return "the network subsystem was not reached";
    if (!a_set_mode || !a_name_index || !a_str_vtable || !a_mp_mode || !a_presence)
        return "SetCurrentGameMode, the game mode name index, the name string vtable, changeMPMode or "
               "declareActivePresence did not resolve on this build";
    room::ModePlan plan;
    if (std::string why = room::plan_mode_switch(memory(), manager(), reinterpret_cast<uint64_t>(ns), plan); !why.empty())
        return why;
    if (plan.already) {
        swofty::log("[netplay] room: the network subsystem is already in LAN mode");
        return {};
    }
    InlineStr name{};
    name.vtable = *reinterpret_cast<void **>(a_str_vtable);
    name.capacity = 0x20;
    name.chars = name.inline_chars;
    std::memcpy(name.inline_chars, "online_custom", sizeof "online_custom");
    uint64_t index = reinterpret_cast<NameIndex_t *>(a_name_index)(reinterpret_cast<void *>(plan.modes + room::MODES_NAMES), &name);
    if (std::string why = room::mode_entity(memory(), plan.modes, index, plan); !why.empty())
        return why;
    swofty::log("[netplay] room: game mode online_custom is entry %llu (entity %p, mode %p), network mode %d",
                static_cast<unsigned long long>(index), reinterpret_cast<void *>(plan.entity),
                reinterpret_cast<void *>(plan.mode), plan.net);
    reinterpret_cast<SetMode_t *>(a_set_mode)(reinterpret_cast<void *>(plan.modes), reinterpret_cast<void *>(plan.mode));
    reinterpret_cast<Presence_t *>(a_presence)(ns);
    uint8_t lan = 1;
    reinterpret_cast<MpMode_t *>(a_mp_mode)(ns, &lan, 1);
    int after = room::net_mode(memory(), reinterpret_cast<uint64_t>(ns));
    swofty::log("[netplay] room: changeMPMode(eLan) called, network mode now %d%s", after,
                after == int(room::NET_SYSTEM_LAN) ? "" : " (the check waits for it)");
    return {};
}

void *owner_from(void *registry, const char *from, std::string &why)
{
    if (!registry || !runtime::readable(registry, 8)) {
        why += std::string(from) + ": not available; ";
        return nullptr;
    }
    auto **vt = *static_cast<void ***>(registry);
    if (!runtime::readable(vt, (REGISTRY_LOOKUP_SLOT + 1) * sizeof(void *)) || !code_pointer(vt[REGISTRY_LOOKUP_SLOT])) {
        why += std::string(from) + ": no lookup function; ";
        return nullptr;
    }
    void *owner = reinterpret_cast<Lookup_t *>(vt[REGISTRY_LOOKUP_SLOT])(registry, reinterpret_cast<const void *>(a_key));
    uint64_t record = 0;
    if (!owner || !room::Memory{read_memory, nullptr}.ptr(reinterpret_cast<uint64_t>(owner) + room::OWNER_REQUEST, record)) {
        why += std::string(from) + ": no join request owner; ";
        return nullptr;
    }
    return owner;
}

void *vm_registry()
{
    if (!swofty::game::g_Vm_Registry.address())
        return nullptr;
    return *swofty::game::g_Vm_Registry;
}

void *request_owner(std::string &why)
{
    if (!a_key) {
        why = "the join request key (g_Game_JoinRequestOwnerKey) did not resolve on this build";
        return nullptr;
    }
    std::string tried;
    void *owner = owner_from(room::bind::registry(), "the init context's registry", tried);
    const char *from = "the init context's registry";
    if (!owner) {
        owner = owner_from(vm_registry(), "the VM registry", tried);
        from = "the VM registry";
    }
    if (!owner) {
        why = "no registry holds the join request owner (" + tried + ")";
        return nullptr;
    }
    static const char *told = nullptr;
    if (told != from) {
        told = from;
        swofty::log("[netplay] room join: the join request owner %p comes from %s", owner, from);
    }
    return owner;
}

std::string plan_for(room::Role r, const std::string &ip, bool whole = true)
{
    std::string reached;
    void *ns = resolve_network(reached);
    if (!ns)
        return "the network subsystem was not reached (" + reached + ")";
    if (r == room::Role::host && !host_headless)
        return {};
    if (r == room::Role::join && !whole) {
        uint32_t parsed = 0;
        if (!room::parse_ipv4(ip, parsed) || parsed == 0 || parsed == 0xffffffffu)
            return "'" + ip + "' is not a dotted IPv4 address";
        return a_notify ? std::string() : "onInviteJoinNotification did not resolve on this build";
    }
    if (r == room::Role::host) {
        if (!a_create)
            return "CustomCreate did not resolve on this build";
        if (!a_notify)
            return "onInviteJoinNotification did not resolve on this build";
        std::string why;
        if (!runtime::car_category_choose(category.addon, why))
            return why;
        return room::plan_host(memory(), reinterpret_cast<uint64_t>(ns), category, host_plan);
    }
    if (!a_notify)
        return "onInviteJoinNotification did not resolve on this build";
    std::string why;
    void *owner = request_owner(why);
    if (!owner)
        return why;
    return room::plan_join(memory(), reinterpret_cast<uint64_t>(ns), reinterpret_cast<uint64_t>(owner), ip, join_plan);
}

bool apply(const std::vector<room::Write> &writes)
{
    for (const auto &w : writes) {
        if (!runtime::readable(reinterpret_cast<void *>(w.address), w.bytes.size()))
            return false;
    }
    for (const auto &w : writes) {
        std::atomic_thread_fence(std::memory_order_release);
        std::memcpy(reinterpret_cast<void *>(w.address), w.bytes.data(), w.bytes.size());
    }
    std::atomic_thread_fence(std::memory_order_release);
    return true;
}

void *ui_container()
{
    void *c = container.load();
    if (!c || !runtime::readable(c, room::CONTAINER_SIZE) || *static_cast<void **>(c) != container_vt.load())
        return nullptr;
    return c;
}

void tabs_mid(safetyhook::Context &c)
{
    void *v = reinterpret_cast<void *>(c.rcx);
    if (!v || !runtime::readable(v, room::CONTAINER_SIZE))
        return;
    if (container.load() != v)
        swofty::log("[netplay] room: f1_world_activity_container %p is building its tabs (network mode %d)", v,
                    net_mode_now());
    container_vt = *static_cast<void **>(v);
    container = v;
}

const char *ui_step_name(room::UiStep step)
{
    switch (step) {
    case room::UiStep::state: return "writing the custom game state";
    case room::UiStep::container: return "waiting for the Car Category's container";
    case room::UiStep::settle: return "waiting for its tab list";
    case room::UiStep::sent: return "open_create_custom sent";
    case room::UiStep::failed: return "failed";
    default: return "idle";
    }
}

bool write_ui_state(const char *when)
{
    int one = 1;
    bool a = ui_vm_set_path(room::UI_STATE_PATH, 0, room::UI_STATE_VALUE);
    bool b = ui_vm_set_path(room::UI_LAN_PATH, 1, &one);
    if (a && b) {
        swofty::log("[netplay] room host: #%s = %s and #%s = true written%s", room::UI_STATE_PATH, room::UI_STATE_VALUE,
                    room::UI_LAN_PATH, when);
        return true;
    }
    static bool told = false;
    if (!told) {
        told = true;
        swofty::log("[netplay] room host: ui_menu_data is not there yet (state %s, lan flag %s); retrying",
                    a ? "written" : "missing", b ? "written" : "missing");
    }
    return false;
}

void ui_act(room::UiDo d, uint64_t now)
{
    if (d == room::UiDo::write_state) {
        static uint64_t last_try = 0;
        if (now - last_try < 250)
            return;
        last_try = now;
        if (write_ui_state(""))
            host_ui_run.wrote(now);
    } else if (d == room::UiDo::send_open) {
        void *c = ui_container();
        auto **vt = c ? *static_cast<void ***>(c) : nullptr;
        constexpr size_t SEND_SLOT = 0x80 / 8;
        if (!c || !runtime::readable(vt, (SEND_SLOT + 1) * sizeof(void *)) || !code_pointer(vt[SEND_SLOT])) {
            host_ui_run.fail("the container has no message handler to send open_create_custom to");
            return;
        }
        struct Data {
            uint32_t kind = 4, zero = 0;
            uint64_t more = 0;
        } data;
        swofty::game::NeStr *name = nullptr;
        swofty::game::Ne_Make_StrFromCStr(&name, room::UI_OPEN_MESSAGE, 0, 0);
        reinterpret_cast<Send_t *>(vt[SEND_SLOT])(c, &name, &data);
        swofty::game::Ne_Make_StrFromCStr(&name, nullptr, 0, 0);
        swofty::log("[netplay] room host: sent %s to the container %p%s; the Car Category tab is next", room::UI_OPEN_MESSAGE, c,
                    host_ui_run.sent_unready() ? " (its tab list never read as ready, sent anyway)" : "");
    }
}

void cover_ui(room::UiStep s);

void ui_tick()
{
    if (!ui_started)
        return;
    uint64_t now = GetTickCount64();
    if (ui_rewrite.exchange(false) && host_ui_run.written() && host_ui_run.active())
        write_ui_state(" again (frontend_transition opened)");
    if (host_ui_run.active()) {
        room::UiFacts f;
        void *c = ui_container();
        f.container = c != nullptr;
        f.ready = c && room::container_facts(memory(), reinterpret_cast<uint64_t>(c)).ready;
        ui_act(host_ui_run.tick(f, now), now);
    }
    if (host_ui_run.step != ui_logged_step) {
        ui_logged_step = host_ui_run.step;
        cover_ui(host_ui_run.step);
        swofty::log("[netplay] room host: %s", ui_step_name(host_ui_run.step));
    }
    if (host_ui_run.failed() && host_ui_run.error != ui_logged_error) {
        ui_logged_error = host_ui_run.error;
        swofty::log("[netplay] room host failed: %s", host_ui_run.error.c_str());
    }
}

void cover_stage(room::Stage s, room::Role r)
{
    if (s == room::Stage::call)
        runtime::room_cover::step(r == room::Role::host ? "Creating the session" : "Reaching the host", 0.78f, 0.86f);
    else if (s == room::Stage::interrupt)
        runtime::room_cover::step(r == room::Role::host ? "Creating the session" : "Reaching the host", 0.82f, 0.9f);
    else if (s == room::Stage::lobby)
        runtime::room_cover::step(r == room::Role::host ? "Opening the lobby" : "Joining the lobby", 0.9f, 0.97f);
}

void cover_ui(room::UiStep s)
{
    if (s == room::UiStep::state)
        runtime::room_cover::step("Setting up the lobby", 0.82f, 0.88f);
    else if (s == room::UiStep::container)
        runtime::room_cover::step("Opening Car Category", 0.88f, 0.93f);
    else if (s == room::UiStep::settle)
        runtime::room_cover::step("Opening Car Category", 0.94f, 0.98f);
}

void finish(room::Phase phase, const std::string &detail)
{
    state.phase = phase;
    state.detail = detail;
    swofty::log("[netplay] room %s: %s %s", room::role_name(role), phase == room::Phase::done ? "ready:" : "failed:",
                detail.c_str());
}

void act(room::Do d)
{
    std::string err;
    switch (d) {
    case room::Do::mode_switch:
        err = switch_to_lan();
        break;
    case room::Do::host_call: {
        err = plan_for(room::Role::host, {});
        if (!err.empty())
            break;
        if (!apply(host_plan.writes)) {
            err = "the Car Category values could not be written";
            break;
        }
        swofty::log("[netplay] room host: wrote DLC domain 0x%x at ns+0x4f4 and cluster id -1 (settings %p), calling "
                    "CustomCreate(gsui %p)%s%s",
                    unsigned(category.preset), reinterpret_cast<void *>(host_plan.settings),
                    reinterpret_cast<void *>(host_plan.gsui), category.addon.empty() ? "" : ", addon category ",
                    category.addon.c_str());
        reinterpret_cast<CustomCreate_t *>(a_create)(reinterpret_cast<void *>(host_plan.gsui));
        break;
    }
    case room::Do::join_write: {
        err = plan_for(room::Role::join, join_ip);
        if (!err.empty())
            break;
        if (!apply(join_plan.writes)) {
            err = "the join request could not be written";
            break;
        }
        swofty::log("[netplay] room join: wrote the pending request at %p: LAN, %s port %u, connection id 0x%08x",
                    reinterpret_cast<void *>(join_plan.request), join_ip.c_str(), unsigned(join_plan.port),
                    room::connection_id(join_plan.ip, join_plan.port));
        break;
    }
    case room::Do::fallback_notify: {
        uint64_t gsui = role == room::Role::host ? host_plan.gsui : join_plan.gsui;
        swofty::log("[netplay] room %s: the invite reached the invite manager but the interrupt raised no join "
                    "within the wait; calling onInviteJoinNotification(gsui %p, 0) by hand (fallback path)",
                    room::role_name(role), reinterpret_cast<void *>(gsui));
        reinterpret_cast<InviteNotify_t *>(a_notify)(reinterpret_cast<void *>(gsui), 0);
        break;
    }
    default:
        return;
    }
    if (d != room::Do::nothing)
        swofty::log("[netplay] room %s: step %s%s%s", room::role_name(role), room::do_name(d),
                    err.empty() ? "" : " failed: ", err.c_str());
    if (!err.empty())
        run.fail(err);
}

}

namespace runtime::netplay_room {

namespace {

bool begin(Role r, const char *host_ip, bool from_home, std::string &why)
{
    auto refuse = [&](const std::string &reason) {
        why = error_now = reason;
        swofty::log("[netplay] room %s not started headless: %s", role_name(r), reason.c_str());
        return false;
    };
    if (!bind::armed())
        return refuse("netplay is not armed (SWOFTY_NETPLAY is unset)");
    if (run.active())
        return refuse("a headless set-up is already running");
    if (from_home && !bind::on_home())
        return refuse("the game is not on its main menu");
    if (bind::in_lobby())
        return refuse("the game is already in a LAN lobby");
    resolve_once();
    role = r;
    join_ip = host_ip ? host_ip : "";
    std::string reason = plan_for(r, join_ip, false);
    if (!reason.empty())
        return refuse(reason);
    if (int now = session_role_now(); now == 1 || now == 2)
        return refuse("the game already has a session (role " + std::to_string(now) + ")");
    error_now.clear();
    host_ui = r == Role::host && !host_headless;
    ui_started = false;
    ui_logged_step = UiStep::idle;
    ui_logged_error.clear();
    run.start(r, GetTickCount64(), from_home, host_ui);
    state = {Phase::running, "mode switch"};
    runtime::room_cover::step("Switching the game mode", 0.6f, 0.72f);
    swofty::log("[netplay] room %s: set-up started %s (plan checked: %s)", role_name(r),
                from_home ? "from the main menu" : "before the main menu",
                host_ui ? "the network subsystem, then the front end's custom game state and the Car Category tab"
                : r == Role::host ? "GameSessionUI, session settings, invite manager, request clear"
                                  : "the network subsystem, invite handler, ip address");
    return true;
}

}

bool provide_context(void *registry, void *context)
{
    return bind::provide_context(registry, context);
}

bool set_up_from_home(Role r, const char *host_ip, std::string &why)
{
    std::lock_guard<std::mutex> g(lock);
    return begin(r, host_ip, true, why);
}

Setup set_up(Role r, const char *host_ip, std::string &why)
{
    tick();
    std::lock_guard<std::mutex> g(lock);
    if (!asked) {
        asked = true;
        asked_role = r;
        static const char *mode = [] {
            static char v[16];
            DWORD n = GetEnvironmentVariableA("SWOFTY_ROOM_HOST", v, sizeof v);
            return n && n < sizeof v ? v : "";
        }();
        if (r == Role::host && !std::strcmp(mode, "headless")) {
            host_headless = true;
            swofty::log("[netplay] room host: SWOFTY_ROOM_HOST=headless, so the host skips the Car Category tab and "
                        "calls CustomCreate itself");
        }
        if (!begin(r, host_ip, false, why)) {
            state = {Phase::failed, why};
            return Setup::failed;
        }
    } else if (asked_role != r) {
        why = "a headless set-up for the other role was already asked for";
        return Setup::failed;
    }
    if (host_ui && state.phase == Phase::done) {
        if (host_ui_run.failed()) {
            why = host_ui_run.error;
            return Setup::failed;
        }
        if (!host_ui_run.released(GetTickCount64())) {
            why = "writing the front end's custom game state";
            return Setup::pending;
        }
        why = state.detail;
        return Setup::done;
    }
    why = state.detail;
    return state.phase == Phase::done ? Setup::done : state.phase == Phase::failed ? Setup::failed : Setup::pending;
}

void set_room_request(Role r, const std::string &host_ip)
{
    std::lock_guard<std::mutex> g(lock);
    request_role = r;
    request_ip = host_ip;
    requested = true;
}

bool room_requested()
{
    std::lock_guard<std::mutex> g(lock);
    return requested;
}

Setup room_poll(std::string &why)
{
    Role r;
    std::string ip;
    {
        std::lock_guard<std::mutex> g(lock);
        if (!requested) {
            why = "no room request (role and host address) was set";
            return Setup::failed;
        }
        r = request_role;
        ip = request_ip;
    }
    return set_up(r, ip.c_str(), why);
}

void set_category(const Category &c)
{
    std::lock_guard<std::mutex> g(lock);
    category = c;
}

Status status()
{
    std::lock_guard<std::mutex> g(lock);
    return state;
}

std::string last_error()
{
    std::lock_guard<std::mutex> g(lock);
    return error_now;
}

void tick()
{
    std::lock_guard<std::mutex> g(lock);
    ui_tick();
    if (!run.active())
        return;
    Facts f;
    f.home = bind::on_home();
    f.lobby = bind::in_lobby();
    f.lan_mode = net_mode_now() == int(NET_SYSTEM_LAN);
    f.session_role = session_role_now();
    if (run.role == Role::join && run.stage == Stage::call) {
        std::string why;
        f.join_ready = request_owner(why) != nullptr;
    }
    if (run.role == Role::join) {
        uint8_t pending = 1;
        f.pending_taken = join_plan.request && read_memory(join_plan.request + REQ_PENDING, &pending, 1, nullptr) && !pending;
    }
    uint64_t invites = run.role == Role::host ? host_plan.invites : join_plan.invites;
    if (invites)
        f.invite = invite_state(memory(), invites);
    Stage before = run.stage;
    Do d = run.tick(f, GetTickCount64());
    if (d != Do::nothing)
        act(d);
    if (run.stage != before && run.stage != Stage::failed && run.stage != Stage::done) {
        cover_stage(run.stage, run.role);
        state.detail = run.stage == Stage::call        ? "calling the game"
                       : run.stage == Stage::interrupt ? "waiting for the invite to reach the join"
                                                       : "waiting for the session";
    }
    if (run.done() && host_ui) {
        finish(Phase::done, "the game is in LAN mode; the Car Category tab is next");
        host_ui_run.start(GetTickCount64());
        ui_started = true;
    } else if (run.done())
        finish(Phase::done, run.by_hand ? "the session is established (join requested by hand)" : "the session is established");
    else if (run.failed())
        finish(Phase::failed, run.error);
}

CoverSignal cover_signal()
{
    std::unique_lock<std::mutex> g(lock, std::try_to_lock);
    if (!g.owns_lock())
        return CoverSignal::waiting;
    if (state.phase == Phase::failed || (host_ui && host_ui_run.failed()))
        return CoverSignal::failed;
    if (host_ui)
        return host_ui_run.sent() ? CoverSignal::shown : CoverSignal::waiting;
    return state.phase == Phase::done ? CoverSignal::shown : CoverSignal::waiting;
}

}

namespace runtime {

namespace {

bool load_request()
{
    namespace room = netplay_room;
    char env[1024] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_DIRECT_START", env, sizeof env);
    std::string text;
    if (n && n < sizeof env && env[0] == '@')
        text = read_file(wide(env + 1), size_t(1) << 16);
    else if (!n)
        text = read_file(swofty_root() + L"direct_start.json", size_t(1) << 16);
    auto request = drive::parse_room_request(text);
    if (!request.room)
        return false;
    if (!request.error.empty()) {
        swofty::log("[netplay] room: %s", request.error.c_str());
        if (request.role != "host" && request.role != "join")
            return false;
    }
    room::set_room_request(request.role == "host" ? room::Role::host : room::Role::join, request.host_ip);
    swofty::log("[netplay] room: the direct start request is a %s%s%s", request.role.c_str(),
                request.role == "join" ? " of " : "", request.role == "join" ? request.host_ip.c_str() : "");
    if (drive::apply_room_config(request)) {
        swofty::log("[netplay] room: configured from the direct start request, self=%d base=%d%s%s", request.self,
                    request.base, request.name.empty() ? "" : ", driver name ", request.name.c_str());
        netplay_bridge_status_changed();
    }
    return true;
}

netplay_room::Setup adapter(std::string &why)
{
    return netplay_room::room_poll(why);
}

}

void netplay_room_setup()
{
    load_request();
    uintptr_t tabs = find(S_ADD_TABS, "container tabs", "netplay");
    if (tabs) {
        auto h = safetyhook::MidHook::create(reinterpret_cast<void *>(tabs), tabs_mid);
        if (h)
            h_add_tabs = std::move(*h);
        else
            swofty::log("[netplay] room: the hook on the container's tab builder failed: %s", hook_error(h.error()).c_str());
    } else {
        swofty::log("[netplay] room: the container's tab builder did not resolve; a host cannot open the Car Category tab");
    }
    direct_start_register_room([](std::string &why) {
        return static_cast<direct_start::Setup>(adapter(why));
    });
}

bool netplay_room_requested()
{
    return netplay_room::room_requested();
}

void netplay_room_screen(const char *name)
{
    if (name && !std::strcmp(name, "frontend_transition") && ui_started && host_ui_run.written())
        ui_rewrite = true;
}

}
