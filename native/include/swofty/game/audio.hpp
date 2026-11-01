#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

using Snd_Request_JobWorker_t = void(void *worker, uint32_t job_type, uint32_t count);
inline constexpr fn<"Snd_Request_JobWorker", Snd_Request_JobWorker_t> Snd_Request_JobWorker{}; // L2 experimental
inline constexpr data<"g_Snd_JobWorkerLock", void *> g_Snd_JobWorkerLock{}; // L2 experimental
inline constexpr data<"g_Snd_JobWorkerOverflow", uint8_t[1152]> g_Snd_JobWorkerOverflow{}; // L2 experimental
inline constexpr data<"g_Snd_JobWorkerPending", int32_t> g_Snd_JobWorkerPending{}; // L2 experimental

}
