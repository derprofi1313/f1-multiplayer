#include "runtime.hpp"

#include <safetyhook.hpp>
#include <swofty/mixin.hpp>

#include <atomic>
#include <vector>
#include <cstdio>
#include <cctype>
#include <cstring>
#include <map>
#include <mutex>

#include "direct_start_logic.hpp"
#include "json.hpp"
#include "room_cover.hpp"
#include "testagent/agent_logic.hpp"
#include "title_skip.hpp"

// The direct start: the game goes from boot to its target with no title, no legal screens' waits and no
// menu. SWOFTY_DIRECT_START in the game's environment, or swofty\direct_start.json written by the launcher for
// this launch, names the target: a session,
// or a room (netplay_drive.cpp). The title engages by itself (title_skip.cpp), which loads the profile;
// the target sets itself up on the flow thread; then the flow leaves the front end from StateFTUE, whose
// finished link is held until then and replaced by its exit link, so StateFrontend (the menu) never starts.
// StateFrontendWorld, with no child left, then takes the target's link ('session' leads to StateLaunch's
// path and StateLoadSession).
//
// The flow machine (Vm_Update_FlowStateMachine 0x144423ce0) calls each state's onStateUpdate (vt+0x20,
// the shared wrapper that updates children first) at 0x144424c75 with rdi = the state and the returned
// link written to [rsp+0x40]; a mid hook on the next instruction (0x144424c78, where it compares that
// link with g_Game_FlowLinkStay) sees every state's link and can replace it. State nodes: +0xb0 the
// name's djb2 hash, +0xbd8 the active child.

namespace runtime {
std::string hook_error(const safetyhook::MidHook::Error &e);
}

namespace {

namespace ds = runtime::direct_start;

const runtime::Sig SIG_FLOW_LINK{"vm.flow_state_update_link",
                                 "48 8B 07 48 8D 54 24 40 48 89 F9 C5 F9 28 D6 FF 50 20 8B 44 24 40 39 05", 0x4424c78,
                                 runtime::Sig::addr, 0, 0x12};
const runtime::Sig SIG_GAME_MANAGER{"game.manager", "48 8B B8 98 01 00 00 48 85 FF 0F 84 ?? ?? ?? ?? 48 8D 4C 24 70 E8 ?? ?? ?? ?? 48 8B 05", 0x604c2c8, runtime::Sig::riprel, 26, 29, 7};
const runtime::Sig SIG_SET_MODE{"game.game_set_current_game_mode", "41 56 56 57 55 53 48 81 EC 80 00 00 00 48 8B 81 88 00 00 00", 0x7d07a0};
const runtime::Sig SIG_MODE_BY_PREDICATE{"game.game_modes_find_by_predicate",
                                         "48 8D 54 24 28 E8 ?? ?? ?? ?? 48 85 C0 74 22 48 89 C7 48 8B 8E 08 03 00 00",
                                         0x7d2280, runtime::Sig::rel32, 5, 6, 5};
const runtime::Sig SIG_GP_PREDICATE{"game.grand_prix_mode_predicate",
                                    "48 8D 05 ?? ?? ?? ?? 48 89 44 24 28 48 8D 54 24 28 E8 ?? ?? ?? ?? 48 85 C0 74 22",
                                    0x77cff0, runtime::Sig::riprel, 0, 3, 7};
const runtime::Sig SIG_BUILD_SETTINGS{"game.championship_game_mode_build_session_settings",
                                      "41 57 41 56 41 55 41 54 56 57 55 53 B8 68 68 00", 0x7b7b90};

constexpr uintptr_t MANAGER_MODES = 0xc0, ENTITY_REGISTRY = 0x1030, ENTITY_MODE = 0xf08;

using ModeByPredicate_t = uint8_t *(void *modes, void *const *predicate);
using SetMode_t = void(void *modes, void *mode);

// A new Grand Prix weekend, as the dev session's Apply (0x141125e80) builds one, on the Grand Prix mode entity E
// (KB build: re/db/subsystems/session.md, re/new-gp): the game setup mode from E's list (vt+0x1d0) by name, the
// setter 0x14079f300, the entrant list by name (EntrantsSystem [game manager+0x290] +0x560), one round at a
// track found by name in E's track system (+0xf58), the season built by 0x141207290 on a stub, E's components
// attached (vt+0xe8) and the entrants filled from the list (0x142460230).
constexpr uintptr_t RVA_SET_SETUP_MODE = 0x79f300, RVA_LIST_STAT_ID = 0x24286e0, RVA_ROUNDS_CLEAR = 0x420cd60,
                    RVA_ADD_TRACK = 0xb99450, RVA_BUILD_SEASON = 0x1207290, RVA_FILL_ENTRANTS = 0x2460230;
// A session request without a box or track gets the stock 2025 grid at Albert Park; it never continues a saved Grand
// Prix, whose weekend may be one the session load can't take.
constexpr const char *DEFAULT_BOX = "f1_2025", *DEFAULT_TRACK = "melbourne";
constexpr uintptr_t MANAGER_ENTRANTS = 0x290, ES_LISTS = 0x560, ENTITY_TRACKS = 0xf58,
                    ENTITY_LIST = 0xe18, ENTITY_ENTRANTS = 0x2e0, ENTRANT_SIZE = 0x88;

struct Game {
    uintptr_t manager = 0, set_mode = 0, by_predicate = 0, gp_predicate = 0;
} game;

// The game modes context SetCurrentGameMode takes, as the game passes it (it sets the mode once at boot).
safetyhook::InlineHook h_set_mode;
std::atomic<void *> modes_seen{nullptr};

void set_mode_detour(void *modes, void *mode)
{
    if (modes && !modes_seen.exchange(modes))
        swofty::log("[direct] the game modes context is %p (SetCurrentGameMode, mode %p)", modes, mode);
    h_set_mode.call<void>(modes, mode);
}

constexpr size_t STATE_HASH = 0xb0, STATE_CHILD = 0xbd8, LINK_SLOT = 0x40;

std::mutex lock;
ds::Driver driver;
json::Value request;
safetyhook::MidHook h_link;
std::atomic<bool> armed{false};
std::atomic<int> session_frames{0};
// Read by the cover from the present thread, which holds the cover's lock; the flow thread calls the cover's step()
// under this file's lock, so the cover side never takes it.
std::atomic<bool> start_failed{false};
std::atomic<uint64_t> session_at{0};
std::atomic<bool> session_screen{false};
// The broadcast intro runs under the cover; it comes off at the garage (or the grid), or this long after StateSession
// started updating when neither screen opens.
constexpr uint64_t SESSION_SCREEN_WAIT_MS = 60000;
ds::Phase logged = ds::Phase::off;

void note_phase()
{
    if (driver.phase == logged)
        return;
    logged = driver.phase;
    if (driver.phase == ds::Phase::failed)
        start_failed = true;
    if (driver.phase == ds::Phase::failed)
        swofty::log("[direct] %s start failed: %s; the menu opens as usual", driver.target.name.c_str(),
                    driver.why.c_str());
    else
        swofty::log("[direct] %s start: %s", driver.target.name.c_str(), ds::phase_name(driver.phase));
}

constexpr size_t STATE_STATUS = 0xb8, STATE_CHILDREN = 0xc0, STATE_LINKS = 0xaf0, STATE_LINK_COUNT = 0xb30,
                 STATE_PARENT = 0xc10;
constexpr uint32_t STATE_APP = 0xe11edb87, STATE_SESSION = 0x1505f42a, STATE_LOAD_SESSION = 0x17e3cd0a;
constexpr size_t LOAD_SYNC = 0xcd0;
constexpr uint64_t LOAD_EXIT_WAIT_MS = 500;

void choose_session(void *modes, const char *when);
template <class T> bool rd(const void *p, size_t off, T &out);

// The loading screen's activation (Game_LoadingScreenVM_OnScreenActivated 0x141ca43e0) releases the fade reason
// FrontEnd (bit 1) on the fade owner [[g_Game_Root]+0xa0]: mask +0x7b8, and with no reason left the fade in (state
// +0x7bc = 3, +0x7cc = 0, curve +0x7c4). A direct start never shows that screen, so the session would stay black
// under its HUD: once StateSession updates, the core releases FrontEnd itself.
constexpr uintptr_t RVA_GAME_ROOT = 0x601db08;
constexpr uint32_t FADE_FRONTEND = 1u << 1;
bool fade_released = false;

void release_frontend_fade()
{
    auto *root = *reinterpret_cast<uint8_t **>(swofty::api->module_base() + RVA_GAME_ROOT);
    uint8_t *owner = nullptr;
    if (!root || !rd(root, 0xa0, owner) || !owner || !runtime::readable(owner + 0x7b8, 0x18))
        return;
    auto &mask = *reinterpret_cast<uint32_t *>(owner + 0x7b8);
    uint32_t was = mask, state = *reinterpret_cast<uint32_t *>(owner + 0x7bc);
    fade_released = true;
    if (!(was & FADE_FRONTEND)) {
        swofty::log("[direct] fade owner %p: mask %08x, state %u; FrontEnd is not held", owner, was, state);
        return;
    }
    mask = was & ~FADE_FRONTEND;
    if (mask == 0) {
        *reinterpret_cast<uint32_t *>(owner + 0x7bc) = 3;
        owner[0x7cc] = 0;
    }
    *reinterpret_cast<uint32_t *>(owner + 0x7c4) = 0xd47eb226;
    swofty::log("[direct] fade owner %p: mask %08x -> %08x (FrontEnd released), state %u -> %u", owner, was, mask, state,
                *reinterpret_cast<uint32_t *>(owner + 0x7bc));
}

// The fade owner's reasons and state as the session's intro runs, logged when they change. A race opens no screen
// of its own: its broadcast starts with the first fade in after the intro video (BroadcastFull released, state 3),
// and the cover comes off there. The grid follows the broadcast's grid rundown, two minutes or more later.
bool race_session = false;

void note_fade()
{
    static uint32_t last_mask = ~0u, last_state = ~0u;
    static int told = 0;
    auto *root = *reinterpret_cast<uint8_t **>(swofty::api->module_base() + RVA_GAME_ROOT);
    uint8_t *owner = nullptr;
    if (told >= 40 || !root || !rd(root, 0xa0, owner) || !owner || !runtime::readable(owner + 0x7b8, 8))
        return;
    uint32_t mask = *reinterpret_cast<uint32_t *>(owner + 0x7b8), state = *reinterpret_cast<uint32_t *>(owner + 0x7bc);
    if (mask == last_mask && state == last_state)
        return;
    bool faded_in = last_mask != ~0u && last_mask != 0 && mask == 0 && state == 3;
    last_mask = mask;
    last_state = state;
    told++;
    uint64_t ms = GetTickCount64() - session_at.load();
    swofty::log("[direct] fade owner: mask %08x, state %u, %llu ms into the session", mask, state, (unsigned long long)ms);
    if (race_session && faded_in && !session_screen.exchange(true))
        swofty::log("[direct] the race's broadcast is up after %llu ms; the cover comes off", (unsigned long long)ms);
}

uint8_t *flow_root = nullptr;
uint64_t load_exiting_at = 0;

// The front end's main scene (the menus' dark backdrop with red light streaks) is despawned by StateFrontend's
// exit (Game_StateFrontend_OnStateExit 0x1436d3b00: Game_FrontendSceneManager_DespawnMainScene 0x1428d9a90 on
// [state+0xd20] when [[state+0xd20]+0x40]+0x1048 or +0x1049 is set). A direct start never enters StateFrontend, so
// the backdrop stays under the session: a Time Trial's garage shows it instead of the 3D view, and a Grand Prix's
// broadcast intro cards sit on it. The node exists in the flow graph all the same (FrontendWorld's child
// 0x957880c6, its +0xd20 set by its init), so the core despawns the scene through its manager, once.
constexpr uint32_t STATE_FRONTEND = 0x957880c6;
constexpr uintptr_t FRONTEND_SCENES = 0xd20, SCENES_MAIN = 0x40, MAIN_SPAWNED = 0x1048, RVA_DESPAWN_MAIN_SCENE = 0x28d9a90;
std::atomic<bool> frontend_scene_despawned{false};

uint8_t *flow_node(uint32_t want)
{
    std::vector<uint8_t *> todo{flow_root};
    for (size_t i = 0; i < todo.size() && i < 1024; ++i) {
        uint8_t *n = todo[i];
        if (!n || !runtime::readable(n, 0xc8))
            continue;
        if (*reinterpret_cast<uint32_t *>(n + 0xb0) == want)
            return n;
        auto *cv = *reinterpret_cast<uint8_t **>(n + 0xc0);
        if (!cv || !runtime::readable(cv, 0x18))
            continue;
        for (auto **p = *reinterpret_cast<uint8_t ***>(cv + 8), **e = *reinterpret_cast<uint8_t ***>(cv + 0x10);
             p && p < e && runtime::readable(p, 8); ++p)
            todo.push_back(*p);
    }
    return nullptr;
}

bool frontend_main_scene(uint8_t *&mgr)
{
    uint8_t *node = flow_node(STATE_FRONTEND);
    mgr = node && runtime::readable(node, FRONTEND_SCENES + 8) ? *reinterpret_cast<uint8_t **>(node + FRONTEND_SCENES)
                                                                : nullptr;
    auto *scenes = mgr && runtime::readable(mgr, SCENES_MAIN + 8) ? *reinterpret_cast<uint8_t **>(mgr + SCENES_MAIN)
                                                                  : nullptr;
    return scenes && runtime::readable(scenes, MAIN_SPAWNED + 2) && (scenes[MAIN_SPAWNED] || scenes[MAIN_SPAWNED + 1]);
}

void despawn_frontend_scene(const char *when)
{
    if (frontend_scene_despawned.load())
        return;
    uint8_t *mgr = nullptr;
    bool spawned = frontend_main_scene(mgr);
    if (!mgr) {
        swofty::log("[direct] no StateFrontend node or front end scene manager %s", when);
        return;
    }
    if (!spawned) {
        swofty::log("[direct] the front end's main scene is not spawned %s (manager %p)", when, mgr);
        return;
    }
    frontend_scene_despawned = true;
    reinterpret_cast<void (*)(void *)>(swofty::api->module_base() + RVA_DESPAWN_MAIN_SCENE)(mgr);
    swofty::log("[direct] despawned the front end's main scene %s (manager %p)", when, mgr);
}
bool load_sync_done = false;
int load_sync_tries = 0;

// StateLoadSession's exit (0x1436e0e40) waits for its sync point (+0xcd0) once the load is done: the loading
// screen's UI sets it as it hands over to the session. A direct start never opened that screen, so once the
// state has been exiting for LOAD_EXIT_WAIT_MS the core sets both nodes itself.
void load_exit(uint8_t *state)
{
    if (*reinterpret_cast<uint8_t *>(state + STATE_STATUS) != 3) {
        load_exiting_at = 0;
        return;
    }
    uint64_t now = GetTickCount64();
    if (!load_exiting_at)
        load_exiting_at = now;
    if (load_sync_done || load_sync_tries >= 10 || now - load_exiting_at < LOAD_EXIT_WAIT_MS * (load_sync_tries + 1))
        return;
    load_sync_tries++;
    bool done = runtime::title::complete_sync(state + LOAD_SYNC, "StateLoadSession's sync point (+0xcd0)");
    load_sync_done = done;
    if (done)
        runtime::room_cover::step("Starting the session", 0.85f, 0.9f);
    swofty::log("[direct] StateLoadSession exiting for %llu ms: its sync point is %s",
                (unsigned long long)(now - load_exiting_at), done ? "complete" : "still not complete");
}
uint32_t seen_states[64], seen_links[64];
size_t seen_count = 0;
uint64_t chain_at = 0;
std::string chain_last;

std::string rva(const void *p)
{
    return runtime::code_where(reinterpret_cast<uintptr_t>(p));
}

void describe_state(uint8_t *state, uint32_t hash, uint32_t link)
{
    for (size_t i = 0; i < seen_count; i++)
        if (seen_states[i] == hash) {
            if (seen_links[i] != link) {
                swofty::log("[direct] state %08x returns link %08x (was %08x)", hash, link, seen_links[i]);
                seen_links[i] = link;
            }
            return;
        }
    if (seen_count < 64) {
        seen_states[seen_count] = hash;
        seen_links[seen_count++] = link;
    }
    auto **vt = *reinterpret_cast<void ***>(state);
    std::string links;
    uint32_t n = *reinterpret_cast<uint32_t *>(state + STATE_LINK_COUNT);
    for (uint32_t i = 0; i < n && i < 16; i++) {
        char b[24];
        snprintf(b, sizeof b, " %08x>%08x", reinterpret_cast<uint32_t *>(state + STATE_LINKS)[i],
                 *reinterpret_cast<uint32_t *>(state + 0xf0 + size_t(i) * 0xa0 + 0x98));
        links += b;
    }
    std::string kids;
    auto *cv = *reinterpret_cast<uint8_t **>(state + STATE_CHILDREN);
    if (cv && runtime::readable(cv, 0x18))
        for (auto **p = *reinterpret_cast<uint8_t ***>(cv + 8), **e = *reinterpret_cast<uint8_t ***>(cv + 0x10);
             p && p < e && kids.size() < 400; ++p)
            if (*p && runtime::readable(*p, STATE_HASH + 4)) {
                char b[12];
                snprintf(b, sizeof b, " %08x", *reinterpret_cast<uint32_t *>(*p + STATE_HASH));
                kids += b;
            }
    auto *parent = *reinterpret_cast<uint8_t **>(state + STATE_PARENT);
    uint32_t ph = parent && runtime::readable(parent, STATE_HASH + 4) ? *reinterpret_cast<uint32_t *>(parent + STATE_HASH) : 0;
    std::string type = "?";
    auto *col = runtime::readable(vt - 1, 8) ? static_cast<uint8_t *>(vt[-1]) : nullptr;
    if (col && runtime::readable(col, 0x18)) {
        auto *td = reinterpret_cast<const char *>(swofty::api->module_base() + *reinterpret_cast<uint32_t *>(col + 0xc));
        if (runtime::readable(td + 0x10, 64))
            type.assign(td + 0x10, strnlen(td + 0x10, 63));
    }
    swofty::log("[direct] flow node %08x (%p, parent %08x, type %s): vt %s enter %s update %s exit %s; link now %08x; "
                "links%s; children%s",
                hash, state, ph, type.c_str(), rva(vt).c_str(), rva(vt[8]).c_str(), rva(vt[9]).c_str(),
                rva(vt[10]).c_str(), link, links.c_str(), kids.c_str());
}

void log_chain()
{
    if (!flow_root)
        return;
    std::string chain;
    uint8_t *s = flow_root;
    for (int depth = 0; s && depth < 24 && runtime::readable(s, STATE_PARENT + 8); depth++) {
        char b[32];
        snprintf(b, sizeof b, "%s%08x:%u", depth ? " > " : "", *reinterpret_cast<uint32_t *>(s + STATE_HASH),
                 unsigned(*reinterpret_cast<uint8_t *>(s + STATE_STATUS)));
        chain += b;
        s = *reinterpret_cast<uint8_t **>(s + STATE_CHILD);
    }
    if (chain == chain_last)
        return;
    chain_last = chain;
    swofty::log("[direct] flow chain: %s", chain.c_str());
}

void link_mid(safetyhook::Context &c)
{
    if (!armed.load())
        return;
    auto *state = reinterpret_cast<uint8_t *>(c.rdi);
    if (!state || !runtime::readable(state, STATE_PARENT + 8))
        return;
    auto *slot = reinterpret_cast<uint32_t *>(c.rsp + LINK_SLOT);
    uint32_t hash = *reinterpret_cast<uint32_t *>(state + STATE_HASH);
    bool child = *reinterpret_cast<void **>(state + STATE_CHILD) != nullptr;
    std::lock_guard<std::mutex> g(lock);
    if (hash == STATE_APP)
        flow_root = state;
    if (hash == STATE_SESSION && *reinterpret_cast<uint8_t *>(state + STATE_STATUS) == 2 &&
        session_frames.fetch_add(1) == 0 && driver.target.name == "session" && !fade_released) {
        session_at = GetTickCount64();
        release_frontend_fade();
        if (request["mode"].str() != "time_trial")
            despawn_frontend_scene("as the Grand Prix session starts");
        int id = testagent::session_id_of(request["session"].str());
        bool race = id >= 15 && id <= 17;
        race_session = race && request["mode"].str() != "time_trial";
        runtime::room_cover::step(race ? "Lining up on the grid" : "Opening the garage", 0.9f, 0.97f);
    }
    if (session_at.load())
        note_fade();
    uint32_t link = *slot, out = driver.on_update(hash, link, child);
    if (out != link) {
        *slot = out;
        if (out != ds::LINK_STAY)
            swofty::log("[direct] state %08x: link %08x replaced by %08x", hash, link, out);
    }
    if (driver.phase == ds::Phase::left) {
        describe_state(state, hash, out);
        if (flow_root && driver.target.name == "session")
            for (uint8_t *n = flow_root; n && runtime::readable(n, STATE_PARENT + 8);
                 n = *reinterpret_cast<uint8_t **>(n + STATE_CHILD))
                if (*reinterpret_cast<uint32_t *>(n + STATE_HASH) == STATE_LOAD_SESSION) {
                    static bool told = false;
                    if (!told && driver.target.name == "session") {
                        told = true;
                        std::string track = request["track"].str();
                        static std::string what;
                        what = track.empty() ? "Loading the session" : "Loading " + track;
                        runtime::room_cover::step(what.c_str(), 0.58f, 0.82f);
                        if (request["mode"].str() != "time_trial")
                            choose_session(modes_seen.load(), "load");
                    }
                    load_exit(n);
                    break;
                }
        uint64_t now = GetTickCount64();
        if (now - chain_at >= 1000) {
            chain_at = now;
            log_chain();
        }
    }
    note_phase();
}

void *registry_call(uint8_t *entity, size_t slot, const void *a2, int a3)
{
    auto *reg = entity + ENTITY_REGISTRY;
    auto **vt = *reinterpret_cast<void ***>(reg);
    return reinterpret_cast<void *(*)(void *, const void *, int)>(vt[slot / 8])(reg, a2, a3);
}

runtime::DirectStartSetup room_setup = nullptr, time_trial_setup = nullptr;
void (*time_trial_screen)(const char *name) = nullptr;

// The race's laps: Game_ChampionshipGameMode_BuildSessionSettings (0x1407b7b90, the championship modes' slot) fills
// the session's SessionSettings (sessionId +4) and stores the laps at +0xdf0 XOR +0xdf8; for a race session the core
// writes the requested laps over it after the game's own value.
safetyhook::InlineHook h_settings;

uint64_t settings_detour(void *mode, uint8_t *settings)
{
    uint64_t r = h_settings.call<uint64_t>(mode, settings);
    if (!armed.load() || !settings)
        return r;
    long long laps = request["laps"].num(0);
    uint32_t id = *reinterpret_cast<uint32_t *>(settings + 4);
    if (laps > 0 && laps <= 100 && id >= 15 && id <= 17 && request["mode"].str() != "time_trial") {
        auto &stored = *reinterpret_cast<uint32_t *>(settings + 0xdf0);
        uint32_t key = *reinterpret_cast<uint32_t *>(settings + 0xdf8), was = stored ^ key;
        stored = uint32_t(laps) ^ key;
        swofty::log("[direct] race laps %u -> %lld (session %s)", was, laps, testagent::SESSION_NAMES[id]);
    }
    return r;
}
std::string request_text;
std::atomic<bool> setup_ran{false};
runtime::DirectStartContext context;

void fill_context()
{
    if (!game.manager)
        return;
    void *mgr = *reinterpret_cast<void **>(game.manager);
    context.game_manager = mgr;
    void *modes = modes_seen.load();
    if (!modes && mgr && runtime::readable(mgr, MANAGER_MODES + 8))
        modes = *reinterpret_cast<void **>(static_cast<uint8_t *>(mgr) + MANAGER_MODES);
    context.game_modes = modes;
}

template <class F> F *at_rva(uintptr_t rva)
{
    return reinterpret_cast<F *>(swofty::api->module_base() + rva);
}

uint8_t *grand_prix_entity(void *modes)
{
    void *pred = reinterpret_cast<void *>(game.gp_predicate);
    uint8_t *gp = reinterpret_cast<ModeByPredicate_t *>(game.by_predicate)(modes, &pred);
    return gp && runtime::readable(gp, ENTITY_TRACKS + 8) ? gp : nullptr;
}

// The weekend the session load reads: the current mode's championship holder ([modes+0x88]) > its entity (+0x260)
// > the current championship (+0x1058) > seasons[season_index] (+0x548, +0x560, 16-byte entries, season at +8) >
// weekends[current_weekend] (+0x288, +0x2a0) (apply/moments.cpp reads the same chain for the session facts).
constexpr size_t WK_SESSIONS = 0x220, WK_CURRENT = 0x238, WK_TRACK = 0x278, SESSION_STRIDE = 0xa0, SESSION_ID = 0x8;

template <class T> bool rd(const void *p, size_t off, T &out)
{
    auto *a = static_cast<const uint8_t *>(p) + off;
    if (!p || !runtime::readable(a, sizeof(T)))
        return false;
    out = *reinterpret_cast<const T *>(a);
    return true;
}

uint8_t *current_weekend(void *modes)
{
    uint8_t *holder = nullptr, *entity = nullptr, *champ = nullptr, *sb = nullptr, *se = nullptr, *season = nullptr,
            **wb = nullptr, **we = nullptr, *wk = nullptr;
    uint32_t si = 0, wi = 0;
    if (!rd(modes, 0x88, holder) || !rd(holder, 0x260, entity) || !rd(entity, 0x1058, champ) ||
        !rd(champ, 0x548, sb) || !rd(champ, 0x550, se) || !rd(champ, 0x560, si) || se <= sb ||
        si >= size_t(se - sb) / 16 || !rd(sb, size_t(si) * 16 + 8, season) || !rd(season, 0x288, wb) ||
        !rd(season, 0x290, we) || !rd(season, 0x2a0, wi) || we <= wb || wi >= size_t(we - wb) ||
        !rd(wb, size_t(wi) * 8, wk) || !runtime::readable(wk, 0x280))
        return nullptr;
    return wk;
}

std::string weekend_text(uint8_t *wk)
{
    auto *b = *reinterpret_cast<uint8_t **>(wk + WK_SESSIONS), *e = *reinterpret_cast<uint8_t **>(wk + WK_SESSIONS + 8);
    uint32_t cur = *reinterpret_cast<uint32_t *>(wk + WK_CURRENT);
    std::string ids;
    for (auto *p = b; p && p < e && runtime::readable(p, SESSION_STRIDE); p += SESSION_STRIDE) {
        uint32_t id = *reinterpret_cast<uint32_t *>(p + SESSION_ID);
        ids += (ids.empty() ? "" : " ") + std::string(id < 19 ? testagent::SESSION_NAMES[id] : "?");
    }
    char head[96];
    snprintf(head, sizeof head, "weekend %p, track stat id %u, current session %u of: ", wk,
             unsigned(wk[WK_TRACK]), cur);
    return head + ids;
}

// The requested session (the CLI's session=, a game session name or practice1..3, qualifying, race) made the
// weekend's current one. A Grand Prix the profile already holds keeps its season (the season build only runs for a
// mode with no season), so the weekend is the saved one: its first session becomes the requested one and the
// weekend starts there. Moving the current index past a session instead leaves the race without the results it
// builds the grid from (the AI's team performance build faulted on a race behind an unplayed qualifying).
void choose_session(void *modes, const char *when)
{
    uint8_t *wk = current_weekend(modes);
    if (!wk) {
        swofty::log("[direct] Grand Prix (%s): no current weekend to read", when);
        return;
    }
    swofty::log("[direct] Grand Prix (%s): %s", when, weekend_text(wk).c_str());
    std::string name = request["session"].str();
    if (name.empty())
        return;
    int want = testagent::session_id_of(name);
    if (want < 0 || want == testagent::SESSION_TIME_TRIAL) {
        swofty::log("[direct] Grand Prix: session '%s' is not a Grand Prix session; the weekend's own is kept", name.c_str());
        return;
    }
    auto *b = *reinterpret_cast<uint8_t **>(wk + WK_SESSIONS), *e = *reinterpret_cast<uint8_t **>(wk + WK_SESSIONS + 8);
    auto &cur = *reinterpret_cast<uint32_t *>(wk + WK_CURRENT);
    uint32_t n = b && e > b ? uint32_t(size_t(e - b) / SESSION_STRIDE) : 0;
    if (!n)
        return;
    if (cur != 0) {
        swofty::log("[direct] Grand Prix: current session %u -> 0", cur);
        cur = 0;
    }
    auto &id = *reinterpret_cast<uint32_t *>(b + SESSION_ID);
    if (id != uint32_t(want)) {
        swofty::log("[direct] Grand Prix: session 0 %s -> %s", id < 19 ? testagent::SESSION_NAMES[id] : "?",
                    testagent::SESSION_NAMES[want]);
        id = uint32_t(want);
    }
}

// The weekend's sessions as the dev session's Apply (0x141125e80) builds them: Sess_Build_DevWeekendStructure
// (0x140b025f0) on a 0x38-byte list {+8, +0x10 the allocator, +0x18..+0x20 16-byte entries} with the game setup mode
// ([E+0x2b0]) and a (practice, qualifying, race) triple from the session name, handed to the weekend service
// (registry component 0x14605ada0) slot 0x268; then the entries are released and the list freed.
constexpr uintptr_t RVA_WEEKEND_STRUCTURE = 0xb025f0, RVA_WEEKEND_SERVICE = 0x605ada0;

struct Triple {
    uint64_t practice, qualifying, race;
};

bool session_triple(int id, Triple &t)
{
    if (id == 4)
        t = {2, 0, 3};
    else if (id >= 1 && id <= 3)
        t = {6, 0, 3};
    else if (id == 5 || id == 6 || id == 7)
        t = {0, 4, 3};
    else if (id == 8)
        t = {0, 3, 3};
    else if (id == 9)
        t = {0, 1, 3};
    else if (id >= 10 && id <= 14)
        t = {0, 3, 3};
    else if (id >= 15 && id <= 17)
        t = {0, 0, 7};
    else
        return false;
    return true;
}

void weekend_structure(uint8_t *e, uint8_t *component)
{
    std::string name = request["session"].str();
    if (name.empty())
        return;
    int id = testagent::session_id_of(name);
    if (name == "practice")
        id = 4;
    Triple t{};
    if (!session_triple(id, t)) {
        swofty::log("[direct] Grand Prix: no weekend structure for session '%s'", name.c_str());
        return;
    }
    void *alloc = *reinterpret_cast<void **>(component + 0x210), *gsm = *reinterpret_cast<void **>(e + 0x2b0);
    void *svc = registry_call(e, 0x10, reinterpret_cast<void *>(swofty::api->module_base() + RVA_WEEKEND_SERVICE), 1);
    if (!alloc || !gsm || !svc) {
        swofty::log("[direct] Grand Prix: the weekend structure is not set (allocator %p, setup mode %p, service %p)",
                    alloc, gsm, svc);
        return;
    }
    alignas(16) uint8_t list[0x38] = {};
    *reinterpret_cast<void **>(list + 8) = alloc;
    *reinterpret_cast<void **>(list + 0x10) = alloc;
    at_rva<void(void *, void *, uint64_t, uint64_t, uint64_t, uint64_t, uint64_t, uint64_t, uint64_t)>(
        RVA_WEEKEND_STRUCTURE)(list, gsm, t.practice, t.qualifying, t.race, 0, 1, 0, 0);
    auto **svt = *reinterpret_cast<void ***>(svc);
    reinterpret_cast<void (*)(void *, void *, int)>(svt[0x268 / 8])(svc, list, 1);
    auto *b = *reinterpret_cast<uint8_t **>(list + 0x18), *end = *reinterpret_cast<uint8_t **>(list + 0x20);
    size_t n = 0;
    for (auto *it = b; it && it < end; it += 0x10, n++) {
        auto *o = *reinterpret_cast<uint8_t **>(it + 8);
        if (!o)
            continue;
        auto **ovt = *reinterpret_cast<void ***>(o);
        if (InterlockedDecrement(reinterpret_cast<volatile LONG *>(o + 8)) == 0) {
            reinterpret_cast<void (*)(void *)>(ovt[0])(o);
            if (InterlockedDecrement(reinterpret_cast<volatile LONG *>(o + 0xc)) == 0)
                reinterpret_cast<void (*)(void *)>(ovt[1])(o);
        }
    }
    if (b)
        reinterpret_cast<void (*)(void *, void *)>((*reinterpret_cast<void ***>(alloc))[0x38 / 8])(alloc, b);
    swofty::log("[direct] Grand Prix: weekend structure for %s: practice %llu, qualifying %llu, race %llu (%zu action(s))",
                name.c_str(), (unsigned long long)t.practice, (unsigned long long)t.qualifying,
                (unsigned long long)t.race, n);
}

// The player's seat, as Apply's drivers config (0x141129850) sets it: the first entrant of the requested team (and
// driver) that is not human gets the player's identity block (the profile [[[game manager+0x140]+0x1c0]+0x1c0]
// slot 0x38, 0x40 bytes to +8) and the human flag (+0x48 = 0x10000000), between the entrants-changing messages.
constexpr uintptr_t RVA_MESSAGE_BUS = 0x618b3b8, RVA_POST_BEGIN = 0x24e2b90, RVA_POST_END = 0x24e2f00,
                    VT_MSG_BEGIN = 0x578d4e8, VT_MSG_END = 0x578d510;

std::map<uint32_t, std::string> labels_of(const char *table, const char *stem)
{
    std::map<uint32_t, std::string> out;
    const SwoftyDb *db = runtime::db_table();
    if (!db || !db->available || !db->available())
        return out;
    int64_t count = 0;
    std::string t = table;
    if (db->int_at((t + "._" + stem + "_instancecount_").c_str(), &count) != SWOFTY_ASSET_OK || count <= 0 || count > 8192)
        return out;
    for (int64_t i = 0; i < count; i++) {
        std::string row = t + "." + stem + "_" + std::to_string(i);
        int64_t id = -1;
        const char *label = nullptr;
        if (db->int_at((row + ".stat_id").c_str(), &id) == SWOFTY_ASSET_OK && id >= 0 && id < 0x3fff &&
            db->string_at((row + "._metadata_").c_str(), &label) == SWOFTY_ASSET_OK && label)
            out[uint32_t(id)] = std::string(label, strnlen(label, 128));
    }
    return out;
}

bool label_matches(const std::string &label, const std::string &want)
{
    if (want.empty())
        return true;
    if (label == want)
        return true;
    std::string l = label, w = want;
    for (auto &c : l)
        c = char(tolower(uint8_t(c)));
    for (auto &c : w)
        c = char(tolower(uint8_t(c)));
    return l.find(w) != std::string::npos;
}

void seat_player(uint8_t *e, void *mgr)
{
    std::string team = request["team"].str(), driver = request["driver"].str();
    auto **begin = reinterpret_cast<uint8_t **>(e + ENTITY_ENTRANTS);
    size_t n = size_t(begin[1] - begin[0]) / ENTRANT_SIZE;
    if (!n)
        return;
    auto teams = labels_of("entrants.teams", "team"), drivers = labels_of("entrants.drivers", "driver");
    auto team_of = [&](uint8_t *r) -> std::string {
        if (r[0x58])
            return "?";
        auto it = teams.find(*reinterpret_cast<uint16_t *>(r + 0x5a));
        return it == teams.end() ? "#" + std::to_string(*reinterpret_cast<uint16_t *>(r + 0x5a)) : it->second;
    };
    auto driver_of = [&](uint8_t *r) -> std::string {
        if (r[0x50])
            return "?";
        auto it = drivers.find(*reinterpret_cast<uint16_t *>(r + 0x52));
        return it == drivers.end() ? "#" + std::to_string(*reinterpret_cast<uint16_t *>(r + 0x52)) : it->second;
    };
    uint8_t *seat = nullptr;
    size_t humans = 0;
    std::string grid;
    for (size_t i = 0; i < n; i++) {
        uint8_t *r = begin[0] + i * ENTRANT_SIZE;
        bool human = (r[0x4b] & 0x10) != 0;
        humans += human;
        if (grid.size() < 1500)
            grid += (grid.empty() ? "" : ", ") + team_of(r) + "/" + driver_of(r) + (human ? " (human)" : "");
        if (!seat && !human && label_matches(team_of(r), team) && label_matches(driver_of(r), driver))
            seat = r;
    }
    swofty::log("[direct] Grand Prix: %zu entrant(s), %zu human: %s", n, humans, grid.c_str());
    if (!seat) {
        if (!team.empty() || !driver.empty())
            swofty::log("[direct] Grand Prix: no entrant of team '%s' driver '%s'; the player takes the first seat",
                        team.c_str(), driver.c_str());
        seat = begin[0];
        if (seat[0x4b] & 0x10) {
            swofty::log("[direct] Grand Prix: the first seat is already the player's (%s/%s)", team_of(seat).c_str(),
                        driver_of(seat).c_str());
            return;
        }
    }
    alignas(16) uint8_t id[0x40] = {};
    void *users = mgr ? *reinterpret_cast<void **>(static_cast<uint8_t *>(mgr) + 0x140) : nullptr;
    void *user = users && runtime::readable(users, 0x1c8) ? *reinterpret_cast<void **>(static_cast<uint8_t *>(users) + 0x1c0) : nullptr;
    void *prof = user && runtime::readable(user, 0x1c8) ? *reinterpret_cast<void **>(static_cast<uint8_t *>(user) + 0x1c0) : nullptr;
    if (!prof) {
        swofty::log("[direct] Grand Prix: no player profile for the identity block; the seat is not set");
        return;
    }
    reinterpret_cast<void (*)(void *, void *)>((*reinterpret_cast<void ***>(prof))[0x38 / 8])(prof, id);
    uintptr_t base = swofty::api->module_base();
    void *bus = *reinterpret_cast<void **>(base + RVA_MESSAGE_BUS);
    void *mb = bus && runtime::readable(bus, 0x10) ? *reinterpret_cast<void **>(static_cast<uint8_t *>(bus) + 8) : nullptr;
    alignas(16) void *msg[3] = {reinterpret_cast<void *>(base + VT_MSG_BEGIN), nullptr, nullptr};
    if (mb)
        at_rva<void(void *, void *)>(RVA_POST_BEGIN)(mb, msg);
    e[0x2f8] = 1;
    for (size_t i = 0; i < n; i++) {
        uint8_t *r = begin[0] + i * ENTRANT_SIZE;
        if (r != seat && (r[0x4b] & 0x10)) {
            *reinterpret_cast<uint32_t *>(r + 0x48) &= ~0x10000000u;
            swofty::log("[direct] Grand Prix: %s/%s is no longer the player's", team_of(r).c_str(), driver_of(r).c_str());
        }
    }
    std::memcpy(seat + 8, id, 0x40);
    *reinterpret_cast<uint32_t *>(seat + 0x48) = 0x10000000;
    msg[0] = reinterpret_cast<void *>(base + VT_MSG_END);
    msg[1] = nullptr;
    msg[2] = e + 0x2d8;
    if (mb)
        at_rva<void(void *, void *)>(RVA_POST_END)(mb, msg);
    e[0x2f8] = 0;
    swofty::log("[direct] Grand Prix: the player drives for %s as %s (seat %zu)", team_of(seat).c_str(),
                driver_of(seat).c_str(), size_t(seat - begin[0]) / ENTRANT_SIZE);
}

ds::Setup new_grand_prix(std::string &why)
{
    if (!game.manager || !game.set_mode || !game.by_predicate || !game.gp_predicate) {
        why = "the game manager, SetCurrentGameMode or the Grand Prix mode lookup did not resolve";
        return ds::Setup::failed;
    }
    void *mgr = *reinterpret_cast<void **>(game.manager);
    void *modes = modes_seen.load();
    if (!mgr || !modes)
        return ds::Setup::pending;
    uint8_t *e = grand_prix_entity(modes);
    if (!e)
        return ds::Setup::pending;
    // The mode attaches its components over the frames after it is made current, so the weekend is built on a
    // later tick, once the rounds component is there (looked up without creating it: 0x1407c1200, mode 0).
    static uint64_t mode_set_at = 0;
    if (!mode_set_at) {
        reinterpret_cast<SetMode_t *>(game.set_mode)(modes, *reinterpret_cast<void **>(e + ENTITY_MODE));
        mode_set_at = GetTickCount64();
        swofty::log("[direct] Grand Prix: the Grand Prix mode is current (entity %p); waiting for its components", e);
        return ds::Setup::pending;
    }
    void *rounds_seen = registry_call(e, 0x10, reinterpret_cast<void *>(swofty::api->module_base() + 0x605ad40), 0);
    if (!rounds_seen) {
        if (GetTickCount64() - mode_set_at < 15000)
            return ds::Setup::pending;
        why = "the Grand Prix mode's rounds component did not appear within 15 s of making the mode current";
        return ds::Setup::failed;
    }
    swofty::log("[direct] Grand Prix: rounds component %p after %llu ms", rounds_seen,
                (unsigned long long)(GetTickCount64() - mode_set_at));
    std::string setup_name = request["setup_mode"].str("swofty_f1"), list_name = request["box"].str(DEFAULT_BOX),
                track_name = request["track"].str(DEFAULT_TRACK);
    // 1. The game setup mode by name, from E's setup modes (vt+0x1d0: a vector at +8..+0x10).
    auto **evt = *reinterpret_cast<void ***>(e);
    auto *modes_vec = reinterpret_cast<uint8_t *(*)(void *)>(evt[0x1d0 / 8])(e);
    uint8_t *gsm = nullptr;
    for (auto **p = *reinterpret_cast<uint8_t ***>(modes_vec + 8), **end = *reinterpret_cast<uint8_t ***>(modes_vec + 0x10);
         p && p != end; ++p) {
        uint8_t *m = *p;
        void *holder = m && runtime::readable(m, 0xe0) ? *reinterpret_cast<void **>(m + 0xd8) : nullptr;
        const char *name = holder ? *reinterpret_cast<const char **>(static_cast<uint8_t *>(holder) + 0x10) : nullptr;
        if (name && setup_name == name) {
            gsm = m;
            break;
        }
    }
    if (!gsm) {
        why = "the Grand Prix mode has no game setup mode '" + setup_name + "'";
        return ds::Setup::failed;
    }
    at_rva<void(void *, void *, int)>(RVA_SET_SETUP_MODE)(e, gsm, 0);
    swofty::log("[direct] Grand Prix: setup mode %s (%p) set on %p", setup_name.c_str(), gsm, e);
    // 2. The entrant list by name: EntrantsSystem +0x560, 16-byte entries, the list at +8.
    uint8_t *es = *reinterpret_cast<uint8_t **>(static_cast<uint8_t *>(mgr) + MANAGER_ENTRANTS);
    uint8_t id = at_rva<uint8_t(const char *)>(RVA_LIST_STAT_ID)(list_name.c_str());
    auto *lists = *reinterpret_cast<uint8_t **>(es + ES_LISTS), *lists_end = *reinterpret_cast<uint8_t **>(es + ES_LISTS + 8);
    if (!lists || size_t(lists_end - lists) / 16 <= id) {
        why = "no entrant list '" + list_name + "' (stat id " + std::to_string(id) + ")";
        return ds::Setup::failed;
    }
    void *list = *reinterpret_cast<void **>(lists + size_t(id) * 16 + 8);
    if (!list) {
        std::string held;
        size_t count = size_t(lists_end - lists) / 16;
        for (size_t i = 0; i < count && held.size() < 1200; i++)
            if (*reinterpret_cast<void **>(lists + i * 16 + 8))
                held += " " + std::to_string(i);
        why = "the entrant list '" + list_name + "' (stat id " + std::to_string(id) + ") is not loaded";
        swofty::log("[direct] Grand Prix: %s; the entrants system holds %zu list slot(s), loaded:%s", why.c_str(), count,
                    held.c_str());
        return ds::Setup::failed;
    }
    *reinterpret_cast<void **>(e + ENTITY_LIST) = list;
    swofty::log("[direct] Grand Prix: entrant list %s (stat id %u) is %p", list_name.c_str(), unsigned(id), list);
    // 3. The track by name in E's track system: 16-byte entries at +0x28..+0x30, the record at +8, its name at +0x78.
    uint8_t *tracks = *reinterpret_cast<uint8_t **>(e + ENTITY_TRACKS);
    uint8_t *track = nullptr;
    if (tracks && runtime::readable(tracks, 0x38))
        for (auto *q = *reinterpret_cast<uint8_t **>(tracks + 0x28), *qend = *reinterpret_cast<uint8_t **>(tracks + 0x30);
             q && q < qend; q += 16) {
            auto *rec = *reinterpret_cast<uint8_t **>(q + 8);
            const char *name = rec && runtime::readable(rec, 0x80) ? *reinterpret_cast<const char **>(rec + 0x78) : nullptr;
            if (name && track_name == name) {
                track = rec;
                break;
            }
        }
    if (!track) {
        why = "the Grand Prix mode's track system has no track '" + track_name + "'";
        return ds::Setup::failed;
    }
    // 4. One round at that track, then the season built from the rounds (0x141207290 on a stub that holds the
    // registry at +0x270, the rounds at +0x290 and the track at [+0x298]+0x18).
    void *rounds = registry_call(e, 0x10, reinterpret_cast<void *>(swofty::api->module_base() + 0x605ad40), 0);
    if (!rounds) {
        why = "the Grand Prix mode has no rounds component";
        return ds::Setup::failed;
    }
    // The clear (0x14420cd60, a jump to runtime-decrypted code) reads its list holder from the second argument:
    // it starts mov rcx,[rdx], and the track select calls it with rdx = rounds+0x298.
    at_rva<void(void *, void *)>(RVA_ROUNDS_CLEAR)(nullptr, static_cast<uint8_t *>(rounds) + 0x298);
    at_rva<void(void *, void *)>(RVA_ADD_TRACK)(rounds, track);
    alignas(16) static uint8_t stub[0x400], holder[0x40];
    std::memset(stub, 0, sizeof stub);
    std::memset(holder, 0, sizeof holder);
    *reinterpret_cast<void **>(stub + 0x270) = e + ENTITY_REGISTRY;
    *reinterpret_cast<void **>(stub + 0x290) = rounds;
    *reinterpret_cast<void **>(holder + 0x18) = track;
    *reinterpret_cast<void **>(stub + 0x298) = holder;
    at_rva<void(void *)>(RVA_BUILD_SEASON)(stub);
    // 5. E's components attached again, as the track select confirm does after its rounds, then the sessions.
    reinterpret_cast<void (*)(void *)>(evt[0xe8 / 8])(e);
    weekend_structure(e, static_cast<uint8_t *>(rounds));
    // 6. The entrants from the list. The fill clears the entrants a saved Grand Prix left first.
    auto **begin = reinterpret_cast<uint8_t **>(e + ENTITY_ENTRANTS);
    size_t before = size_t(begin[1] - begin[0]) / ENTRANT_SIZE;
    {
        alignas(16) static uint8_t blk[0x190];
        std::memset(blk, 0, sizeof blk);
        *reinterpret_cast<uint64_t *>(blk + 0x180) = 1;
        at_rva<void(void *, void *, void *, int)>(RVA_FILL_ENTRANTS)(es, e, blk, 0);
    }
    size_t entrants = size_t(begin[1] - begin[0]) / ENTRANT_SIZE;
    swofty::log("[direct] Grand Prix: entrants %zu before the fill, %zu after", before, entrants);
    seat_player(e, mgr);
    swofty::log("[direct] Grand Prix: new weekend: setup mode %s, list %s (%p), track %s (%p), rounds %p, %zu entrant(s)",
                setup_name.c_str(), list_name.c_str(), list, track_name.c_str(), track, rounds, entrants);
    if (uint8_t *wk = current_weekend(modes)) {
        uint8_t id = track[1];
        if (wk[WK_TRACK] != id) {
            swofty::log("[direct] Grand Prix: the weekend's track stat id %u -> %u (%s)", unsigned(wk[WK_TRACK]),
                        unsigned(id), track_name.c_str());
            wk[WK_TRACK] = id;
        }
    }
    choose_session(modes, "built");
    return ds::Setup::done;
}

ds::Setup set_up_target(std::string &why)
{
    if (driver.target.name == "session" && request["mode"].str() == "time_trial") {
        if (!time_trial_setup) {
            why = "no Time Trial setup is registered (direct_start_tt.cpp)";
            return ds::Setup::failed;
        }
        return time_trial_setup(why);
    }
    if (driver.target.name == "session")
        return new_grand_prix(why);
    if (driver.target.name == "room") {
        if (!room_setup) {
            why = "no room setup is registered (netplay_drive.cpp)";
            return ds::Setup::failed;
        }
        return room_setup(why);
    }
    return ds::Setup::done;
}

}

namespace runtime {

void direct_start_setup()
{
    struct Ran {
        ~Ran() { setup_ran = true; }
    } ran;
    char env[1024] = {};
    DWORD n = GetEnvironmentVariableA("SWOFTY_DIRECT_START", env, sizeof env);
    std::string target, text;
    if (n && n < sizeof env && env[0] == '@')
        text = runtime::read_file(runtime::wide(env + 1), size_t(1) << 16);
    else if (n && n < sizeof env)
        target = env;
    else
        text = runtime::read_file(runtime::swofty_root() + L"direct_start.json", size_t(1) << 16);
    if (target.empty()) {
        if (text.empty())
            return;
        request_text = text;
        if (!json::parse(text, request)) {
            swofty::log("[direct] the direct start request is unusable; the game starts as usual");
            return;
        }
        target = request["target"].str();
    }
    ds::Target t;
    if (target == "session")
        t = ds::session_target();
    else if (target == "room")
        t = ds::room_target();
    else {
        swofty::log("[direct] swofty\\direct_start.json names no known target ('%s'); the game starts as usual",
                    target.c_str());
        return;
    }
    game.manager = runtime::find(SIG_GAME_MANAGER, "game manager", "direct");
    game.set_mode = runtime::find(SIG_SET_MODE, "SetCurrentGameMode", "direct");
    game.by_predicate = runtime::find(SIG_MODE_BY_PREDICATE, "game mode lookup by predicate", "direct");
    game.gp_predicate = runtime::find(SIG_GP_PREDICATE, "Grand Prix mode predicate", "direct");
    if (game.set_mode) {
        auto h = safetyhook::InlineHook::create(reinterpret_cast<void *>(game.set_mode),
                                                reinterpret_cast<void *>(set_mode_detour));
        if (h)
            h_set_mode = std::move(*h);
    }
    if (target == "session" && request["laps"].num(0) > 0) {
        uintptr_t at = runtime::same_build() ? runtime::find(SIG_BUILD_SETTINGS, "session settings", "direct") : 0;
        bool hooked = false;
        if (at) {
            auto h = safetyhook::InlineHook::create(reinterpret_cast<void *>(at), reinterpret_cast<void *>(settings_detour));
            if (h) {
                h_settings = std::move(*h);
                hooked = true;
            }
        }
        if (!hooked)
            swofty::log("[direct] the race laps are not set: Game_ChampionshipGameMode_BuildSessionSettings did not hook");
    }
    runtime::title::setup();
    if (!runtime::title::why().empty()) {
        swofty::log("[direct] off: the title can't pass by itself (%s)", runtime::title::why().c_str());
        return;
    }
    if (t.exit_state || t.from_state) {
        uintptr_t at = runtime::same_build() ? runtime::find(SIG_FLOW_LINK, "flow link", "direct") : 0;
        if (!at) {
            swofty::log("[direct] off: the flow link site is the KB build's and did not resolve on this build");
            return;
        }
        auto h = safetyhook::MidHook::create(reinterpret_cast<void *>(at), link_mid);
        if (!h) {
            swofty::log("[direct] off: the hook on the flow link site failed: %s", runtime::hook_error(h.error()).c_str());
            return;
        }
        h_link = std::move(*h);
    }
    {
        std::lock_guard<std::mutex> g(lock);
        driver.arm(t);
        note_phase();
    }
    runtime::title::want(true);
    armed = true;
}

void log_stack(const char *what)
{
    void *frames[24] = {};
    USHORT n = RtlCaptureStackBackTrace(1, 24, frames, nullptr);
    swofty::log("[direct] %s, thread %lu, from:", what, GetCurrentThreadId());
    for (USHORT i = 0; i < n; i++)
        swofty::log("[direct]   #%u %s", unsigned(i), runtime::code_where(reinterpret_cast<uintptr_t>(frames[i])).c_str());
}

uintptr_t direct_start_set_mode()
{
    return game.set_mode;
}

DirectStartContext direct_start_context()
{
    return context;
}

void direct_start_register_room(DirectStartSetup setup)
{
    room_setup = setup;
}

void direct_start_register_time_trial(DirectStartSetup setup)
{
    time_trial_setup = setup;
}

std::string direct_start_request_text()
{
    return request_text;
}

int direct_start_cover_signal()
{
    if (!setup_ran.load())
        return 0;
    if (!armed.load())
        return -1;
    if (start_failed.load())
        return -1;
    if (session_screen.load())
        return 1;
    uint64_t at = session_at.load();
    return at && GetTickCount64() - at >= SESSION_SCREEN_WAIT_MS ? 1 : 0;
}

void *direct_start_flow_root()
{
    return flow_root;
}

void direct_start_despawn_frontend_scene(const char *when)
{
    despawn_frontend_scene(when);
}

void direct_start_register_time_trial_screen(void (*screen)(const char *name))
{
    time_trial_screen = screen;
}

void direct_start_screen(const char *name)
{
    if (armed.load() && name && time_trial_screen)
        time_trial_screen(name);
    if (!armed.load() || !name || !session_at.load() || session_screen.load())
        return;
    static std::atomic<int> told{0};
    if (told.fetch_add(1) < 40)
        swofty::log("[direct] screen %s opened %llu ms into the session", name,
                    (unsigned long long)(GetTickCount64() - session_at.load()));
    if (!std::strcmp(name, "garage_cockpit") || !std::strcmp(name, "race_hud") || !std::strcmp(name, "grid")) {
        session_screen = true;
        swofty::log("[direct] the session's screen %s is up after %llu ms; the cover comes off", name,
                    (unsigned long long)(GetTickCount64() - session_at.load()));
    }
}

void direct_start_profile_read()
{
    static std::atomic<bool> once{false};
    if (armed.load() && !once.exchange(true))
        log_stack("the first profile file read");
}

void direct_start_profile_loaded(bool ok)
{
    if (!armed.load())
        return;
    static std::atomic<bool> once{false};
    if (!once.exchange(true))
        log_stack("the profile root load");
    std::lock_guard<std::mutex> g(lock);
    if (!ok) {
        swofty::log("[direct] the profile load failed; the start waits for the next one");
        return;
    }
    runtime::room_cover::step("Profile loaded", 0.4f, 0.54f);
    driver.profile_loaded();
    note_phase();
}

void direct_start_tick()
{
    if (!armed.load())
        return;
    std::lock_guard<std::mutex> g(lock);
    if (driver.phase == ds::Phase::setup) {
        fill_context();
        std::string why;
        ds::Setup r = set_up_target(why);
        driver.setup_result(r, why);
        if (r == ds::Setup::done && driver.target.name == "session")
            runtime::room_cover::step("Weekend ready", 0.5f, 0.56f);
    }
    // A session keeps the virtual pad plugged in, idle: the game bound the player to it at the title, and letting
    // go of the claim would stop the session on CONTROLLER DISCONNECTED.
    if (!driver.engage_title() && runtime::title::wanted() && !runtime::test_run() && driver.target.name != "session")
        runtime::title::want(false);
    note_phase();
}

}
