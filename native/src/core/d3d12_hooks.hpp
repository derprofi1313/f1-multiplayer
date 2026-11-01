#pragma once
#include <windows.h>

#include <d3d12.h>
#include <dxgi1_6.h>

#include <string>

// The core's one set of D3D12 / DXGI hooks. Features subscribe instead of hooking the same entry points
// themselves. Subscribe before arming; callbacks run on the game's thread inside the hooked call.
namespace d3d12hooks {

struct Swapchain {
    void *sc = nullptr;
    ID3D12CommandQueue *queue = nullptr;
    std::string how, by, fg;
    bool streamline = false;
    UINT color_space = 0;
    bool color_space_set = false;
};

using PresentFn = void (*)(IDXGISwapChain3 *sc, const Swapchain *known, void *caller);
using PresentedFn = void (*)(IDXGISwapChain3 *sc, HRESULT hr);
using ResizeFn = void (*)(IDXGISwapChain3 *sc);
using ExecFn = bool (*)(ID3D12CommandQueue *q, UINT n, ID3D12CommandList *const *lists);
using BarrierFn = bool (*)(ID3D12GraphicsCommandList *list, UINT n, const D3D12_RESOURCE_BARRIER *barriers);
using SetNameFn = void (*)(ID3D12Object *object, LPCWSTR name);
using ResetFn = void (*)(ID3D12GraphicsCommandList *list);

void on_present(PresentFn fn);
void on_presented(PresentedFn fn);
void on_resize(ResizeFn fn);
void on_exec(ExecFn fn);
void on_barrier(BarrierFn fn);
void on_setname(SetNameFn fn);
void on_reset(ResetFn fn);

bool arm_swapchain();
bool arm_device();

// While one is alive, arm_device and arm_swapchain share one throwaway probe
// device: creating a D3D12 device takes about half a second on the main thread.
struct ProbeScope {
    ProbeScope();
    ~ProbeScope();
    ProbeScope(const ProbeScope &) = delete;
    ProbeScope &operator=(const ProbeScope &) = delete;
};

void exec_original(ID3D12CommandQueue *q, UINT n, ID3D12CommandList *const *lists);
void barrier_original(ID3D12GraphicsCommandList *list, UINT n, const D3D12_RESOURCE_BARRIER *barriers);

// What the core's compute work in a game's command list changes: the pipeline, the descriptor heaps
// and the compute root signature with its arguments, and the graphics descriptor tables, which are
// set again after the heaps come back. Once arm_list_state has run, every list's are
// followed from its Reset; a feature saves them before recording into the game's list and puts them
// back afterwards, so the game's next draw or dispatch finds its own bindings.
struct RootArg {
    uint8_t kind = 0;
    UINT count = 0;
    UINT64 value = 0;
    uint32_t c[64] = {};
};

struct ListState {
    bool known = false;
    ID3D12PipelineState *pso = nullptr;
    ID3D12StateObject *state_object = nullptr;
    UINT heaps = 0;
    ID3D12DescriptorHeap *heap[2] = {};
    ID3D12RootSignature *compute_root = nullptr;
    UINT args = 0;
    RootArg arg[64];
    ID3D12RootSignature *graphics_root = nullptr;
    UINT tables = 0;
    UINT64 table[64] = {};
};

bool arm_list_state();
bool list_state(ID3D12GraphicsCommandList *list, ListState &out);
void restore_list_state(ID3D12GraphicsCommandList *list, const ListState &s);

std::string module_of(const void *addr, HMODULE *mod = nullptr);

}
