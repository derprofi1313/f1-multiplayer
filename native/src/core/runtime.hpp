#pragma once

#include "direct_start_logic.hpp"
#include "session_lifecycle_logic.hpp"
#include <windows.h>

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <functional>
#include <set>
#include <string>
#include <vector>

#include "json.hpp"
#include "swofty_api.h"

namespace swofty_core::registry {
class Store;
}

namespace runtime {

enum Stage { STAGE_WINDOW = 1 };

struct Sig {
    const char *what, *pattern;
    uintptr_t rva;
    enum Kind { addr, rel32, riprel } kind = addr;
    uintptr_t insn = 0, anchor = 0, len = 0;
};

struct OverlayErp {
    std::string path;
    uint64_t size = 0;
    size_t resources = 0;
    std::vector<std::string> targets;
};

struct Overlay {
    std::vector<OverlayErp> erps;
    size_t replaced = 0, added = 0;
};

const json::Value &config();
const std::string &config_source();
const std::wstring &game_root();
const std::wstring &swofty_root();
const std::wstring &state_root();
bool enabled(const char *feature, const char *what);
std::string load_stage();
bool same_build();

uintptr_t find(const Sig &s, const char *tag, const char *channel = "assets");
bool readable(const void *p, size_t n);
std::string read_file(const std::wstring &path, size_t limit = size_t(512) << 20);
bool write_file(const std::wstring &path, const std::string &data);
bool file_size(const std::wstring &path, uint64_t *size);
std::wstring wide(const std::string &s);
std::string narrow(const std::wstring &w);
std::wstring under_swofty(const std::string &rel);
std::string lower(std::string s);
uint64_t fnv64(const char *s, size_t n);
inline uint64_t fnv64(const std::string &s) { return fnv64(s.data(), s.size()); }
uint32_t fnv32(const char *s);
void md5(const void *data, size_t n, uint8_t out[16]);
std::string hex16(uint64_t v);

void on_stage(void (*fn)(int stage));
void when_code_ready(const char *what, void (*fn)());
bool code_ready();

const Overlay *resource_overlay(DWORD wait_ms);
void overlay_registered(const std::vector<uint64_t> &erp_hashes, size_t groups);
void overlay_dropped(const std::string &why);
std::string erp_name(uint64_t hash);

const SwoftyAssets *assets_table();
const SwoftyEvents *events_table();
const SwoftyOverlay *overlay_table();
const SwoftyTelemetry *telemetry_table();
const SwoftyDb *db_table();
const SwoftyStorage *storage_table();
const SwoftyCareer *career_table();
const SwoftyPopup *popup_table();
const SwoftyLang *lang_table();
const SwoftyCharacters *characters_table();
const SwoftyCareerWizard *wizard_table();
void lang_override_setup();
const json::Value &sdk();
void ready();
void assets_setup();
void resources_setup();
void gamedb_setup();
void dbread_setup();
void overlay_setup();
void events_setup();
void events_drop_module(void *module);
size_t events_fire(const char *event, int64_t a, int64_t b, const void *data);
void session_emit(const session_life::Emit &e);
void session_flow(const char *known, int to);

struct CoreMixins {
    const char *name;
    std::vector<SwoftyMixin> mixins;
};

enum class TileRoute { pending, live, off };

std::vector<CoreMixins> career_mixins();
std::vector<CoreMixins> characters_mixins();
std::vector<CoreMixins> wizard_mixins();
std::vector<CoreMixins> tile_route_mixins();
void characters_commit();
void wizard_commit();
void tile_route_commit();
void tile_route_want();
TileRoute tile_route_state();
void wizard_tile_routed(const std::string &group, void *out);
bool wizard_route_running(const std::string &route, void *name);
bool core_mixins_active(const char *name);
bool core_mixins_closed();
void career_setup();
void career_tick();
int career_block(const void *from, uint32_t mode, const char *title, const char *body);
int career_unblock(const void *from, uint32_t mode);
void career_start_setup();
void career_start_drop_module(void *module);
bool career_start_live();
uint32_t career_start_picked();
void popup_setup();
void popup_tick();
void mfd_setup();
void mfd_tick();
int popup_show(const std::string &title, const std::string &body, const std::string &by);
int hud_toast(const std::string &text, const std::string &detail, float seconds, const std::string &by);
const SwoftyCareerContent *career_content_table();
std::vector<CoreMixins> career_content_mixins();
void career_content_commit();
void career_content_tick();
void hub_tick();
const SwoftyEra *era_table();
const SwoftyRegistry *registry_table();
int registry_write_reports();
swofty_core::registry::Store &older_store();
std::string older_addon(const void *from);
std::vector<CoreMixins> era_mixins();
void era_commit();
void era_session_load();
void era_session_trace(const char *state, int to);
void era_session_unloaded(const char *state, int to);
std::vector<CoreMixins> tt_guard_mixins();
void tt_guard_commit();
std::vector<CoreMixins> career_alias_mixins();
void career_alias_commit();
std::vector<CoreMixins> car_category_mixins();
void car_category_commit();
std::vector<CoreMixins> save_splice_mixins();
void save_splice_commit();
// The text of swofty/settings.json as the launcher wrote it (empty when absent), and the running build's
// distribution as the binding set names it: "steam", "ea" or "unknown".
const std::string &settings_json_text();
std::string build_distribution();
// Career saves (career_saves.cpp): the career index and per-career addon data.
namespace career_saves {
struct SlotRead;
}
void career_saves_start();
void career_saves_saving();
void career_saves_saved();
void career_saves_listed(const std::vector<career_saves::SlotRead> &slots, bool complete, bool at_load);
// Save sandbox (save_sandbox.cpp): under `f1mp test` with a run token, or with the launcher's saves.
std::vector<CoreMixins> save_sandbox_mixins();
void save_sandbox_commit();
bool save_sandbox_test_mode();
bool save_sandbox_launcher_mode();
bool save_sandbox_armed();
std::string save_sandbox_source();
std::string save_sandbox_file();
std::string career_saves_id_for_slot(const std::string &key);
// FileWrite calls the sandbox has served: a save that reached the save device.
uint32_t save_sandbox_writes();
// The test run `f1mp test` started the game for (SWOFTY_TEST_RUN, SWOFTY_TEST_TOKEN and run.json, read
// once by save_sandbox.cpp, which ends the game when they disagree); null outside a test run.
struct TestRun {
    std::wstring dir;
    std::string id, token;
    bool sandbox = false;
    int agent_v = 0;
    std::vector<std::string> dlls;
};
const TestRun *test_run();
// run.json still holds the token: false once `f1mp test` has retired the run.
bool test_run_live();
// W4 career snapshots (native/src/core/snapshots/runtime.cpp): started once the save splice settles.
void snapshots_start(bool splice_live);
// career::report_season from caller (an addon id): SWOFTY_ASSET_OK, _LATE, _INVALID or SWOFTY_REFUSED.
int snapshots_report_season(const std::string &caller, int year, int season_index);
// The addon that registered the career content of data_group (and its save container name), or empty.
std::string career_content_owner(const std::string &data_group, std::string *save_name = nullptr);
// The save slot a career load selects (site_Game_SaveSlot_Selected), the career id committed when the
// career tracker sees the career loaded (<mode>:<slot guid>, else the tracker's key), and that id for a
// tracker key.
void snapshots_slot_selected(uintptr_t manager, uint64_t slot_id, uint64_t slot_guid);
std::string snapshots_career_loaded(const char *key, unsigned mode);
std::string snapshots_career_id(const char *key);
std::vector<CoreMixins> ers_newlap_mixins();
void ers_newlap_commit();
std::vector<CoreMixins> ers_mode_mixins();
void ers_mode_commit();
std::vector<CoreMixins> car_levers_mixins();
void car_levers_commit();
std::vector<CoreMixins> standings_guard_mixins();
void standings_guard_commit();
std::vector<CoreMixins> fe_tile_guard_mixins();
void fe_tile_guard_commit();
std::vector<CoreMixins> ai_corner_guard_mixins();
void ai_corner_guard_commit();
std::vector<CoreMixins> watchdog_grace_mixins();
void watchdog_grace_commit();
void watchdog_flow(uint32_t hash, uint32_t to, const char *name);
std::vector<CoreMixins> tyre_slots_mixins();
void tyre_slots_commit();
std::vector<CoreMixins> garage_crew_mixins();
std::vector<CoreMixins> pit_slots_mixins();
void garage_crew_commit();
void pit_slots_commit();
std::vector<CoreMixins> pit_box_mixins();
void pit_box_commit();
std::vector<CoreMixins> garage_slots_mixins();
void garage_slots_commit();
std::vector<CoreMixins> chassis_registry_mixins();
void chassis_registry_commit();
std::vector<CoreMixins> track_acclim_mixins();
void track_acclim_commit();
std::vector<CoreMixins> audio_queue_mixins();
void audio_queue_commit();
std::vector<CoreMixins> heap_sizes_mixins();
void heap_sizes_commit();
bool save_heap_raised(uint64_t &bytes);
std::vector<CoreMixins> track_select_mixins();
void track_select_commit();
std::vector<CoreMixins> team_slots_mixins();
void team_slots_commit();
std::vector<CoreMixins> netplay_icons_mixins();
void netplay_icons_commit();
uint8_t *team_record_of(uint16_t stat);
void team_record_inserted(std::function<void(uint16_t, uint8_t *)> fn);
std::vector<CoreMixins> leaderboard_guard_mixins();
void leaderboard_guard_commit();
std::vector<CoreMixins> thrustmaster_lcd_mixins();
void thrustmaster_lcd_commit();
std::vector<CoreMixins> garage_setup_mixins();
void garage_setup_commit();
std::vector<CoreMixins> session_enter_mixins();
void session_enter_commit();
void hud_setup();
void focus_setup();
void hud_fatal();
void grade_setup();
void postfx_setup();
void telemetry_setup();
void netplay_setup();
void netplay_bridge_setup();
// Refuses the game's lookups and HTTP connects to EA and Codemasters hosts (ea_guard.cpp).
void ea_guard_setup();
void netplay_bridge_status_changed();
void netplay_exit();
void netplay_drive_setup();
void netplay_room_setup();
bool netplay_room_requested();
void netplay_room_screen(const char *name);
void netplay_flow(uint32_t hash, uint32_t to);
void netplay_screen(const char *name);
bool netplay_popup(const char *id, void *fn, void *ctx);
void netplay_home_vm(void *vm);
void netplay_tick();
void net_trace_setup();
void udp_ids_setup();
// The test agent (native/src/core/testagent/agent.cpp): armed only by `f1mp test`'s run token.
void testagent_setup();
void direct_start_setup();
void direct_start_tick();
void direct_start_profile_loaded(bool ok);
void direct_start_profile_read();
// The room target's setup (netplay_drive.cpp registers it): polled on the flow thread after the profile has
// loaded, while StateFTUE is held, until it answers done or failed.
using DirectStartSetup = runtime::direct_start::Setup (*)(std::string &why);
void direct_start_register_room(DirectStartSetup setup);
// The Time Trial target's setup (a session request with "mode": "time_trial"; direct_start_tt.cpp registers it):
// polled the same way, with the request's text from direct_start_request_text().
void direct_start_register_time_trial(DirectStartSetup setup);
// Every screen the UI opens while a direct start is armed (ui.cpp's open hook), for the Time Trial's own steps.
void direct_start_register_time_trial_screen(void (*screen)(const char *name));
// The flow graph's root state (StateApp) once the flow has updated it, or null.
void *direct_start_flow_root();
// Despawns the front end's main scene (the menu backdrop) once; a direct start never runs StateFrontend's exit.
void direct_start_despawn_frontend_scene(const char *when);
std::string direct_start_request_text();
// For the cover: 0 while a session target is on its way, 1 once its session is on screen, -1 when it failed.
int direct_start_cover_signal();
// A UI screen opened (ui.cpp): the session's own screen takes the cover off.
void direct_start_screen(const char *name);
uintptr_t direct_start_set_mode();
// What the direct start knows on the flow thread from the profile load on: the game manager and the game modes
// context (the object Game_Game_SetCurrentGameMode takes, [game manager+0xc0]).
struct DirectStartContext {
    void *game_manager = nullptr, *game_modes = nullptr;
};
DirectStartContext direct_start_context();
std::string code_where(uintptr_t a);
void testagent_tick();
void testagent_fired(const std::string &id, bool hot);
void testagent_screen(const char *name);
void testagent_flow(const std::string &name, uint32_t to);
void testagent_home_vm(void *vm);
void testagent_route_ran(const std::string &name);
bool frame_capture_request(const std::wstring &path);
int frame_capture_poll(std::string *why);
int group_add(const json::Value &entry);
int group_extend(const std::string &group, const std::string &erp);
void own_item(uint32_t key, const std::string &addon, const std::string &what);
int db_add(const json::Value &entry);
void db_older_edits(const std::function<void(uint64_t key, int tag, const std::string &text)> &fn);
int lang_add(const json::Value &entry);
int file_add(const json::Value &entry);
std::string file_route_prefix();
bool file_route(const std::string &game_path, const std::string *target, const std::string &source, std::string &why);
bool file_route_target(const std::string &game_path, std::string &target);

struct Robin {
    uint8_t *s;

    template <class T> T at(size_t off) const { return *reinterpret_cast<const T *>(s + off); }
    uint64_t mult() const { return at<uint64_t>(0x10); }
    uint64_t *slots() const { return at<uint64_t *>(0x18); }
    uint8_t *info() const { return at<uint8_t *>(0x20); }
    uint64_t count() const { return at<uint64_t>(0x28); }
    uint64_t mask() const { return at<uint64_t>(0x30); }
    uint32_t inc() const { return at<uint32_t>(0x40); }
    uint32_t shift() const { return at<uint32_t>(0x44); }

    size_t buffer() const
    {
        uint64_t n = mask() + 1, a = n < 0x28f5c28f5c28f5dull ? n * 80 / 100 : (n / 100) * 80;
        return size_t(n + std::min<uint64_t>(a, 0xff));
    }

    bool sane() const
    {
        if (!readable(s, 0x48))
            return false;
        uint64_t n = mask() + 1;
        return n >= 8 && !(n & (n - 1)) && inc() >= 1 && inc() <= 0x20 && slots() && info() &&
               readable(info(), 1) && readable(info() + buffer(), 1) && readable(slots(), 16) &&
               readable(reinterpret_cast<uint8_t *>(slots()) + buffer() * 16 - 16, 16);
    }

    uint64_t *find(uint64_t key) const
    {
        uint64_t h = key * mult();
        h ^= h >> 33;
        uint32_t inf = inc() + ((uint32_t(h) & 0x1f) >> (shift() & 0x3f));
        size_t idx = size_t((h >> 5) & mask());
        const uint8_t *in = info();
        uint64_t *sl = slots();
        do {
            if (inf == in[idx] && sl[idx * 2] == key)
                return &sl[idx * 2];
            ++idx;
            inf += inc();
            if (inf == in[idx] && sl[idx * 2] == key)
                return &sl[idx * 2];
            ++idx;
            inf += inc();
        } while (inf <= in[idx]);
        return nullptr;
    }
};

}
