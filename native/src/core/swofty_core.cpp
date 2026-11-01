#include <swofty/game.hpp>
#include <swofty/mixin.hpp>

#include "perf.hpp"
#include "runtime.hpp"

#include <safetyhook.hpp>

using namespace swofty::game;

bool settings_command(uint32_t hash);

#if defined(SWOFTY_NO_MOD_SETTINGS)
#define SWOFTY_MOD_SETTINGS_MODE "compiled out (diagnostic build: no site hook, no tile)"
#elif defined(SWOFTY_MOD_SETTINGS_NOOP)
#define SWOFTY_MOD_SETTINGS_MODE "site hook installed, handler is a no-op (diagnostic build: no tile)"
#else
#define SWOFTY_MOD_SETTINGS_MODE "active"
#define SWOFTY_MOD_SETTINGS_ACTIVE 1
#endif

namespace {

static_assert(sizeof(TileEntry) == 0x10);

unsigned long long rva(uintptr_t a)
{
    return a - swofty::api->module_base();
}

#ifdef SWOFTY_MOD_SETTINGS_ACTIVE

constexpr const char *ROUTE = "mod_menu";
constexpr const char *TITLE = "ADDON SETTINGS";
constexpr const char *DESCRIPTION = "Configure the addons installed by Swofty Launcher";

unsigned hook_calls, populate_calls, action_calls;
void **mod_vt;

void set_string(void *reg, void *parent, const char *name, const char *text)
{
    NeStr *s = nullptr;
    Ne_Make_StrFromCStr(&s, text, 0, 0);
    Vm_Set_String(reg, parent, name, &s, 1);
    Ne_Make_StrFromCStr(&s, nullptr, 0, 0);
}

void mod_populate(HomeItem *item, void *node, uint32_t a7)
{
    unsigned call = ++populate_calls;
    FeTile_ItemBase_Populate(item, node, a7);
    auto vm_node = static_cast<uint8_t *>(item->button_node.node);
    if (!vm_node) {
        swofty::log("[core] populate #%u item=%p: no VM node after base populate, label left as is", call, item);
        return;
    }
    void *reg = *g_Vm_Registry, *parent = vm_node + 8;
    uint8_t no = 0;
    set_string(reg, parent, "title", TITLE);
    Vm_Set_Bool(reg, parent, "title_localised", &no, 1);
    set_string(reg, parent, "description", DESCRIPTION);
    Vm_Set_Bool(reg, parent, "description_localised", &no, 1);
    swofty::log("[core] populate #%u item=%p node=%p a7=%u vm=%p: title='%s' description set, localised=false", call,
                item, node, a7, vm_node, TITLE);
}

Action *mod_get_action(HomeItem *item, Action *out)
{
    out->type = 1;
    out->str = nullptr;
    Ne_Make_StrFromCStr(&out->str, ROUTE, 0, 0);
    swofty::log("[core] getAction #%u item=%p -> {type 1, route '%s'}", ++action_calls, item, ROUTE);
    return out;
}

HomeItem *last_item(TileList *list, size_t *count)
{
    auto begin = reinterpret_cast<uintptr_t>(list->items_begin), end = reinterpret_cast<uintptr_t>(list->items_end);
    if (!begin || end <= begin || (end - begin) % sizeof(TileEntry))
        return nullptr;
    *count = list->items_end - list->items_begin;
    Tile *tile = list->items_end[-1].tile;
    return tile ? tile->item : nullptr;
}

// MULTIPLAYER CAREER home tile: a second copy of fe_career's two-player career item ("online_career",
// 0x1b0 bytes, vtable vt_FeTile_OnlineCareerItem 0x1457043e0) on the home screen. The stock item greys
// itself out offline: slot 1 (online state, 0x14172b610) sets enabled from item+0x150 or, offline,
// enabled false with lng_2p_career_offline_desc; slot 12 (0x14172b760) clears +0x150 and asks EA's career
// service for the player's online careers. This copy is always enabled and never asks EA. Its getAction is
// the stock one (0x14172b2d0): with +0x1a8 set it resets the wizard state at +0x148 and routes
// career_wizard_online, which the UI build copies from fe_career's route group into fe_home's.
const runtime::Sig S_ONLINE_CAREER_VT{"online career tile vtable",
                                      "C4 C1 78 11 B4 24 64 01 00 00 48 8D 05 ?? ?? ?? ?? 49 89 04 24 C4 C1 78 11 B4 24 "
                                      "88 01 00 00",
                                      0x57043e0, runtime::Sig::riprel, 10, 13, 7};
constexpr const char *MP_ID = "swofty_mp_career";
constexpr const char *MP_TITLE = "ZWEISPIELERKARRIERE";
constexpr const char *MP_DESCRIPTION = "Zu zweit von 2016 bis 2026: \xC3\xBC" "ber deinen F1-Multiplayer-Server, "
                                       "ohne EA-Online";
constexpr size_t MP_SIZE = 0x1b0, MP_WIZARD = 0x148, MP_ENABLED = 0x150, MP_NEW = 0x1a8;
enum : size_t { MP_SET_ONLINE = 1, MP_INIT = 3, MP_POPULATE = 4, MP_ACTION = 5, MP_GATE_A = 6, MP_FETCH = 12, MP_SYNC = 19 };

void **mp_stock;
unsigned mp_calls;

// The two-player career's own wording, set while the player is in the flow this tile starts (from its
// getAction) and cleared when the home screen builds its tiles again, so the single-player career keeps
// the game's texts. German for the game's "ger", English for every other language.
struct MpText {
    const char *key, *ger, *eng;
};
const MpText MP_TEXTS[] = {
    {"lng_screen_career_setup", "Zweispielerkarriere", "Two-Player Career"},
    {"lng_driver_career_overview", "Zweispielerkarriere", "Two-Player Career"},
    {"lng_career_wizard_season_length", "Zweispielerkarriere: Saisonl\xC3\xA4" "nge", "Two-Player Career: season length"},
    {"lng_2p_career_choose_friend_title", "Mitspieler:in per Server-Code einladen", "Invite your teammate by server code"},
    {"lng_2p_career_choose_friend_subtitle", "Dein:e Mitspieler:in tritt mit dem 6-stelligen Server-Code deines F1-Multiplayer-Servers bei, sobald du deinen Fahrer eingerichtet hast", "Your teammate joins with your F1 Multiplayer server's six-digit code once you have set up your driver"},
    {"lng_invite_friends", "Mitspieler:in einladen", "Invite your teammate"},
    {"lng_2p_career_later_title", "Sp\xC3\xA4" "ter per Server-Code", "Later, by server code"},
    {"lng_2p_career_later_subtitle", "Dein:e Mitspieler:in kann sp\xC3\xA4" "ter jederzeit mit dem Server-Code beitreten", "Your teammate can join any time later with the server code"},
    {"lng_invite_friend_later", "Mitspieler:in kommt sp\xC3\xA4" "ter per Server-Code", "Teammate joins later by server code"},
    {"lng_career_wizard_settings", "Zweispielerkarriere: Einstellungen", "Two-Player Career: settings"},
    {"lng_career_wizard_add_team", "Zweispielerkarriere: 11. Team", "Two-Player Career: 11th team"},
    {"lng_career_wizard_driver_icons", "Zweispielerkarriere: Fahrer-Icons", "Two-Player Career: driver icons"},
    {"lng_driver_select_instruction", "Zweispielerkarriere: Fahrer w\xC3\xA4" "hlen", "Two-Player Career: choose your driver"},
    {"lng_team_select", "Zweispielerkarriere: Team w\xC3\xA4" "hlen", "Two-Player Career: choose your team"},
};
bool mp_texts_on;

void mp_texts(bool on)
{
    const SwoftyLang *lang = runtime::lang_table();
    if (on == mp_texts_on || !lang || !lang->available())
        return;
    for (auto &t : MP_TEXTS)
        if (on) {
            lang->override_set("*", t.key, t.eng);
            lang->override_set("ger", t.key, t.ger);
        } else {
            lang->override_clear("*", t.key);
            lang->override_clear("ger", t.key);
        }
    mp_texts_on = on;
    lang->refresh();
    swofty::log("[core] mp career: two-player texts %s (%zu keys)", on ? "on" : "off", std::size(MP_TEXTS));
}

// An NeStr's text: length in the low 20 bits of the dword at +6, characters from +9 (as the test agent reads it).
std::string nestr_text(const NeStr *s)
{
    auto b = reinterpret_cast<const uint8_t *>(s);
    if (!b || !runtime::readable(b, 0x10))
        return {};
    uint32_t lf;
    memcpy(&lf, b + 6, 4);
    size_t n = std::min<size_t>(lf & 0xfffff, 96);
    return runtime::readable(b + 9, n) ? std::string(reinterpret_cast<const char *>(b + 9), n) : std::string();
}

template <class T> T &at(HomeItem *item, size_t off)
{
    return *reinterpret_cast<T *>(reinterpret_cast<uint8_t *>(item) + off);
}

void mp_text(HomeItem *item)
{
    auto vm_node = static_cast<uint8_t *>(item->button_node.node);
    if (!vm_node)
        return;
    void *reg = *g_Vm_Registry, *parent = vm_node + 8;
    uint8_t no = 0, yes = 1;
    set_string(reg, parent, "title", MP_TITLE);
    Vm_Set_Bool(reg, parent, "title_localised", &no, 1);
    set_string(reg, parent, "description", MP_DESCRIPTION);
    Vm_Set_Bool(reg, parent, "description_localised", &no, 1);
    Vm_Set_Bool(reg, parent, "enabled", &yes, 1);
}

// Slot 19 syncs the title key and the enabled property from +0x150; keep +0x150 set and our text on top.
void mp_sync(HomeItem *item)
{
    at<uint8_t>(item, MP_ENABLED) = 1;
    reinterpret_cast<void (*)(HomeItem *)>(mp_stock[MP_SYNC])(item);
    mp_text(item);
}

void mp_set_online(HomeItem *item, int state)
{
    swofty::log("[core] mp career tile %p: online state %d from the game, kept enabled", item, state);
    mp_sync(item);
}

// The home screen's select runs getAction only when gate_a is false or gate_b returns 2; the stock item's
// gate_a is true and its gate_b returns +0x170 (0), so it is ungated here as the home tiles are.
bool mp_gate_a(HomeItem *)
{
    return false;
}

void mp_fetch(HomeItem *item)
{
    at<uint8_t>(item, MP_ENABLED) = 1;
}

uint32_t mp_init(HomeItem *item, void *ctx, void *services)
{
    auto rc = reinterpret_cast<uint32_t (*)(HomeItem *, void *, void *)>(mp_stock[MP_INIT])(item, ctx, services);
    swofty::log("[core] mp career tile %p: init returned %u, wizard state %p", item, rc, at<void *>(item, MP_WIZARD));
    return rc;
}

void mp_populate(HomeItem *item, void *node, uint32_t a7)
{
    FeTile_ItemBase_Populate(item, node, a7);
    mp_sync(item);
    swofty::log("[core] mp career tile %p: populated, title '%s', enabled", item, MP_TITLE);
}

Action *mp_get_action(HomeItem *item, Action *out)
{
    if (!at<void *>(item, MP_WIZARD)) {
        out->type = 1;
        out->str = nullptr;
        Ne_Make_StrFromCStr(&out->str, "career", 0, 0);
        swofty::log("[core] mp career tile %p: no wizard state from init; routing career instead", item);
        return out;
    }
    at<uint8_t>(item, MP_NEW) = 1; // a new two-player career: the stock action resets the wizard, routes career_wizard_online
    auto r = reinterpret_cast<Action *(*)(HomeItem *, Action *)>(mp_stock[MP_ACTION])(item, out);
    mp_texts(true);
    swofty::log("[core] mp career tile %p: getAction -> {type %d, route '%s'}", item, r->type,
                nestr_text(r->str).c_str());
    return r;
}

// The two-player career's EA careerservice calls. Each is a UI online operation (ServerMethodOperation;
// CreateCareer's is built by 0x14107b110, vtable 0x1456bae60) whose method name sits at +0x118 as a C
// string. Its onStart first checks the EA sign-in and, offline, ends at once through the operations'
// common finish (0x14443acd0, a jmp thunk to 0x1543d3640) with an error code byte (4 = not signed in;
// the player sees online_services_generic_exception_online and stays on the screen). Code 0 is success:
// finish then runs the operation's onComplete continuation and builds its result. While the player is in
// the flow the home tile started, finish is told 0 for the career service's writing calls, so the game
// carries on as if EA had stored the career; nothing is sent anywhere. Other calls keep their error and
// are logged, to answer them once their results are understood.
constexpr uintptr_t OP_FINISH = 0x443acd0, OP_FINISH_TARGET = 0x143d3640;
constexpr size_t OP_METHOD = 0x118;
safetyhook::InlineHook h_op_finish;

std::string op_method(void *op)
{
    auto at = reinterpret_cast<const char *const *>(static_cast<uint8_t *>(op) + OP_METHOD);
    if (!runtime::readable(at, sizeof(*at)) || !runtime::readable(*at, 1))
        return {};
    std::string name;
    for (const char *c = *at; name.size() < 64 && runtime::readable(c, 1) && *c; c++)
        name += *c;
    return name;
}

bool answered_locally(const std::string &method)
{
    for (const char *prefix : {"Create", "Update", "Delete", "Set", "Add", "Remove", "Join", "Leave", "GetCareer", "GetSessions"})
        if (method.rfind(prefix, 0) == 0)
            return true;
    return false;
}

void mp_op_finish(void *op, uint8_t *code)
{
    if (mp_texts_on && code && *code) {
        std::string method = op_method(op);
        if (answered_locally(method)) {
            uint8_t ok = 0;
            swofty::log("[core] mp career: %s answered locally (success instead of EA error %u)", method.c_str(),
                        unsigned(*code));
            h_op_finish.call<void>(op, &ok);
            return;
        }
        swofty::log("[core] mp career: %s ends with EA error %u (not answered locally)", method.c_str(),
                    unsigned(*code));
    }
    h_op_finish.call<void>(op, code);
}

void mp_install_online_ops()
{
    static bool tried;
    if (tried)
        return;
    tried = true;
    uintptr_t base = swofty::api->module_base();
    auto at = reinterpret_cast<const uint8_t *>(base + OP_FINISH);
    int32_t rel = 0;
    bool same = runtime::readable(at, 5) && at[0] == 0xE9;
    if (same) {
        memcpy(&rel, at + 1, 4);
        same = uintptr_t(at + 5 + rel) == base + OP_FINISH_TARGET;
    }
    if (same)
        if (auto r = safetyhook::InlineHook::create(const_cast<uint8_t *>(at), reinterpret_cast<void *>(&mp_op_finish)))
            h_op_finish = std::move(*r);
    swofty::log("[core] mp career: online operation finish at base+0x%llx %s", (unsigned long long)OP_FINISH,
                h_op_finish ? "hooked; career service writes are answered locally in the two-player flow"
                            : "not as expected; career creation stays online-only");
}

// Replaces the Career copy the adder just pushed with a two-player career item built the way the builder's
// fe_career case builds one: the base HomeItem fields, then zeroed +0x148..+0x1b0.
void add_mp_tile(TileList *list, swofty::Context &c, unsigned call)
{
    if (!mp_stock) {
        mp_stock = reinterpret_cast<void **>(runtime::find(S_ONLINE_CAREER_VT, "home", "core"));
        swofty::log("[core] mp career tile: two-player career vtable %p", mp_stock);
    }
    if (!mp_stock)
        return;
    mp_install_online_ops();
    mp_texts(false);
    size_t before = 0, after = 0;
    last_item(list, &before);
    FeHome_Add_CareerTile(list, uint8_t(c.r12 + 1));
    HomeItem *career = last_item(list, &after);
    c.r12 += 1;
    c.r15 += 1;
    Tile *tile = after == before + 1 ? list->items_end[-1].tile : nullptr;
    auto alloc = tile ? *reinterpret_cast<NeAllocator **>(reinterpret_cast<uint8_t *>(tile) + 0xc8) : nullptr;
    if (!career || !alloc) {
        swofty::log("[core] hook #%u: mp career tile not built (items %zu -> %zu, tile %p)", call, before, after, tile);
        return;
    }
    auto vt = reinterpret_cast<void **>(alloc->vtable);
    auto item = static_cast<HomeItem *>(reinterpret_cast<void *(*)(NeAllocator *, size_t, size_t)>(vt[3])(alloc, MP_SIZE, 0x10));
    if (!item) {
        swofty::log("[core] hook #%u: mp career tile allocation failed; Career copy left", call);
        return;
    }
    memcpy(item, career, sizeof(uint8_t) * MP_WIZARD);
    memset(reinterpret_cast<uint8_t *>(item) + MP_WIZARD, 0, MP_SIZE - MP_WIZARD);
    item->id = nullptr;
    Ne_Make_StrFromCStr(&item->id, MP_ID, 0, 0);
    item->vtable = reinterpret_cast<ItemVtbl *>(mp_stock);
    swofty::subclass(item, {{MP_SET_ONLINE, reinterpret_cast<void *>(mp_set_online)},
                            {MP_INIT, reinterpret_cast<void *>(mp_init)},
                            {MP_POPULATE, reinterpret_cast<void *>(mp_populate)},
                            {MP_ACTION, reinterpret_cast<void *>(mp_get_action)},
                            {MP_GATE_A, reinterpret_cast<void *>(mp_gate_a)},
                            {MP_FETCH, reinterpret_cast<void *>(mp_fetch)},
                            {MP_SYNC, reinterpret_cast<void *>(mp_sync)}});
    tile->item = item;
    reinterpret_cast<void (*)(HomeItem *, int)>(reinterpret_cast<void **>(career->vtable)[0])(career, 0);
    reinterpret_cast<void (*)(NeAllocator *, void *)>(vt[7])(alloc, career);
    swofty::log("[core] hook #%u: added the mp career tile button_%u (items %zu -> %zu) item=%p, #%u", call,
                unsigned(c.r12), before, after, item, ++mp_calls);
}

#endif

bool check()
{
    uintptr_t site = site_FeHome_AfterCareerCall.address(), builder = FeTile_Build_List.address();
    swofty::log("[core] swofty_core init, game base 0x%llx", (unsigned long long)swofty::api->module_base());
    swofty::log("[core] ADDON SETTINGS mixin: %s", SWOFTY_MOD_SETTINGS_MODE);
    if (site < builder || site - builder > 0x8000) {
        swofty::log("[core] hook site 0x%llx is not inside the builder at 0x%llx; installing nothing",
                    (unsigned long long)site, (unsigned long long)builder);
        return false;
    }
    swofty::log("[core] builder base+0x%llx, hook site base+0x%llx", rva(builder), rva(site));
    swofty::log("[core] career_adder base+0x%llx, base_populate base+0x%llx, nestr_from base+0x%llx",
                rva(FeHome_Add_CareerTile.address()), rva(FeTile_ItemBase_Populate.address()),
                rva(Ne_Make_StrFromCStr.address()));
    swofty::log("[core] registry global base+0x%llx (= %p now), vm_set_string base+0x%llx, vm_set_bool base+0x%llx",
                rva(g_Vm_Registry.address()), *g_Vm_Registry, rva(Vm_Set_String.address()), rva(Vm_Set_Bool.address()));
    return true;
}

}

#if defined(SWOFTY_MOD_SETTINGS_NOOP)

SWOFTY_SITE(site_FeHome_AfterCareerCall, swofty::Context &c)
{
    static unsigned calls;
    swofty::log("[core] hook #%u (no-op): list(r14)=0x%llx r12=0x%llx r15=0x%llx; nothing added", ++calls,
                (unsigned long long)c.r14, (unsigned long long)c.r12, (unsigned long long)c.r15);
}

#elif defined(SWOFTY_MOD_SETTINGS_ACTIVE)

SWOFTY_SITE(site_FeHome_AfterCareerCall, swofty::Context &c)
{
    unsigned call = ++hook_calls;
    auto list = reinterpret_cast<TileList *>(c.r14);
    swofty::log("[core] hook #%u: list(r14)=0x%llx r12=0x%llx r15=0x%llx rsi=0x%llx rip=0x%llx", call,
                (unsigned long long)c.r14, (unsigned long long)c.r12, (unsigned long long)c.r15,
                (unsigned long long)c.rsi, (unsigned long long)c.rip);
    size_t before = 0, after = 0;
    HomeItem *career = last_item(list, &before);
    if (!career) {
        swofty::log("[core] hook #%u: items vector does not end with a tile, nothing added", call);
        return;
    }
    auto career_vt = reinterpret_cast<void **>(career->vtable);
    uint8_t index = uint8_t(c.r12 + 1);
    FeHome_Add_CareerTile(list, index);
    HomeItem *item = last_item(list, &after);
    c.r12 += 1;
    c.r15 += 1;
    if (!item || after != before + 1 || reinterpret_cast<void **>(item->vtable) != career_vt) {
        swofty::log("[core] hook #%u: adder result unexpected (items %zu -> %zu, item %p, vt %p vs career %p); tile "
                    "left as a plain Career copy",
                    call, before, after, item, item ? static_cast<void *>(item->vtable) : nullptr, career_vt);
        return;
    }
    void **vt = swofty::subclass(item, {{4, reinterpret_cast<void *>(mod_populate)}, {5, reinterpret_cast<void *>(mod_get_action)}});
    if (vt != mod_vt) {
        mod_vt = vt;
        swofty::log("[core] hook #%u: built mod vtable %p from career vtable %p (slot 4 populate %p -> %p, slot 5 "
                    "getAction %p -> %p)",
                    call, vt, career_vt, career_vt[4], vt[4], career_vt[5], vt[5]);
    }
    NeStr *old_id = item->id;
    Ne_Make_StrFromCStr(&item->id, ROUTE, 0, 0);
    swofty::log("[core] hook #%u: added button_%u (items %zu -> %zu) item=%p, id %p -> %p '%s' (old id released by "
                "the game's string setter), r12 -> 0x%llx, r15 -> 0x%llx",
                call, (unsigned)index, before, after, item, old_id, item->id, ROUTE, (unsigned long long)c.r12,
                (unsigned long long)c.r15);
    add_mp_tile(list, c, call);
}

#endif

SWOFTY_WRAP(FeHome_Handle_Command, uint64_t, FEHomeViewModel *vm, NeStr **cmd)
{
    static const uint32_t probe = perf::probe("core", "home command (addon settings)");
    {
        perf::Scope p(probe);
        settings_command(cmd && *cmd ? *reinterpret_cast<const uint32_t *>(*cmd) : 0x1505);
    }
    runtime::netplay_home_vm(vm);
    runtime::testagent_home_vm(vm);
    return orig(vm, cmd);
}

// F1 THE MOVIE home tile. The home case of FeTile_Build_List adds it (FeHome_Add_MovieTile at index
// r15b+2, then r12 = r15) only while feature flag bit 42 ([rsi+5] & 4) is set; with the flag clear the
// je skips to the F1 World block, which numbers itself r12+1 from the tile before. The anchor is the
// movie block's first instruction, the je's fall-through right after the 6 bytes the
// site_FeHome_AfterCareerCall hook relocates; a short jmp there to the F1 World block builds the
// flag-clear list, so the button numbering stays contiguous. No branch lands in the 2 replaced bytes.
namespace {

const runtime::Sig S_MOVIE_TILE{"FeHome_MovieTileBlock",
                                "41 80 C7 02 4C 89 F1 44 89 FA E8 ?? ?? ?? ?? 45 89 FC 45 8D 7C 24 01 4C 89 F1 44 89 FA E8",
                                0x1742fdc, runtime::Sig::addr, 0, 0, 0};

void hide_movie_tile()
{
    static const uint8_t skip_movie[] = {0xEB, 0x10};
    auto at = reinterpret_cast<void *>(runtime::find(S_MOVIE_TILE, "home", "core"));
    DWORD old;
    bool hidden = at && VirtualProtect(at, sizeof skip_movie, PAGE_EXECUTE_READWRITE, &old);
    if (hidden) {
        memcpy(at, skip_movie, sizeof skip_movie);
        VirtualProtect(at, sizeof skip_movie, old, &old);
        FlushInstructionCache(GetCurrentProcess(), at, sizeof skip_movie);
    }
    swofty::log("[core] F1 THE MOVIE home tile %s", hidden ? "hidden" : "left in place (signature did not resolve)");
}

}

SWOFTY_ON_LOAD
{
    runtime::when_code_ready("core: F1 THE MOVIE home tile", hide_movie_tile);
}

SWOFTY_MOD(check)
