#include "d3d12_hooks.hpp"
#include "hud_guard.hpp"
#include "perf.hpp"
#include "postfx_logic.hpp"
#include "runtime.hpp"
#include "view_state.hpp"

#include <swofty/mixin.hpp>

#include <d3d12.h>

#include <algorithm>
#include <atomic>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <cwchar>
#include <cwctype>
#include <memory>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

bool settings_state(const std::string &addon, const std::string &option, size_t &current,
                    std::vector<std::string> *texts, std::vector<std::string> *commands,
                    std::vector<std::string> *values);

// Post-process passes. The game tonemaps the 3D picture into eaid://ego_package/tonemap_output.image,
// grades and encodes it into the back buffer, then draws the HUD into the back buffer. The passes run
// once a frame between the picture and the HUD, so the HUD draws over their result as the game drew it,
// translucent panels included. The game records a frame's lists in parallel, so where they run is
// decided per back buffer: each one is fresh from the barrier that makes it a render target until the
// passes run on it.
//
// - On track while the HUD reads the finished picture (paused, menus over the race), the passes run at
//   the first barrier of a list that moves the back buffer from render target to a read state.
// - Otherwise, in the game's list where it moves eaid://ego_package/hudless_backbuffer.image out of UAV
//   while the back buffer it made a render target is still one (its HUD-less step).
// - Where the list that made the picture ends before that step finishes (the step goes on on another
//   queue), in a list of the core's own submitted right after it.
//
// In the game's lists the core records its passes before the game's barrier and then puts back the
// pipeline, descriptor heaps, compute bindings and graphics descriptor tables it changed
// (d3d12hooks::list_state). The passes copy the back buffer, run as DXIL compute shaders
// (swofty/postfx.hlsli) through a slot's own textures, and copy the result back; with frame generation
// on, the result goes into the game's HUD-less copy too. Each slot has its own fence and working
// textures, so frames the GPU runs at once on different queues never share them; every texture rests
// in NON_PIXEL_SHADER_RESOURCE between recordings. A pass whose pipeline cannot be made, or whose GPU
// time stays above its budget, is turned off and logged; SWOFTY_POSTFX=0 or a swofty/postfx.off file
// turns every pass off.

namespace {

using SerializeRootFn = HRESULT(WINAPI *)(const D3D12_ROOT_SIGNATURE_DESC *, D3D_ROOT_SIGNATURE_VERSION, ID3DBlob **,
                                          ID3DBlob **);

constexpr wchar_t TONEMAP_NAME[] = L"eaid://ego_package/tonemap_output.image";
constexpr wchar_t BACKBUFFER_NAME[] = L"BackBuffer";
constexpr wchar_t HUDLESS_NAME[] = L"eaid://ego_package/hudless_backbuffer.image";
constexpr wchar_t DEPTH_NAME[] = L"eaid://ego_package/gbuffer_depth_stencil.image";
constexpr wchar_t UI_PREFIX[] = L"eaid://ui_package/";
constexpr UINT FRAMES = 8, MAX_PASSES = 32, CONSTANTS = 40, BUDGET_FRAMES = 120, INPUTS = 8, TABLE = 2 + INPUTS + 1;
constexpr D3D12_RESOURCE_STATES HOME = D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;
constexpr D3D12_RESOURCE_STATES READ_STATES =
    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;
enum : int { IN_NONE = -1, IN_ORIGINAL = -2, IN_PREVIOUS = -3 };

struct Live {
    UINT slot;
    std::string option;
};

struct Pass {
    std::string addon, addon_id, name, shader, enabled;
    std::vector<uint8_t> dxil;
    float options[16] = {};
    std::vector<Live> live;
    int inputs[INPUTS] = {IN_NONE, IN_NONE, IN_NONE, IN_NONE, IN_NONE, IN_NONE, IN_NONE, IN_NONE};
    UINT n_inputs = 0;
    int output = -1;
    UINT threads[2] = {8, 8};
    postfx::When when;
    double budget_ms = 1.5;
    ID3D12PipelineState *pso = nullptr;
    bool off = false, on = true;
    const char *why = nullptr;
    double ms_sum = 0, ms_peak = 0;
    unsigned ms_frames = 0, over = 0;
};

struct Tex {
    std::string name, file;
    double scale = 1;
    UINT width = 0, height = 0;
    DXGI_FORMAT format = DXGI_FORMAT_R16G16B16A16_FLOAT;
    bool persistent = false;
    ID3D12Resource *res = nullptr, *upload = nullptr;
    UINT w = 0, h = 0;
    std::string pixels;
    bool clear = false, uploaded = false;
};

struct Toggle {
    std::string addon_id, addon, option;
    int vk = 0;
    bool down = false, off = false;
};

struct Mark {
    bool tonemap = false, ui = false, left_rt = false;
    int slot = -1;
    ID3D12Resource *target = nullptr, *hudless = nullptr, *depth = nullptr, *ran_back = nullptr;
    D3D12_RESOURCE_STATES state = D3D12_RESOURCE_STATE_COMMON, hudless_state = D3D12_RESOURCE_STATE_COMMON,
                          depth_state = D3D12_RESOURCE_STATE_COMMON;
    std::vector<D3D12_RESOURCE_BARRIER> seen;
    std::string order;
};

struct Slot {
    ID3D12CommandAllocator *alloc = nullptr;
    ID3D12GraphicsCommandList *list = nullptr;
    ID3D12Fence *fence = nullptr;
    UINT64 fence_value = 0;
    bool pending = false;
    ULONGLONG armed_at = 0;
    UINT passes = 0;
    int pass_of[MAX_PASSES] = {};
};

// The core's textures for one picture size: each slot's working set, last frame's result, and the
// addons' persistent textures and images.
// A slot's working textures: the picture before the passes, the two the passes write in turn, and
// the addon textures that are neither persistent nor images. Each slot has its own, so passes of two
// frames that the GPU runs at once on different queues never write the same texture.
struct Work {
    ID3D12Resource *orig = nullptr, *pic[2] = {};
    std::vector<ID3D12Resource *> tex;
    bool clear = false;
};

struct Frame {
    D3D12_RESOURCE_DESC desc{};
    DXGI_FORMAT view = DXGI_FORMAT_UNKNOWN;
    Work work[FRAMES];
    ID3D12Resource *prev = nullptr;
    ID3D12DescriptorHeap *heap = nullptr, *cpu = nullptr;
    UINT stride = 0;
    bool prev_valid = false;
    uint32_t prev_serial = 0;
    unsigned prev_frame = 0;
};

struct Retired {
    std::vector<IUnknown *> objects;
    UINT64 after = 0;
};

// One presented frame, for SWOFTY_POSTFX_TRACE.
struct Trace {
    const char *path = "none", *why = nullptr;
    unsigned runs = 0, ran = 0, live = 0;
    std::string skipped;
    ID3D12Resource *back = nullptr;
    view::State view;
};
// The resources the barrier hook watches, read without the lock. Writers hold the lock; readers only
// load. A key keeps its slot and erase only clears its bits, so lookups never walk over tombstones.
// The game names short-lived UI resources all the time, so once half the slots hold keys the writer
// copies the live keys into a new table and publishes it, which keeps probe runs short. A replaced
// table is freed a few seconds later, long after any reader that loaded it has returned.
class PtrSet {
    size_t mask;
    std::unique_ptr<std::atomic<uintptr_t>[]> key;
    std::unique_ptr<std::atomic<uint8_t>[]> on;
    size_t used = 0, live = 0;

    size_t hash(uintptr_t k) const { return size_t((k * 0x9E3779B97F4A7C15ull) >> 40) & mask; }

    size_t find(uintptr_t k, bool claim)
    {
        for (size_t i = hash(k), n = 0; n <= mask; i = (i + 1) & mask, n++) {
            uintptr_t v = key[i].load(std::memory_order_acquire);
            if (v == k)
                return i;
            if (v == 0) {
                if (!claim)
                    return mask + 1;
                key[i].store(k, std::memory_order_release);
                used++;
                return i;
            }
        }
        return mask + 1;
    }

public:
    explicit PtrSet(size_t n) : mask(n - 1), key(new std::atomic<uintptr_t>[n]()), on(new std::atomic<uint8_t>[n]()) {}

    uint8_t get(const void *p) const
    {
        uintptr_t k = uintptr_t(p);
        for (size_t i = hash(k), n = 0; n <= mask; i = (i + 1) & mask, n++) {
            uintptr_t v = key[i].load(std::memory_order_acquire);
            if (v == k)
                return on[i].load(std::memory_order_acquire);
            if (v == 0)
                return 0;
        }
        return 0;
    }
    size_t size() const { return mask + 1; }
    size_t count() const { return live; }
    bool half_full() const { return used * 2 >= mask + 1; }
    void set(const void *p, uint8_t bits)
    {
        size_t i = find(uintptr_t(p), bits != 0);
        if (i > mask)
            return;
        uint8_t was = on[i].load(std::memory_order_relaxed);
        live += (!was && bits) - (was && !bits);
        on[i].store(bits, std::memory_order_release);
    }
    template <class F> void each(F f) const
    {
        for (size_t i = 0; i <= mask; i++) {
            uintptr_t k = key[i].load(std::memory_order_relaxed);
            uint8_t b = on[i].load(std::memory_order_relaxed);
            if (k && b)
                f(reinterpret_cast<const void *>(k), b);
        }
    }
};

enum : uint8_t { W_TONEMAP = 1, W_BACK = 2, W_HUDLESS = 4, W_UI = 8, W_DEPTH = 16 };

// The few resources the passes key on and the game's many UI images live in separate tables, so UI
// churn can never push the tonemap or the back buffers out. Freed resources leave dead slots behind,
// so once half a table's slots are taken it is rebuilt from its live keys, at least four times as
// large as they are; a lookup for an unwatched resource, nearly every barrier, stays a probe or two.
template <size_t MIN> class Table {
    std::atomic<PtrSet *> cur{new PtrSet(MIN)};
    std::vector<std::pair<PtrSet *, ULONGLONG>> retired;
    const char *what;
    size_t told = MIN;

public:
    explicit Table(const char *name) : what(name) {}

    uint8_t get(const void *p) const { return cur.load(std::memory_order_acquire)->get(p); }

    void put(const void *p, uint8_t bits)
    {
        PtrSet *w = cur.load(std::memory_order_relaxed);
        if (!bits || !w->half_full()) {
            w->set(p, bits);
            return;
        }
        size_t n = MIN;
        while (n < (w->count() + 1) * 4)
            n *= 2;
        auto *fresh = new PtrSet(n);
        w->each([&](const void *k, uint8_t b) {
            if (k != p)
                fresh->set(k, b);
        });
        fresh->set(p, bits);
        if (n > told) {
            told = n;
            swofty::log("[postfx] the %s table grew to %zu slots for %zu live resource(s)", what, n, fresh->count());
        }
        cur.store(fresh, std::memory_order_release);
        ULONGLONG now = GetTickCount64();
        std::erase_if(retired, [&](const std::pair<PtrSet *, ULONGLONG> &r) {
            if (now - r.second < 5000)
                return false;
            delete r.first;
            return true;
        });
        retired.emplace_back(w, now);
    }
};

Table<256> key_watched("key resource");
Table<4096> ui_watched("UI");

uint8_t watched_bits(const void *p)
{
    return key_watched.get(p) | ui_watched.get(p);
}

void watch(const void *p, uint8_t bits)
{
    key_watched.put(p, uint8_t(bits & ~W_UI));
    ui_watched.put(p, uint8_t(bits & W_UI));
}

std::recursive_mutex lock;
std::vector<Pass> passes;
std::vector<Tex> textures;
std::vector<Toggle> toggles;
bool uses_previous;
ID3D12Resource *depth_res;
D3D12_RESOURCE_STATES depth_state;
bool depth_logged;
std::unordered_map<ID3D12Resource *, std::wstring> names;
std::unordered_map<ID3D12GraphicsCommandList *, Mark> marks;
unsigned order_left, final_left, final_at;
struct FinalProbe {
    ID3D12Resource *rb = nullptr;
    unsigned frame = 0, land = 0;
    bool pending = false;
};
FinalProbe finals[256];
D3D12_PLACED_SUBRESOURCE_FOOTPRINT final_fp;
UINT final_w, final_h;
// Each back buffer's use for one frame: fresh from the barrier that makes it a render target until the
// passes run on it, and the depth buffer as the picture's list left it.
struct BackUse {
    ID3D12Resource *back = nullptr;
    bool fresh = false;
    ID3D12Resource *depth = nullptr;
    D3D12_RESOURCE_STATES depth_state = D3D12_RESOURCE_STATE_COMMON;
    unsigned seen = 0;
};
BackUse back_uses[6];
unsigned back_seen, hud_read_at;
bool hud_read_seen;


BackUse &back_use(ID3D12Resource *b)
{
    BackUse *oldest = &back_uses[0];
    for (auto &u : back_uses) {
        if (u.back == b)
            return u;
        if (u.seen < oldest->seen)
            oldest = &u;
    }
    *oldest = BackUse{};
    oldest->back = b;
    oldest->seen = ++back_seen;
    return *oldest;
}
bool diag_done, killed, broken, warned_busy, hudless_logged, saw_back, tonemap_named, state_armed;
std::wstring off_file, trace_file, swofty_root_dir;

ID3D12Device *device;
ID3D12RootSignature *root;
Slot slots[FRAMES];
UINT slot_at;
ID3D12QueryHeap *queries;
ID3D12Resource *query_readback;
double ticks_per_ms;
Frame frame;
std::vector<Retired> retired;
std::atomic<bool> fg_on{true};
ULONGLONG started, last_check, live_at, frame_at;
float frametime;
unsigned frame_count, ran_total;

constexpr unsigned NAMED_LINES = 80, STAT_LINES = 30;
constexpr const wchar_t *INTERESTING[] = {L"tonemap", L"output", L"final", L"hudless", L"backbuffer", L"pathtrac",
                                          L"_pt", L"pt_", L"denois", L"upscal", L"composite", L"present",
                                          L"display", L"post", L"lut", L"bloom", L"exposure"};
std::vector<std::wstring> named_seen;

struct Stats {
    unsigned presented = 0, in_list = 0, between = 0, without = 0, doubled = 0, ui_together = 0, unknown_state = 0,
             busy = 0, deferred = 0, dropped = 0;
};
Stats stats;
unsigned stat_lines;
ULONGLONG stat_at;
Trace trace;
unsigned trace_left, trace_present;
ULONGLONG trace_checked;

std::string capture_dir;
ID3D12Resource *capture_rb[2];
D3D12_PLACED_SUBRESOURCE_FOOTPRINT capture_fp;
ULONGLONG capture_next, capture_checked;
unsigned captures;
std::wstring capture_file;
bool capture_asked, capture_pending, capture_now;
UINT64 capture_fence;
int capture_slot = -1;

void release(IUnknown *p)
{
    if (p)
        p->Release();
}

std::string name_of(ID3D12Resource *r)
{
    auto it = names.find(r);
    return it == names.end() ? std::string("?") : runtime::narrow(it->second);
}

bool killed_now()
{
    ULONGLONG now = GetTickCount64();
    if (now - last_check < 1000)
        return killed;
    last_check = now;
    bool was = killed;
    killed = GetFileAttributesW(off_file.c_str()) != INVALID_FILE_ATTRIBUTES;
    if (killed != was)
        swofty::log(killed ? "[postfx] swofty/postfx.off exists; every pass is off" : "[postfx] swofty/postfx.off is gone; passes run again");
    return killed;
}

DXGI_FORMAT view_format(DXGI_FORMAT f)
{
    switch (f) {
    case DXGI_FORMAT_R8G8B8A8_UNORM:
    case DXGI_FORMAT_R8G8B8A8_UNORM_SRGB:
    case DXGI_FORMAT_R8G8B8A8_TYPELESS:
        return DXGI_FORMAT_R8G8B8A8_UNORM;
    case DXGI_FORMAT_R10G10B10A2_UNORM:
    case DXGI_FORMAT_R10G10B10A2_TYPELESS:
        return DXGI_FORMAT_R10G10B10A2_UNORM;
    case DXGI_FORMAT_R16G16B16A16_FLOAT:
    case DXGI_FORMAT_R16G16B16A16_TYPELESS:
        return DXGI_FORMAT_R16G16B16A16_FLOAT;
    case DXGI_FORMAT_R11G11B10_FLOAT:
        return DXGI_FORMAT_R11G11B10_FLOAT;
    default:
        return DXGI_FORMAT_UNKNOWN;
    }
}

UINT format_kind(DXGI_FORMAT view)
{
    return view == DXGI_FORMAT_R10G10B10A2_UNORM ? 1 : view == DXGI_FORMAT_R16G16B16A16_FLOAT ? 2 : 0;
}

ID3D12Resource *buffer(D3D12_HEAP_TYPE type, UINT64 size, D3D12_RESOURCE_STATES state)
{
    D3D12_HEAP_PROPERTIES heap{};
    heap.Type = type;
    D3D12_RESOURCE_DESC bd{};
    bd.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    bd.Width = size;
    bd.Height = 1;
    bd.DepthOrArraySize = 1;
    bd.MipLevels = 1;
    bd.SampleDesc.Count = 1;
    bd.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    ID3D12Resource *r = nullptr;
    return SUCCEEDED(device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &bd, state, nullptr, IID_PPV_ARGS(&r)))
               ? r
               : nullptr;
}

ID3D12Resource *texture(UINT w, UINT h, DXGI_FORMAT f, bool uav)
{
    D3D12_RESOURCE_DESC td{};
    td.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;
    td.Width = w;
    td.Height = h;
    td.DepthOrArraySize = 1;
    td.MipLevels = 1;
    td.Format = f;
    td.SampleDesc = {1, 0};
    td.Flags = uav ? D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS : D3D12_RESOURCE_FLAG_NONE;
    D3D12_HEAP_PROPERTIES heap{};
    heap.Type = D3D12_HEAP_TYPE_DEFAULT;
    ID3D12Resource *r = nullptr;
    return SUCCEEDED(device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &td, HOME, nullptr, IID_PPV_ARGS(&r)))
               ? r
               : nullptr;
}

void transition(ID3D12GraphicsCommandList *list, ID3D12Resource *r, D3D12_RESOURCE_STATES from,
                D3D12_RESOURCE_STATES to)
{
    if (from == to || !r)
        return;
    D3D12_RESOURCE_BARRIER b{};
    b.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    b.Transition.pResource = r;
    b.Transition.Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
    b.Transition.StateBefore = from;
    b.Transition.StateAfter = to;
    d3d12hooks::barrier_original(list, 1u, &b);
}

bool make_root()
{
    auto serialize =
        reinterpret_cast<SerializeRootFn>(GetProcAddress(GetModuleHandleA("d3d12.dll"), "D3D12SerializeRootSignature"));
    if (!serialize)
        return false;
    D3D12_DESCRIPTOR_RANGE ranges[3]{};
    ranges[0].RangeType = D3D12_DESCRIPTOR_RANGE_TYPE_SRV;
    ranges[0].NumDescriptors = 2 + INPUTS;
    ranges[0].OffsetInDescriptorsFromTableStart = 0;
    ranges[1].RangeType = D3D12_DESCRIPTOR_RANGE_TYPE_UAV;
    ranges[1].NumDescriptors = 1;
    ranges[1].OffsetInDescriptorsFromTableStart = 2 + INPUTS;
    D3D12_ROOT_PARAMETER params[2]{};
    params[0].ParameterType = D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS;
    params[0].Constants.Num32BitValues = CONSTANTS;
    params[0].ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
    params[1].ParameterType = D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;
    params[1].DescriptorTable.NumDescriptorRanges = 2;
    params[1].DescriptorTable.pDescriptorRanges = ranges;
    params[1].ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
    D3D12_STATIC_SAMPLER_DESC samplers[4]{};
    for (UINT i = 0; i < 4; i++) {
        samplers[i].Filter = i % 2 == 0 ? D3D12_FILTER_MIN_MAG_MIP_LINEAR : D3D12_FILTER_MIN_MAG_MIP_POINT;
        samplers[i].AddressU = samplers[i].AddressV = samplers[i].AddressW =
            i < 2 ? D3D12_TEXTURE_ADDRESS_MODE_CLAMP : D3D12_TEXTURE_ADDRESS_MODE_WRAP;
        samplers[i].MaxLOD = D3D12_FLOAT32_MAX;
        samplers[i].ShaderRegister = i;
        samplers[i].ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
    }
    D3D12_ROOT_SIGNATURE_DESC rd{};
    rd.NumParameters = 2;
    rd.pParameters = params;
    rd.NumStaticSamplers = 4;
    rd.pStaticSamplers = samplers;
    ID3DBlob *blob = nullptr, *err = nullptr;
    HRESULT hr = serialize(&rd, D3D_ROOT_SIGNATURE_VERSION_1, &blob, &err);
    if (SUCCEEDED(hr))
        hr = device->CreateRootSignature(0, blob->GetBufferPointer(), blob->GetBufferSize(), IID_PPV_ARGS(&root));
    release(blob);
    release(err);
    return SUCCEEDED(hr);
}

// The device comes from the game's resource, never from the queue: a queue can be Streamline's proxy,
// whose device makes proxy command lists that the hooked ExecuteCommandLists cannot take.
bool init(ID3D12Resource *res)
{
    if (FAILED(res->GetDevice(IID_PPV_ARGS(&device))))
        return false;
    if (!make_root()) {
        swofty::log("[postfx] could not make the passes' root signature; post-processing is off");
        return false;
    }
    for (auto &s : slots) {
        if (FAILED(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&s.fence))) ||
            FAILED(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&s.alloc))) ||
            FAILED(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, s.alloc, nullptr, IID_PPV_ARGS(&s.list))))
            return false;
        s.list->Close();
    }
    D3D12_QUERY_HEAP_DESC qd{};
    qd.Type = D3D12_QUERY_HEAP_TYPE_TIMESTAMP;
    qd.Count = FRAMES * MAX_PASSES * 2;
    if (SUCCEEDED(device->CreateQueryHeap(&qd, IID_PPV_ARGS(&queries))))
        query_readback = buffer(D3D12_HEAP_TYPE_READBACK, qd.Count * 8, D3D12_RESOURCE_STATE_COPY_DEST);
    for (auto &p : passes) {
        D3D12_COMPUTE_PIPELINE_STATE_DESC pd{};
        pd.pRootSignature = root;
        pd.CS.pShaderBytecode = p.dxil.data();
        pd.CS.BytecodeLength = p.dxil.size();
        HRESULT hr = device->CreateComputePipelineState(&pd, IID_PPV_ARGS(&p.pso));
        if (FAILED(hr)) {
            p.off = true;
            swofty::log("[postfx] %s's pass %s is off: its pipeline could not be made (0x%08lx)", p.addon.c_str(),
                        p.name.c_str(), hr);
        } else
            swofty::log("[postfx] %s's pass %s is ready (%s, %zu bytes of DXIL)", p.addon.c_str(), p.name.c_str(),
                        p.shader.c_str(), p.dxil.size());
    }
    started = GetTickCount64();
    return true;
}

// The GPU clock rate comes from the first queue the passes run on.
void read_frequency(ID3D12CommandQueue *q)
{
    UINT64 freq = 0;
    if (!ticks_per_ms && query_readback && q && SUCCEEDED(q->GetTimestampFrequency(&freq)) && freq)
        ticks_per_ms = double(freq) / 1000.0;
}

void retire(std::vector<IUnknown *> objects)
{
    std::erase(objects, nullptr);
    if (!objects.empty())
        retired.push_back({std::move(objects), UINT64(frame_count) + 2 * FRAMES});
}

void free_retired()
{
    std::erase_if(retired, [&](Retired &r) {
        if (frame_count < r.after)
            return false;
        for (auto *o : r.objects)
            o->Release();
        return true;
    });
}

bool read_tex_file(Tex &t)
{
    std::string bytes = runtime::read_file(runtime::under_swofty(t.file), size_t(64) << 20);
    uint32_t head[4] = {};
    if (bytes.size() < 16 || memcmp(bytes.data(), "SWTX", 4) != 0)
        return false;
    memcpy(head, bytes.data(), 16);
    t.width = head[1];
    t.height = head[2];
    if (!t.width || !t.height || t.width > 8192 || t.height > 8192 || bytes.size() < 16 + size_t(t.width) * t.height * 4)
        return false;
    t.pixels = bytes.substr(16, size_t(t.width) * t.height * 4);
    t.format = DXGI_FORMAT_R8G8B8A8_UNORM;
    return true;
}

// Makes the textures for a picture of desc's size; the ones being replaced go once the GPU is past them.
bool make_frame(const D3D12_RESOURCE_DESC &d)
{
    DXGI_FORMAT view = view_format(d.Format);
    if (view == DXGI_FORMAT_UNKNOWN) {
        swofty::log("[postfx] the game's picture has format %d, which the passes do not take; post-processing is off",
                    int(d.Format));
        return false;
    }
    std::vector<IUnknown *> old = {frame.prev, frame.heap, frame.cpu};
    for (auto &wk : frame.work) {
        old.insert(old.end(), {wk.orig, wk.pic[0], wk.pic[1]});
        old.insert(old.end(), wk.tex.begin(), wk.tex.end());
    }
    depth_res = nullptr;
    frame = Frame{};
    frame.desc = d;
    frame.view = view;
    UINT w = UINT(d.Width), h = d.Height;
    if (uses_previous)
        frame.prev = texture(w, h, view, false);
    for (auto &t : textures) {
        if (!t.file.empty() && t.res)
            continue;
        postfx::Size s = postfx::texture_size(t.scale, t.width, t.height, w, h);
        t.w = s.w;
        t.h = s.h;
        if (!t.persistent && t.file.empty())
            continue;
        old.push_back(t.res);
        t.res = texture(t.w, t.h, t.format, t.file.empty());
        t.clear = t.file.empty();
        if (!t.res) {
            swofty::log("[postfx] texture %s (%ux%u, format %d) could not be made; post-processing is off",
                        t.name.c_str(), t.w, t.h, int(t.format));
            return false;
        }
    }
    retire(std::move(old));
    D3D12_DESCRIPTOR_HEAP_DESC hd{};
    hd.Type = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;
    hd.NumDescriptors = FRAMES * MAX_PASSES * TABLE + UINT(textures.size()) * (FRAMES + 1) + 1;
    hd.Flags = D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE;
    D3D12_DESCRIPTOR_HEAP_DESC cd = hd;
    cd.NumDescriptors = UINT(textures.size()) * (FRAMES + 1) + 1;
    cd.Flags = D3D12_DESCRIPTOR_HEAP_FLAG_NONE;
    if ((uses_previous && !frame.prev) ||
        FAILED(device->CreateDescriptorHeap(&hd, IID_PPV_ARGS(&frame.heap))) ||
        FAILED(device->CreateDescriptorHeap(&cd, IID_PPV_ARGS(&frame.cpu))))
        return false;
    frame.stride = device->GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
    if (!capture_dir.empty() && !capture_pending) {
        retire({capture_rb[0], capture_rb[1]});
        UINT rows = 0;
        UINT64 row_bytes = 0, total = 0;
        device->GetCopyableFootprints(&d, 0, 1, 0, &capture_fp, &rows, &row_bytes, &total);
        for (auto &r : capture_rb)
            r = buffer(D3D12_HEAP_TYPE_READBACK, total, D3D12_RESOURCE_STATE_COPY_DEST);
    }
    swofty::log("[postfx] passes run on the game's %llux%u picture (format %d) with %zu texture(s) of their own",
                (unsigned long long)d.Width, d.Height, int(d.Format), textures.size());
    return true;
}

bool shared(const Tex &t)
{
    return t.persistent || !t.file.empty();
}

// A slot's working set, made the first time the slot runs on this picture size.
bool ensure_work(int k)
{
    Work &wk = frame.work[k];
    if (wk.orig)
        return true;
    UINT w = UINT(frame.desc.Width), h = frame.desc.Height;
    wk.orig = texture(w, h, frame.view, false);
    wk.pic[0] = texture(w, h, frame.view, true);
    wk.pic[1] = texture(w, h, frame.view, true);
    wk.tex.assign(textures.size(), nullptr);
    bool ok = wk.orig && wk.pic[0] && wk.pic[1];
    for (size_t i = 0; i < textures.size() && ok; i++)
        if (!shared(textures[i]))
            ok = (wk.tex[i] = texture(textures[i].w, textures[i].h, textures[i].format, true)) != nullptr;
    wk.clear = true;
    if (!ok)
        swofty::log("[postfx] the working textures of slot %d could not be made; post-processing is off", k);
    return ok;
}

ID3D12Resource *tex_of(size_t i, int k)
{
    return shared(textures[i]) ? textures[i].res : frame.work[k].tex[i];
}

D3D12_CPU_DESCRIPTOR_HANDLE cpu_at(ID3D12DescriptorHeap *h, UINT i)
{
    return {h->GetCPUDescriptorHandleForHeapStart().ptr + SIZE_T(i) * frame.stride};
}

D3D12_GPU_DESCRIPTOR_HANDLE gpu_at(ID3D12DescriptorHeap *h, UINT i)
{
    return {h->GetGPUDescriptorHandleForHeapStart().ptr + UINT64(i) * frame.stride};
}

void srv(ID3D12Resource *r, DXGI_FORMAT f, D3D12_CPU_DESCRIPTOR_HANDLE at)
{
    D3D12_SHADER_RESOURCE_VIEW_DESC sv{};
    sv.Format = f;
    sv.ViewDimension = D3D12_SRV_DIMENSION_TEXTURE2D;
    sv.Shader4ComponentMapping = D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING;
    sv.Texture2D.MipLevels = 1;
    device->CreateShaderResourceView(r, &sv, at);
}

void uav(ID3D12Resource *r, DXGI_FORMAT f, D3D12_CPU_DESCRIPTOR_HANDLE at)
{
    D3D12_UNORDERED_ACCESS_VIEW_DESC uv{};
    uv.Format = f;
    uv.ViewDimension = D3D12_UAV_DIMENSION_TEXTURE2D;
    device->CreateUnorderedAccessView(r, nullptr, &uv, at);
}

DXGI_FORMAT depth_view(DXGI_FORMAT f)
{
    switch (f) {
    case DXGI_FORMAT_R24G8_TYPELESS:
        return DXGI_FORMAT_R24_UNORM_X8_TYPELESS;
    case DXGI_FORMAT_R32G8X24_TYPELESS:
        return DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS;
    case DXGI_FORMAT_R32_TYPELESS:
        return DXGI_FORMAT_R32_FLOAT;
    case DXGI_FORMAT_R16_TYPELESS:
        return DXGI_FORMAT_R16_UNORM;
    default:
        return DXGI_FORMAT_UNKNOWN;
    }
}

void write_ppm(const std::string &path, const uint8_t *src, UINT pitch, UINT w, UINT h, DXGI_FORMAT view)
{
    std::string out = "P6\n" + std::to_string(w) + " " + std::to_string(h) + "\n255\n";
    size_t at = out.size();
    out.resize(at + size_t(w) * h * 3);
    for (UINT y = 0; y < h; y++)
        for (UINT x = 0; x < w; x++) {
            uint32_t v;
            memcpy(&v, src + size_t(y) * pitch + size_t(x) * 4, 4);
            uint8_t *o = reinterpret_cast<uint8_t *>(&out[at + (size_t(y) * w + x) * 3]);
            if (view == DXGI_FORMAT_R10G10B10A2_UNORM) {
                o[0] = uint8_t((v & 1023) >> 2), o[1] = uint8_t(((v >> 10) & 1023) >> 2), o[2] = uint8_t(((v >> 20) & 1023) >> 2);
            } else {
                o[0] = uint8_t(v), o[1] = uint8_t(v >> 8), o[2] = uint8_t(v >> 16);
            }
        }
    runtime::write_file(runtime::wide(path), out);
}

void land_capture()
{
    if (!capture_pending || capture_slot < 0 || slots[capture_slot].pending ||
        slots[capture_slot].fence->GetCompletedValue() < capture_fence)
        return;
    capture_pending = false;
    if (frame.view != DXGI_FORMAT_R8G8B8A8_UNORM && frame.view != DXGI_FORMAT_R10G10B10A2_UNORM)
        return;
    const char *which[2] = {"before", "after"};
    for (int i = 0; i < 2; i++) {
        uint8_t *mapped = nullptr;
        if (!capture_rb[i] || FAILED(capture_rb[i]->Map(0, nullptr, reinterpret_cast<void **>(&mapped))))
            continue;
        char file[64];
        snprintf(file, sizeof file, "\\postfx-%02u-%s.ppm", captures, which[i]);
        write_ppm(capture_dir + file, mapped + capture_fp.Offset, capture_fp.Footprint.RowPitch, UINT(frame.desc.Width),
                  frame.desc.Height, frame.view);
        D3D12_RANGE none{0, 0};
        capture_rb[i]->Unmap(0, &none);
    }
    swofty::log("[postfx] wrote the picture before and after the passes to %s\\postfx-%02u-{before,after}.ppm",
                capture_dir.c_str(), captures);
    captures++;
}

// The mean colour and saturation of a picture read back to the CPU, over a grid of its middle.
void grid_mean(const uint8_t *src, UINT pitch, UINT w, UINT h, DXGI_FORMAT f, double out[4])
{
    double r = 0, g = 0, bl = 0, sat = 0;
    unsigned n = 0;
    for (UINT y = h / 8; y < h * 7 / 10; y += std::max(1u, h / 40))
        for (UINT x = w / 10; x < w * 9 / 10; x += std::max(1u, w / 64)) {
            uint32_t v;
            memcpy(&v, src + size_t(y) * pitch + size_t(x) * 4, 4);
            double c[3];
            if (f == DXGI_FORMAT_R10G10B10A2_UNORM)
                c[0] = (v & 1023) / 1023.0, c[1] = ((v >> 10) & 1023) / 1023.0, c[2] = ((v >> 20) & 1023) / 1023.0;
            else
                c[0] = (v & 255) / 255.0, c[1] = ((v >> 8) & 255) / 255.0, c[2] = ((v >> 16) & 255) / 255.0;
            double mx = std::max(c[0], std::max(c[1], c[2])), mn = std::min(c[0], std::min(c[1], c[2]));
            r += c[0], g += c[1], bl += c[2], sat += mx > 0 ? (mx - mn) / mx : 0;
            n++;
        }
    n = std::max(n, 1u);
    out[0] = r / n, out[1] = g / n, out[2] = bl / n, out[3] = sat / n;
}

void read_times(Slot &s)
{
    if (!ticks_per_ms || !s.passes)
        return;
    UINT base = UINT(&s - slots) * MAX_PASSES * 2;
    D3D12_RANGE r{SIZE_T(base) * 8, SIZE_T(base + s.passes * 2) * 8};
    uint64_t *t = nullptr;
    if (FAILED(query_readback->Map(0, &r, reinterpret_cast<void **>(&t))))
        return;
    for (UINT i = 0; i < s.passes; i++) {
        Pass &p = passes[size_t(s.pass_of[i])];
        uint64_t a = t[base + i * 2], b = t[base + i * 2 + 1];
        if (b < a)
            continue;
        double ms = double(b - a) / ticks_per_ms;
        p.ms_sum += ms;
        p.ms_peak = std::max(p.ms_peak, ms);
        if (++p.ms_frames < BUDGET_FRAMES)
            continue;
        double avg = p.ms_sum / p.ms_frames;
        if (avg > p.budget_ms) {
            if (++p.over >= 3 && !p.off) {
                p.off = true;
                swofty::log("[postfx] %s's pass %s is off: it took %.3f ms of GPU time a frame, over its %.2f ms budget",
                            p.addon.c_str(), p.name.c_str(), avg, p.budget_ms);
            }
        } else
            p.over = 0;
        if (ran_total <= 4 * BUDGET_FRAMES)
            swofty::log("[postfx] %s's pass %s: %.3f ms of GPU time a frame on average, %.3f ms at most", p.addon.c_str(),
                        p.name.c_str(), avg, p.ms_peak);
        p.ms_sum = 0;
        p.ms_peak = 0;
        p.ms_frames = 0;
    }
    D3D12_RANGE none{0, 0};
    query_readback->Unmap(0, &none);
}

// Where a frame's passes are recorded: into the game's list at its HUD-less step, or into a slot's own
// list between the game's lists. The states are the resources' states at that point of the frame.
struct Where {
    ID3D12GraphicsCommandList *list;
    bool game;
    ID3D12Resource *back;
    D3D12_RESOURCE_STATES back_state;
    ID3D12Resource *hudless;
    D3D12_RESOURCE_STATES hudless_state;
    ID3D12Resource *depth;
    D3D12_RESOURCE_STATES depth_state;
    const char *path;
};

// A slot whose GPU work is done. The game records frames ahead, so the slots of lists it has not
// submitted yet are skipped rather than waited for. A slot recorded into a game list that was never
// submitted is free again after two seconds: the game reset that list, so the core's work in it never
// runs.
int pick_slot()
{
    ULONGLONG now = GetTickCount64();
    for (UINT n = 0; n < FRAMES; n++) {
        UINT k = (slot_at + n) % FRAMES;
        Slot &s = slots[k];
        if (s.pending && now - s.armed_at > 2000) {
            swofty::log("[postfx] a game list holding the passes was never submitted; its slot is reused");
            s.pending = false;
        }
        if (s.pending || s.fence->GetCompletedValue() < s.fence_value)
            continue;
        slot_at = (k + 1) % FRAMES;
        return int(k);
    }
    if (!warned_busy)
        swofty::log("[postfx] every one of the %u slots is in flight; passes skip frames until one is free", FRAMES);
    warned_busy = true;
    stats.busy++;
    trace.why = "every slot is in flight";
    return -1;
}

bool addon_off(const std::string &id)
{
    return std::any_of(toggles.begin(), toggles.end(), [&](const Toggle &t) { return t.off && t.addon_id == id; });
}

std::vector<int> live_passes(const view::State &v)
{
    std::vector<int> live;
    for (size_t i = 0; i < passes.size() && live.size() < MAX_PASSES; i++) {
        Pass &p = passes[i];
        const char *why = p.off            ? "off"
                          : !p.on          ? "its option turns it off"
                          : addon_off(p.addon_id) ? "its addon's key turned it off"
                                           : postfx::ineligible(p.when, v.kind, v.flags, v.valid);
        if (!why) {
            live.push_back(int(i));
            continue;
        }
        if (!p.off && trace_left)
            trace.skipped += (trace.skipped.empty() ? "" : ", ") + p.name + " (" + why + ")";
    }
    return live;
}

ID3D12Resource *input_res(int in, int k, DXGI_FORMAT &f)
{
    f = frame.view;
    if (in == IN_ORIGINAL)
        return frame.work[k].orig;
    if (in == IN_PREVIOUS)
        return frame.prev;
    if (in >= 0 && size_t(in) < textures.size()) {
        f = textures[size_t(in)].format;
        return tex_of(size_t(in), k);
    }
    f = DXGI_FORMAT_R8G8B8A8_UNORM;
    return nullptr;
}

void upload(ID3D12GraphicsCommandList *l, Tex &t)
{
    D3D12_RESOURCE_DESC d = t.res->GetDesc();
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT fp{};
    UINT rows = 0;
    UINT64 row_bytes = 0, total = 0;
    device->GetCopyableFootprints(&d, 0, 1, 0, &fp, &rows, &row_bytes, &total);
    if (!t.upload && !(t.upload = buffer(D3D12_HEAP_TYPE_UPLOAD, total, D3D12_RESOURCE_STATE_GENERIC_READ)))
        return;
    uint8_t *mapped = nullptr;
    if (FAILED(t.upload->Map(0, nullptr, reinterpret_cast<void **>(&mapped))))
        return;
    for (UINT y = 0; y < t.height; y++)
        memcpy(mapped + fp.Offset + size_t(y) * fp.Footprint.RowPitch, t.pixels.data() + size_t(y) * t.width * 4,
               size_t(t.width) * 4);
    t.upload->Unmap(0, nullptr);
    D3D12_TEXTURE_COPY_LOCATION to{}, from{};
    to.pResource = t.res;
    to.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    from.pResource = t.upload;
    from.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    from.PlacedFootprint = fp;
    transition(l, t.res, HOME, D3D12_RESOURCE_STATE_COPY_DEST);
    l->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
    transition(l, t.res, D3D12_RESOURCE_STATE_COPY_DEST, HOME);
    t.uploaded = true;
    t.pixels.clear();
    t.pixels.shrink_to_fit();
    retire({t.upload});
    t.upload = nullptr;
}

void capture_copy(ID3D12GraphicsCommandList *l, ID3D12Resource *back, int which)
{
    D3D12_TEXTURE_COPY_LOCATION rb{}, from{};
    rb.pResource = capture_rb[which];
    rb.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    rb.PlacedFootprint = capture_fp;
    from.pResource = back;
    from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    l->CopyTextureRegion(&rb, 0, 0, 0, &from, nullptr);
}

// Records this frame's passes at w into slot k's tables. Returns false when nothing was recorded.
bool record(const Where &w, int k)
{
    if (!device && !init(w.back)) {
        broken = true;
        return false;
    }
    if (device->GetDeviceRemovedReason() != S_OK) {
        broken = true;
        swofty::log("[postfx] the GPU device was removed; post-processing is off");
        return false;
    }
    land_capture();
    free_retired();
    Slot &s = slots[k];
    read_times(s);
    s.passes = 0;
    D3D12_RESOURCE_DESC d = w.back->GetDesc();
    if (!frame.heap || d.Width != frame.desc.Width || d.Height != frame.desc.Height || d.Format != frame.desc.Format)
        if (!make_frame(d)) {
            broken = true;
            return false;
        }
    if (!ensure_work(k)) {
        broken = true;
        return false;
    }
    Work &wk = frame.work[k];
    view::State v = view::current();
    trace.view = v;
    std::vector<int> live = live_passes(v);
    trace.live = unsigned(live.size());
    if (live.empty()) {
        trace.why = "every pass is off or not for this view";
        return false;
    }
    d3d12hooks::ListState saved;
    if (w.game && !d3d12hooks::list_state(w.list, saved)) {
        stats.unknown_state++;
        trace.why = "the game's list has state from before the hooks";
        return false;
    }
    ID3D12GraphicsCommandList *l = w.list;
    ULONGLONG now = GetTickCount64();
    if (!capture_dir.empty() && now - capture_checked >= 1000) {
        capture_checked = now;
        if (GetFileAttributesW(capture_file.c_str()) != INVALID_FILE_ATTRIBUTES) {
            DeleteFileW(capture_file.c_str());
            capture_asked = true;
        }
    }
    capture_now = !capture_dir.empty() && !capture_pending && capture_rb[0] && capture_rb[1] &&
                  (capture_asked || (captures < 6 && ran_total && now >= capture_next));
    if (frame.prev_serial != v.serial) {
        frame.prev_valid = false;
        for (auto &t : textures)
            t.clear |= t.persistent;
    }
    bool has_previous =
        frame.prev && frame.prev_valid && frame.prev_serial == v.serial && frame_count - frame.prev_frame <= 1;
    UINT pw = UINT(d.Width), ph = d.Height;

    transition(l, w.back, w.back_state, D3D12_RESOURCE_STATE_COPY_SOURCE);
    transition(l, wk.orig, HOME, D3D12_RESOURCE_STATE_COPY_DEST);
    l->CopyResource(wk.orig, w.back);
    transition(l, wk.orig, D3D12_RESOURCE_STATE_COPY_DEST, HOME);
    for (auto &t : textures)
        if (!t.file.empty() && !t.uploaded && t.res)
            upload(l, t);
    l->SetComputeRootSignature(root);
    l->SetDescriptorHeaps(1, &frame.heap);
    UINT clear_at = FRAMES * MAX_PASSES * TABLE, ntex = UINT(textures.size());
    for (size_t i = 0; i < textures.size(); i++) {
        Tex &t = textures[i];
        bool own = !shared(t);
        ID3D12Resource *r = tex_of(i, k);
        if (!r || !(own ? wk.clear : t.clear))
            continue;
        UINT at = own ? ntex * (1 + UINT(k)) + UINT(i) : UINT(i);
        uav(r, t.format, cpu_at(frame.heap, clear_at + at));
        uav(r, t.format, cpu_at(frame.cpu, at));
        transition(l, r, HOME, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
        const float zero[4] = {};
        l->ClearUnorderedAccessViewFloat(gpu_at(frame.heap, clear_at + at), cpu_at(frame.cpu, at), r, zero, 0, nullptr);
        transition(l, r, D3D12_RESOURCE_STATE_UNORDERED_ACCESS, HOME);
        if (!own)
            t.clear = false;
    }
    wk.clear = false;

    uint32_t c[CONSTANTS] = {};
    float t_now = float(double(now - started) / 1000.0);
    memcpy(&c[4], &t_now, 4);
    c[5] = frame_count;
    c[6] = format_kind(frame.view);
    memcpy(&c[27], &frametime, 4);
    c[28] = pw;
    c[29] = ph;
    c[30] = frame_count * 2654435761u + 0x9e3779b9u;
    c[31] = v.kind;
    c[32] = v.flags;
    c[33] = v.valid;
    c[34] = has_previous;
    ID3D12Resource *depth = nullptr;
    DXGI_FORMAT depth_fmt = DXGI_FORMAT_R32_FLOAT;
    if (w.depth && (w.depth_state & (D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE | D3D12_RESOURCE_STATE_DEPTH_WRITE))) {
        D3D12_RESOURCE_DESC dd = w.depth->GetDesc();
        DXGI_FORMAT f = depth_view(dd.Format);
        if (f != DXGI_FORMAT_UNKNOWN && dd.SampleDesc.Count == 1 && !(dd.Flags & D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE)) {
            depth = w.depth;
            depth_fmt = f;
            c[24] = UINT(dd.Width);
            c[25] = dd.Height;
            c[26] = 1;
        }
    }
    if (depth && !depth_logged) {
        depth_logged = true;
        swofty::log("[postfx] passes read the game's depth buffer (%ux%u, format %d) as t1", c[24], c[25],
                    int(depth->GetDesc().Format));
    }
    D3D12_RESOURCE_STATES depth_read = D3D12_RESOURCE_STATE_DEPTH_READ | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;
    bool depth_moved = depth && !(w.depth_state & D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    if (depth_moved)
        transition(l, depth, w.depth_state, depth_read);
    bool timed = queries && query_readback;
    UINT qbase = UINT(k) * MAX_PASSES * 2;
    ID3D12Resource *cur = wk.orig;
    int next = 0;
    for (int index : live) {
        Pass &p = passes[size_t(index)];
        ID3D12Resource *out = p.output < 0 ? wk.pic[next] : tex_of(size_t(p.output), k);
        DXGI_FORMAT of = p.output < 0 ? frame.view : textures[size_t(p.output)].format;
        UINT ow = p.output < 0 ? pw : textures[size_t(p.output)].w, oh = p.output < 0 ? ph : textures[size_t(p.output)].h;
        UINT at = (UINT(k) * MAX_PASSES + s.passes) * TABLE;
        srv(cur, frame.view, cpu_at(frame.heap, at));
        srv(depth, depth_fmt, cpu_at(frame.heap, at + 1));
        for (UINT i = 0; i < INPUTS; i++) {
            DXGI_FORMAT f;
            ID3D12Resource *r = input_res(p.inputs[i], k, f);
            srv(r, f, cpu_at(frame.heap, at + 2 + i));
        }
        uav(out, of, cpu_at(frame.heap, at + TABLE - 1));
        transition(l, out, HOME, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
        c[0] = ow;
        c[1] = oh;
        float texel[2] = {1.0f / float(ow), 1.0f / float(oh)};
        memcpy(&c[2], texel, 8);
        c[7] = s.passes;
        memcpy(&c[8], p.options, sizeof p.options);
        if (timed)
            l->EndQuery(queries, D3D12_QUERY_TYPE_TIMESTAMP, qbase + s.passes * 2);
        l->SetPipelineState(p.pso);
        l->SetComputeRoot32BitConstants(0, CONSTANTS, c, 0);
        l->SetComputeRootDescriptorTable(1, gpu_at(frame.heap, at));
        l->Dispatch(postfx::groups(ow, p.threads[0]), postfx::groups(oh, p.threads[1]), 1);
        if (timed)
            l->EndQuery(queries, D3D12_QUERY_TYPE_TIMESTAMP, qbase + s.passes * 2 + 1);
        transition(l, out, D3D12_RESOURCE_STATE_UNORDERED_ACCESS, HOME);
        s.pass_of[s.passes++] = index;
        if (p.output < 0) {
            cur = out;
            next = 1 - next;
        }
    }
    if (depth_moved)
        transition(l, depth, depth_read, w.depth_state);
    if (timed)
        l->ResolveQueryData(queries, D3D12_QUERY_TYPE_TIMESTAMP, qbase, s.passes * 2, query_readback, UINT64(qbase) * 8);

    transition(l, cur, HOME, D3D12_RESOURCE_STATE_COPY_SOURCE);
    if (cur != wk.orig) {
        transition(l, w.back, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COPY_DEST);
        l->CopyResource(w.back, cur);
        transition(l, w.back, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE);
    }
    if (capture_now)
        capture_copy(l, w.back, 1);
    transition(l, w.back, D3D12_RESOURCE_STATE_COPY_SOURCE, w.back_state);
    if (w.hudless && cur != wk.orig && fg_on.load()) {
        D3D12_RESOURCE_DESC hd = w.hudless->GetDesc();
        if (hd.Width == d.Width && hd.Height == d.Height && view_format(hd.Format) == frame.view) {
            transition(l, w.hudless, w.hudless_state, D3D12_RESOURCE_STATE_COPY_DEST);
            l->CopyResource(w.hudless, cur);
            transition(l, w.hudless, D3D12_RESOURCE_STATE_COPY_DEST, w.hudless_state);
            if (!hudless_logged)
                swofty::log("[postfx] the game's HUD-less copy of the picture gets the passes too, for Frame Generation");
            hudless_logged = true;
        }
    }
    if (frame.prev) {
        transition(l, frame.prev, HOME, D3D12_RESOURCE_STATE_COPY_DEST);
        l->CopyResource(frame.prev, cur);
        transition(l, frame.prev, D3D12_RESOURCE_STATE_COPY_DEST, HOME);
        frame.prev_valid = true;
        frame.prev_serial = v.serial;
        frame.prev_frame = frame_count;
    }
    transition(l, cur, D3D12_RESOURCE_STATE_COPY_SOURCE, HOME);
    if (w.game)
        d3d12hooks::restore_list_state(l, saved);

    s.pending = true;
    s.armed_at = now;
    if (capture_now) {
        capture_pending = true;
        capture_asked = false;
        capture_slot = k;
        capture_next = now + 10000;
    }
    trace.runs++;
    trace.ran = s.passes;
    trace.path = w.path;
    trace.back = w.back;
    trace.why = nullptr;
    if (++ran_total == 1) {
        capture_next = now + 30000;
        swofty::log("[postfx] the picture now goes through %u pass(es) before the HUD, %s", s.passes,
                    w.game ? "inside the game's list at its HUD-less step" : "in a list between the game's lists");
    }
    return true;
}

// The list of the core's own for a frame whose picture list ends with the back buffer still a render
// target and draws no HUD.
void run_between(ID3D12CommandQueue *q, ID3D12Resource *back, D3D12_RESOURCE_STATES state, ID3D12Resource *hudless,
                 D3D12_RESOURCE_STATES hudless_state)
{
    if (!device && !init(back)) {
        broken = true;
        return;
    }
    int k = pick_slot();
    if (k < 0)
        return;
    Slot &s = slots[k];
    if (FAILED(s.alloc->Reset()) || FAILED(s.list->Reset(s.alloc, nullptr))) {
        broken = true;
        swofty::log("[postfx] could not record the passes' command list; post-processing is off");
        return;
    }
    Where w{s.list, false, back, state, hudless, hudless_state, depth_res, depth_state, "between"};
    bool made = record(w, k);
    if (FAILED(s.list->Close())) {
        broken = true;
        s.pending = false;
        swofty::log("[postfx] the passes' command list did not close; post-processing is off");
        return;
    }
    if (!made)
        return;
    read_frequency(q);
    ID3D12CommandList *one[] = {s.list};
    d3d12hooks::exec_original(q, 1u, one);
    q->Signal(s.fence, ++s.fence_value);
    s.pending = false;
    if (capture_slot == k)
        capture_fence = s.fence_value;
    stats.between++;
}

void log_diag(ID3D12GraphicsCommandList *list, const Mark &m)
{
    swofty::log("[postfx] the first tonemap list %p has %zu transitions; back buffer %p ends in state 0x%x; %s",
                static_cast<void *>(list), m.seen.size(), static_cast<void *>(m.target), unsigned(m.state),
                m.ui ? "it draws UI too" : "the UI comes in a later list");
    for (auto &b : m.seen) {
        D3D12_RESOURCE_DESC d = b.Transition.pResource->GetDesc();
        swofty::log("[postfx]   %s %llux%u fmt %d: 0x%x -> 0x%x", name_of(b.Transition.pResource).c_str(),
                    (unsigned long long)d.Width, d.Height, int(d.Format), unsigned(b.Transition.StateBefore),
                    unsigned(b.Transition.StateAfter));
    }
}

bool interesting(LPCWSTR name)
{
    if (!wcsstr(name, L"eaid://") || wcsncmp(name, UI_PREFIX, wcslen(UI_PREFIX)) == 0)
        return false;
    std::wstring lower(name);
    for (auto &c : lower)
        c = wchar_t(towlower(c));
    for (auto *k : INTERESTING)
        if (lower.find(k) != std::wstring::npos)
            return true;
    return false;
}

void log_named(ID3D12Object *o, LPCWSTR name)
{
    if (named_seen.size() >= NAMED_LINES ||
        std::find(named_seen.begin(), named_seen.end(), name) != named_seen.end())
        return;
    ID3D12Resource *res = nullptr;
    if (FAILED(o->QueryInterface(__uuidof(ID3D12Resource), reinterpret_cast<void **>(&res))) || !res)
        return;
    D3D12_RESOURCE_DESC d = res->GetDesc();
    res->Release();
    if (d.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D)
        return;
    named_seen.emplace_back(name);
    swofty::log("[postfx] diag: named texture %s %llux%u fmt %d flags 0x%x", runtime::narrow(name).c_str(),
                (unsigned long long)d.Width, d.Height, int(d.Format), unsigned(d.Flags));
}

void on_setname(ID3D12Object *o, LPCWSTR name)
{
    static const uint32_t probe = perf::probe("core", "postfx SetName");
    if (!name)
        return;
    perf::Scope p(probe);
    auto *res = reinterpret_cast<ID3D12Resource *>(o);
    std::lock_guard<std::recursive_mutex> g(lock);
    if (watched_bits(res)) {
        names.erase(res);
        watch(res, 0);
    }
    if (res == depth_res)
        depth_res = nullptr;
    bool tonemap = wcscmp(name, TONEMAP_NAME) == 0, back = wcscmp(name, BACKBUFFER_NAME) == 0;
    uint8_t bits = (tonemap ? W_TONEMAP : 0) | (back ? W_BACK : 0) | (wcscmp(name, HUDLESS_NAME) == 0 ? W_HUDLESS : 0) |
                   (wcscmp(name, DEPTH_NAME) == 0 ? W_DEPTH : 0) |
                   (wcsncmp(name, UI_PREFIX, wcslen(UI_PREFIX)) == 0 ? W_UI : 0);
    if (tonemap || back || (!diag_done && wcsstr(name, L"eaid://")))
        names[res] = name;
    if (back && !saw_back) {
        saw_back = true;
        swofty::log("[postfx] the game names its back buffers \"BackBuffer\"");
    }
    if (tonemap && !tonemap_named) {
        tonemap_named = true;
        swofty::log("[postfx] the game named its tonemap output %p", static_cast<void *>(res));
    }
    if (interesting(name))
        log_named(o, name);
    if (bits)
        watch(res, bits);
}

// While the game reads the finished picture for its HUD (on track), that read is the one place the
// passes run; without such reads (menus) they run at the HUD-less step or between the game's lists.
bool hud_reads()
{
    return hud_read_seen && frame_count - hud_read_at <= 30;
}

// Marks what each game list does with the watched resources. At the HUD-less step (the HUD-less copy
// leaves UAV while this list has made the back buffer a render target) the passes are recorded into the
// list, before the game's barrier, with every state as it was before this batch.
bool on_barrier(ID3D12GraphicsCommandList *list, UINT n, const D3D12_RESOURCE_BARRIER *barriers)
{
    static const uint32_t probe = perf::probe("core", "postfx ResourceBarrier");
    if (!barriers || !n || broken)
        return false;
    perf::Scope p(probe);
    bool look = false;
    for (UINT i = 0; i < n && !look; i++)
        look = barriers[i].Type == D3D12_RESOURCE_BARRIER_TYPE_TRANSITION && watched_bits(barriers[i].Transition.pResource);
    if (!look)
        return false;
    static const uint32_t marking = perf::probe("core", "postfx ResourceBarrier (watched)");
    perf::Scope m_scope(marking);
    std::lock_guard<std::recursive_mutex> g(lock);
    auto it = marks.find(list);
    Mark *m = it == marks.end() ? nullptr : &it->second;
    ID3D12Resource *hudless = nullptr, *leaving = nullptr;
    D3D12_RESOURCE_STATES hudless_now{};
    bool moves_back = false;
    for (UINT i = 0; i < n; i++) {
        const D3D12_RESOURCE_BARRIER &b = barriers[i];
        if (b.Type != D3D12_RESOURCE_BARRIER_TYPE_TRANSITION)
            continue;
        uint8_t c = watched_bits(b.Transition.pResource);
        moves_back |= (c & W_BACK) != 0;
        if ((c & W_HUDLESS) && (b.Transition.StateBefore & D3D12_RESOURCE_STATE_UNORDERED_ACCESS) &&
            b.Flags == D3D12_RESOURCE_BARRIER_FLAG_NONE) {
            hudless = b.Transition.pResource;
            hudless_now = b.Transition.StateBefore;
        }
        if ((c & W_BACK) && b.Transition.StateBefore == D3D12_RESOURCE_STATE_RENDER_TARGET &&
            (b.Transition.StateAfter & READ_STATES) && b.Flags == D3D12_RESOURCE_BARRIER_FLAG_NONE && !leaving)
            leaving = b.Transition.pResource;
    }
    if (final_left && device && m && m->target) {
        for (UINT i = 0; i < n; i++) {
            const D3D12_RESOURCE_BARRIER &b = barriers[i];
            if (b.Type != D3D12_RESOURCE_BARRIER_TYPE_TRANSITION || b.Transition.pResource != m->target ||
                b.Transition.StateBefore != D3D12_RESOURCE_STATE_RENDER_TARGET || b.Transition.StateAfter != 0)
                continue;
            FinalProbe &fp = finals[final_at % 256];
            if (fp.pending)
                break;
            D3D12_RESOURCE_DESC d = m->target->GetDesc();
            if (!fp.rb) {
                D3D12_RESOURCE_DESC box = d;
                box.Width = final_w = UINT(d.Width) * 8 / 10;
                box.Height = final_h = d.Height * 6 / 10;
                UINT rows = 0;
                UINT64 row_bytes = 0, total = 0;
                device->GetCopyableFootprints(&box, 0, 1, 0, &final_fp, &rows, &row_bytes, &total);
                fp.rb = buffer(D3D12_HEAP_TYPE_READBACK, total, D3D12_RESOURCE_STATE_COPY_DEST);
            }
            if (!fp.rb)
                break;
            transition(list, m->target, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE);
            D3D12_TEXTURE_COPY_LOCATION to{}, from{};
            to.pResource = fp.rb;
            to.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
            to.PlacedFootprint = final_fp;
            from.pResource = m->target;
            from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
            D3D12_BOX src{UINT(d.Width) / 10, d.Height / 8, 0, UINT(d.Width) / 10 + final_w, d.Height / 8 + final_h, 1};
            list->CopyTextureRegion(&to, 0, 0, 0, &from, &src);
            transition(list, m->target, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_RENDER_TARGET);
            fp.pending = true;
            fp.frame = frame_count;
            fp.land = frame_count + 60;
            final_at++;
            final_left--;
            break;
        }
    }
    // The game records a frame's lists in parallel, so the core cannot tell at one place whether another
    // place of the same frame runs the passes too. On track the game reads the finished picture once
    // per frame for its HUD, as the first back buffer barrier of the HUD's list (or the first one that
    // leaves the render target in the list that drew it): the passes run there and nowhere else. In
    // menus there is no such read; the passes run at the HUD-less step of the list that made the back
    // buffer a render target, once per back buffer and frame.
    if ((hudless || leaving) && state_armed && list->GetType() == D3D12_COMMAND_LIST_TYPE_DIRECT && !killed_now() &&
        !passes.empty()) {
        Where w{list, true, nullptr, D3D12_RESOURCE_STATE_RENDER_TARGET, hudless, hudless_now, nullptr,
                D3D12_RESOURCE_STATE_COMMON, "in-list"};
        if (leaving && (!m || !m->left_rt)) {
            BackUse &u = back_use(leaving);
            hud_read_seen = true;
            hud_read_at = frame_count;
            w.back = leaving;
            w.hudless = m ? m->hudless : nullptr;
            w.hudless_state = m ? m->hudless_state : D3D12_RESOURCE_STATE_COMMON;
            w.depth = m && m->depth ? m->depth : u.depth;
            w.depth_state = m && m->depth ? m->depth_state : u.depth_state;
            w.path = "in-list (HUD read)";
        } else if (hudless && !hud_reads() && !moves_back && m && m->target &&
                   m->state == D3D12_RESOURCE_STATE_RENDER_TARGET && m->slot < 0 && back_use(m->target).fresh) {
            w.back = m->target;
            w.depth = m->depth;
            w.depth_state = m->depth_state;
        }
        if (w.back) {
            back_use(w.back).fresh = false;
            if (!device && !init(w.back)) {
                broken = true;
                return false;
            }
            int k = pick_slot();
            if (k >= 0 && record(w, k)) {
                if (!m)
                    m = &marks[list];
                m->slot = k;
                m->ran_back = w.back;
                stats.in_list++;
                if (order_left) {
                    char at[24];
                    snprintf(at, sizeof at, " [PASSES@%04x]", unsigned(uintptr_t(w.back) & 0xffff));
                    m->order += at;
                }
            }
        }
    }
    for (UINT i = 0; i < n; i++) {
        const D3D12_RESOURCE_BARRIER &b = barriers[i];
        if (b.Type != D3D12_RESOURCE_BARRIER_TYPE_TRANSITION)
            continue;
        ID3D12Resource *r = b.Transition.pResource;
        uint8_t c = watched_bits(r);
        bool tonemap = c & W_TONEMAP, back = c & W_BACK, depth = c & W_DEPTH;
        if (!m) {
            if (!tonemap && !back && !depth && !(c & W_HUDLESS))
                continue;
            m = &marks[list];
        }
        if (depth) {
            m->depth = r;
            m->depth_state = b.Transition.StateAfter;
        }
        if (c & W_UI)
            m->ui = true;
        if (c & W_HUDLESS) {
            m->hudless = r;
            m->hudless_state = b.Transition.StateAfter;
            if ((b.Transition.StateAfter & D3D12_RESOURCE_STATE_UNORDERED_ACCESS) && m->target &&
                m->state == D3D12_RESOURCE_STATE_RENDER_TARGET) {
                BackUse &u = back_use(m->target);
                u.depth = m->depth;
                u.depth_state = m->depth_state;
            }
        }
        if (!diag_done && m->seen.size() < 96)
            m->seen.push_back(b);
        if (order_left && m->order.size() < 3000) {
            char step[64];
            snprintf(step, sizeof step, " %s%s%s%s%s:%x>%x%s", tonemap ? "T" : "", back ? "B" : "", depth ? "D" : "",
                     (c & W_HUDLESS) ? "H" : "", (c & W_UI) ? "U" : "", unsigned(b.Transition.StateBefore),
                     unsigned(b.Transition.StateAfter), b.Flags ? "s" : "");
            if (back)
                snprintf(step + strlen(step), sizeof step - strlen(step), "@%04x", unsigned(uintptr_t(r) & 0xffff));
            m->order += step;
        }
        if (tonemap &&
            (b.Transition.StateBefore & (D3D12_RESOURCE_STATE_RENDER_TARGET | D3D12_RESOURCE_STATE_UNORDERED_ACCESS)) &&
            (b.Transition.StateAfter & READ_STATES))
            m->tonemap = true;
        if (back) {
            if (b.Transition.StateBefore == D3D12_RESOURCE_STATE_RENDER_TARGET &&
                (b.Transition.StateAfter & READ_STATES))
                m->left_rt = true;
            m->target = r;
            m->state = b.Transition.StateAfter;
            if (b.Transition.StateAfter == D3D12_RESOURCE_STATE_RENDER_TARGET &&
                !(b.Transition.StateBefore & (READ_STATES | D3D12_RESOURCE_STATE_RENDER_TARGET))) {
                back_use(r).fresh = true;
            }
        }
    }
    return false;
}

// A list reset before it was submitted throws its commands away, the core's passes with them: their
// slot is free again and the frame's back buffer still waits for them.
void on_reset(ID3D12GraphicsCommandList *list)
{
    std::lock_guard<std::recursive_mutex> g(lock);
    auto it = marks.find(list);
    if (it == marks.end())
        return;
    if (it->second.slot >= 0) {
        slots[it->second.slot].pending = false;
        if (it->second.ran_back)
            back_use(it->second.ran_back).fresh = true;
        stats.dropped++;
    }
    marks.erase(it);
}

// Submits the game's lists, the core's own list after a picture list that ended before the HUD, and a
// fence signal after any list the passes were recorded into, so their slot is known done.
bool on_exec(ID3D12CommandQueue *q, UINT n, ID3D12CommandList *const *lists)
{
    static const uint32_t probe = perf::probe("core", "postfx ExecuteCommandLists");
    if (!lists || !n || broken)
        return false;
    perf::Scope p(probe);
    UINT split = n;
    ID3D12Resource *res = nullptr, *hudless = nullptr, *later_depth = nullptr;
    D3D12_RESOURCE_STATES state{}, hudless_state{}, later_depth_state{};
    int signal[FRAMES];
    UINT signals = 0;
    {
        std::lock_guard<std::recursive_mutex> g(lock);
        for (UINT i = 0; i < n && !marks.empty(); i++) {
            auto it = marks.find(reinterpret_cast<ID3D12GraphicsCommandList *>(lists[i]));
            if (it == marks.end())
                continue;
            Mark &m = it->second;
            if (m.slot >= 0 && signals < FRAMES)
                signal[signals++] = m.slot;
            if (m.tonemap && !diag_done) {
                log_diag(reinterpret_cast<ID3D12GraphicsCommandList *>(lists[i]), m);
                diag_done = true;
            }
            if (m.tonemap && m.slot < 0 && split == n && !hud_reads() && !killed_now() && !passes.empty()) {
                if (m.ui) {
                    stats.ui_together++;
                    trace.why = "the picture and the HUD are in one list without a HUD-less step";
                } else if (m.target && m.state == D3D12_RESOURCE_STATE_RENDER_TARGET && back_use(m.target).fresh) {
                    // The picture is in the back buffer; when the list ends with the HUD-less step still
                    // open, the rest of that step runs on another queue and leaves the back buffer alone.
                    stats.deferred += m.hudless && (m.hudless_state & D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
                    back_use(m.target).fresh = false;
                    split = i;
                    res = m.target;
                    state = m.state;
                    hudless = m.hudless;
                    hudless_state = m.hudless_state;
                }
            }
            if (m.depth) {
                if (split == n || i <= split) {
                    depth_res = m.depth;
                    depth_state = m.depth_state;
                } else {
                    later_depth = m.depth;
                    later_depth_state = m.depth_state;
                }
            }
            if (order_left && m.target && !m.order.empty()) {
                order_left--;
                swofty::log("[postfx] order: list %p of %u in this submission%s:%s%s", static_cast<void *>(lists[i]), n,
                            m.tonemap ? " (picture)" : "", m.order.c_str(), split == i ? " [BETWEEN after this list]" : "");
            }
            marks.erase(it);
        }
    }
    if (split == n && !signals) {
        if (later_depth) {
            std::lock_guard<std::recursive_mutex> g(lock);
            depth_res = later_depth;
            depth_state = later_depth_state;
        }
        return false;
    }
    {
        perf::Pause game;
        d3d12hooks::exec_original(q, split == n ? n : split + 1, lists);
    }
    std::lock_guard<std::recursive_mutex> g(lock);
    if (split < n)
        run_between(q, res, state, hudless, hudless_state);
    if (later_depth) {
        depth_res = later_depth;
        depth_state = later_depth_state;
    }
    if (split + 1 < n) {
        perf::Pause game;
        d3d12hooks::exec_original(q, n - split - 1, lists + split + 1);
    }
    if (signals)
        read_frequency(q);
    for (UINT i = 0; i < signals; i++) {
        Slot &s = slots[signal[i]];
        q->Signal(s.fence, ++s.fence_value);
        s.pending = false;
        if (capture_slot == signal[i])
            capture_fence = s.fence_value;
    }
    return true;
}

// Frame generation from the game's hardware settings, read every two seconds; on while unknown.
bool frame_gen_on()
{
    static ULONGLONG checked;
    static bool on = true;
    static std::wstring hardware_settings;
    ULONGLONG now = GetTickCount64();
    if (checked && now - checked < 2000)
        return on;
    checked = now;
    if (hardware_settings.empty()) {
        wchar_t docs[MAX_PATH] = {};
        using GetFolder = HRESULT(WINAPI *)(HWND, int, HANDLE, DWORD, LPWSTR);
        HMODULE shell = LoadLibraryW(L"shell32.dll");
        auto get = shell ? reinterpret_cast<GetFolder>(GetProcAddress(shell, "SHGetFolderPathW")) : nullptr;
        if (get && SUCCEEDED(get(nullptr, 0x0005, nullptr, 0, docs)))
            hardware_settings = std::wstring(docs) + L"\\My Games\\F1 25\\hardwaresettings\\hardware_settings_config.xml";
    }
    on = hud::guard::fg_setting(runtime::read_file(hardware_settings, size_t(1) << 20)).on();
    return on;
}

bool setting_value(const std::string &addon, const std::string &option, std::string &text)
{
    size_t current = 0;
    std::vector<std::string> values;
    if (!settings_state(addon, option, current, nullptr, nullptr, &values) || current >= values.size())
        return false;
    text = values[current];
    return true;
}

// Options the passes read live, their on/off options and the addons' toggle keys, from the settings
// ADDON SETTINGS changes in game.
void refresh_live()
{
    std::string text;
    float v = 0;
    for (auto &p : passes) {
        for (auto &l : p.live)
            if (l.slot < 16 && setting_value(p.addon_id, l.option, text) && postfx::option_value(text, v))
                p.options[l.slot] = v;
        if (!p.enabled.empty() && setting_value(p.addon_id, p.enabled, text) && postfx::option_value(text, v))
            p.on = v != 0;
    }
    for (auto &t : toggles)
        if (setting_value(t.addon_id, t.option, text))
            t.vk = postfx::key_code(text);
}

bool game_in_front()
{
    DWORD pid = 0;
    GetWindowThreadProcessId(GetForegroundWindow(), &pid);
    return pid == GetCurrentProcessId();
}

void poll_toggles()
{
    bool front = game_in_front();
    for (auto &t : toggles) {
        bool down = t.vk && front && (GetAsyncKeyState(t.vk) & 0x8000);
        if (down && !t.down) {
            t.off = !t.off;
            swofty::log("[postfx] %s's passes are %s (its key)", t.addon.c_str(), t.off ? "off" : "on");
            runtime::hud_toast(t.off ? "SHADERS OFF" : "SHADERS ON", t.addon, 2.0f, t.addon_id);
        }
        t.down = down;
    }
}

void log_trace()
{
    if (!trace_left)
        return;
    trace_left--;
    swofty::log("[postfx] trace present=%u path=%s runs=%u passes=%u/%u view=%s flags=%s valid=%d serial=%u fg=%s "
                "back=%p%s%s%s%s",
                trace_present++, trace.path, trace.runs, trace.ran, trace.live,
                std::string(postfx::view_text(trace.view.kind)).c_str(), postfx::flag_text(trace.view.flags).c_str(),
                int(trace.view.valid), trace.view.serial, fg_on.load() ? "on" : "off", static_cast<void *>(trace.back),
                trace.why ? " why=" : "", trace.why ? trace.why : "", trace.skipped.empty() ? "" : " skipped=",
                trace.skipped.c_str());
}

void check_trace_file(ULONGLONG now)
{
    if (now - trace_checked < 1000)
        return;
    trace_checked = now;
    std::wstring order_file = swofty_root_dir + L"postfx.order";
    if (GetFileAttributesW(order_file.c_str()) != INVALID_FILE_ATTRIBUTES) {
        std::string text = runtime::read_file(order_file, 64);
        DeleteFileW(order_file.c_str());
        unsigned lists = unsigned(std::strtoul(text.c_str(), nullptr, 10));
        order_left = lists ? lists : 200;
        swofty::log("[postfx] logging the barrier order of the next %u list(s) that draw the picture", order_left);
    }
    std::wstring burst_file = swofty_root_dir + L"postfx.burst";
    if (GetFileAttributesW(burst_file.c_str()) != INVALID_FILE_ATTRIBUTES && !final_left) {
        std::string text = runtime::read_file(burst_file, 64);
        DeleteFileW(burst_file.c_str());
        unsigned frames = unsigned(std::strtoul(text.c_str(), nullptr, 10));
        final_left = std::min(frames ? frames : 60u, 240u);
        swofty::log("[postfx] burst: reading back the next %u frame(s) as the game hands them to Present", final_left);
    }
    if (GetFileAttributesW(trace_file.c_str()) == INVALID_FILE_ATTRIBUTES)
        return;
    std::string text = runtime::read_file(trace_file, 64);
    DeleteFileW(trace_file.c_str());
    unsigned frames = unsigned(std::strtoul(text.c_str(), nullptr, 10));
    trace_left = frames ? frames : 600;
    swofty::log("[postfx] tracing the next %u presented frame(s)", trace_left);
}

void final_land()
{
    for (auto &fp : finals) {
        if (!fp.pending || frame_count < fp.land)
            continue;
        fp.pending = false;
        uint8_t *mapped = nullptr;
        if (FAILED(fp.rb->Map(0, nullptr, reinterpret_cast<void **>(&mapped))))
            continue;
        double m[4];
        grid_mean(mapped + final_fp.Offset, final_fp.Footprint.RowPitch, final_w, final_h, frame.view, m);
        D3D12_RANGE none{0, 0};
        fp.rb->Unmap(0, &none);
        swofty::log("[postfx] burst f=%u mean=%.4f,%.4f,%.4f sat=%.4f", fp.frame, m[0], m[1], m[2], m[3]);
    }
}

// Present closes the frame: its trace line and counts, the toggle keys, and the live settings.
void on_present(IDXGISwapChain3 *, const d3d12hooks::Swapchain *known, void *)
{
    static const uint32_t probe = perf::probe("core", "postfx Present");
    perf::Scope p(probe);
    std::lock_guard<std::recursive_mutex> g(lock);
    if (broken)
        return;
    fg_on = !known || !known->queue || !known->fg.empty() || (known->streamline && frame_gen_on());
    ULONGLONG now = GetTickCount64();
    frametime = frame_at ? float(double(now - frame_at) / 1000.0) : 0.0f;
    frame_at = now;
    frame_count++;
    stats.presented++;
    stats.without += trace.runs == 0;
    stats.doubled += trace.runs > 1;
    final_land();
    log_trace();
    trace = Trace{};
    check_trace_file(now);
    poll_toggles();
    if (now - live_at >= 250) {
        live_at = now;
        refresh_live();
    }
    if (stat_lines < STAT_LINES && now >= stat_at) {
        if (stat_at && ran_total)
            swofty::log("[postfx] last 60 s: %u frame(s) presented; passes ran in the game's list on %u and between "
                        "its lists on %u (%u after a list that left its HUD-less step to another queue; %u recorded into "
                        "lists the game reset unsubmitted); %u without passes "
                        "(%u with the picture and the HUD in one list, %u with list state from before the hooks, %u "
                        "with the GPU behind); %u frame(s) ran them twice",
                        stats.presented, stats.in_list, stats.between, stats.deferred, stats.dropped, stats.without,
                        stats.ui_together, stats.unknown_state, stats.busy, stats.doubled);
        stat_lines += stat_at != 0 && ran_total;
        stat_at = now + 60000;
        stats = Stats{};
    }
}

int texture_named(const std::string &name)
{
    if (name == "original")
        return IN_ORIGINAL;
    if (name == "previous")
        return IN_PREVIOUS;
    for (size_t i = 0; i < textures.size(); i++)
        if (textures[i].name == name)
            return int(i);
    return IN_NONE;
}

bool load_passes()
{
    const json::Value &cfg = runtime::config();
    for (auto &e : cfg["postfx_textures"].a) {
        Tex t;
        t.name = e["name"].str();
        t.file = e["file"].str();
        if (e["scale"].type == json::Value::Num)
            t.scale = e["scale"].n;
        if (e["width"].type == json::Value::Num && e["height"].type == json::Value::Num) {
            t.width = UINT(e["width"].n);
            t.height = UINT(e["height"].n);
        }
        t.persistent = e["persistent"].type == json::Value::Bool && e["persistent"].b;
        int f = postfx::format_of(e["format"].str("rgba16f"));
        t.format = f ? DXGI_FORMAT(f) : DXGI_FORMAT_R16G16B16A16_FLOAT;
        if (!t.file.empty() && !read_tex_file(t)) {
            swofty::log("[postfx] texture %s is left out: %s is missing or not a texture", t.name.c_str(), t.file.c_str());
            t.file.clear();
            t.name += " (missing)";
        }
        textures.push_back(std::move(t));
    }
    for (auto &e : cfg["postfx"].a) {
        Pass p;
        p.addon = e["addon"].str("an addon");
        p.addon_id = e["addon_id"].str();
        p.name = e["name"].str("pass");
        p.shader = e["shader"].str();
        p.enabled = e["enabled"].str();
        const json::Value &b = e["budget_ms"];
        if (b.type == json::Value::Num && b.n > 0)
            p.budget_ms = b.n;
        size_t k = 0;
        for (auto &o : e["options"].a)
            if (k < 16)
                p.options[k++] = o.type == json::Value::Num ? float(o.n) : 0.0f;
        for (auto &l : e["live"].a)
            if (l["slot"].type == json::Value::Num)
                p.live.push_back({UINT(l["slot"].n), l["option"].str()});
        const char *bad = nullptr;
        for (auto &in : e["inputs"].a) {
            int t = texture_named(in.str());
            if (t == IN_NONE || p.n_inputs >= INPUTS)
                bad = "names an input that is not there";
            else
                p.inputs[p.n_inputs++] = t;
        }
        std::string out = e["output"].str();
        if (!out.empty()) {
            p.output = texture_named(out);
            if (p.output < 0 || !textures[size_t(p.output)].file.empty())
                bad = "writes a texture that is not there or comes from a file";
            for (UINT i = 0; i < p.n_inputs; i++)
                if (p.output >= 0 && p.inputs[i] == p.output)
                    bad = "reads the texture it writes";
        }
        const auto &th = e["threads"].a;
        if (th.size() == 2 && th[0].n >= 1 && th[1].n >= 1) {
            p.threads[0] = UINT(th[0].n);
            p.threads[1] = UINT(th[1].n);
        }
        for (auto &v : e["when"]["views"].a)
            p.when.views |= postfx::view_bit(v.str());
        for (auto &f : e["when"]["skip"].a)
            p.when.skip |= postfx::flag_bit(f.str());
        std::string bytes = runtime::read_file(runtime::under_swofty(p.shader), size_t(4) << 20);
        if (bytes.size() < 32 || memcmp(bytes.data(), "DXBC", 4) != 0)
            bad = bytes.empty() ? "has no DXIL" : "is not a DXIL container";
        if (bad) {
            swofty::log("[postfx] %s's pass %s is off: it %s (%s)", p.addon.c_str(), p.name.c_str(), bad,
                        p.shader.c_str());
            continue;
        }
        p.dxil.assign(bytes.begin(), bytes.end());
        uses_previous |= std::find(p.inputs, p.inputs + p.n_inputs, IN_PREVIOUS) != p.inputs + p.n_inputs;
        passes.push_back(std::move(p));
    }
    for (auto &e : cfg["postfx_toggles"].a) {
        Toggle t;
        t.addon_id = e["addon_id"].str();
        t.option = e["option"].str();
        t.addon = t.addon_id;
        for (auto &p : passes)
            if (p.addon_id == t.addon_id)
                t.addon = p.addon;
        if (!t.addon_id.empty() && !t.option.empty())
            toggles.push_back(std::move(t));
    }
    return !passes.empty();
}

}

namespace runtime {

void postfx_setup()
{
    if (!enabled("postfx", nullptr))
        return;
    char v[260] = {};
    if (GetEnvironmentVariableA("SWOFTY_POSTFX", v, sizeof v) && v[0] == '0') {
        swofty::log("[postfx] SWOFTY_POSTFX=0; every pass is off");
        return;
    }
    off_file = swofty_root() + L"postfx.off";
    trace_file = swofty_root() + L"postfx.trace";
    swofty_root_dir = swofty_root();
    if (!load_passes()) {
        swofty::log("[postfx] the postfx feature is on, but assets.json names no usable pass");
        return;
    }
    if (GetEnvironmentVariableA("SWOFTY_POSTFX_CAPTURE", v, sizeof v) && v[0]) {
        capture_dir = v;
        capture_file = swofty_root() + L"postfx.capture";
    }
    if (GetEnvironmentVariableA("SWOFTY_POSTFX_TRACE", v, sizeof v) && v[0])
        trace_left = unsigned(std::strtoul(v, nullptr, 10));
    if (GetEnvironmentVariableA("SWOFTY_POSTFX_ORDER", v, sizeof v) && v[0])
        order_left = unsigned(std::strtoul(v, nullptr, 10));
    refresh_live();
    d3d12hooks::on_setname(on_setname);
    d3d12hooks::on_barrier(on_barrier);
    d3d12hooks::on_exec(on_exec);
    d3d12hooks::on_present(on_present);
    d3d12hooks::on_reset(on_reset);
    d3d12hooks::ProbeScope probe;
    bool hooked = d3d12hooks::arm_device();
    if (!hooked) {
        swofty::log("[postfx] the core's D3D12 device hooks did not arm; post-processing is off");
        return;
    }
    state_armed = d3d12hooks::arm_list_state();
    if (!state_armed)
        swofty::log("[postfx] the command list state hooks did not arm; passes run only between the game's lists");
    if (!d3d12hooks::arm_swapchain())
        swofty::log("[postfx] the swapchain hooks did not arm; toggle keys, live settings and traces are off");
    swofty::log("[postfx] %zu pass(es) and %zu texture(s) wait for the game's picture%s%s", passes.size(),
                textures.size(), capture_dir.empty() ? "" : "; captures go to ", capture_dir.c_str());
}

}
