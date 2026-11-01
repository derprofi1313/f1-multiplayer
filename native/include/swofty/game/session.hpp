#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

struct AiController {
    uint8_t _pad_0x0[0x8];
    struct InputRecord *input; // L2 experimental
    void *owner; // L2 experimental
    uint8_t vehicle; // L2 experimental
};
static_assert(offsetof(AiController, input) == 0x8);
static_assert(offsetof(AiController, owner) == 0x10);
static_assert(offsetof(AiController, vehicle) == 0x18);

struct ControllerSet {
    uint8_t _pad_0x0[0x10];
    uint32_t mode_masked; // L2 experimental
    uint8_t _pad_0x14[0x4];
    uint32_t mode_key; // L2 experimental
    uint8_t _pad_0x1c[0x4];
    void *human_controller; // L2 experimental
    void *ai_controller; // L2 experimental
    uint32_t flags; // L2 experimental
    uint8_t vehicle; // L2 experimental
};
static_assert(offsetof(ControllerSet, mode_masked) == 0x10);
static_assert(offsetof(ControllerSet, mode_key) == 0x18);
static_assert(offsetof(ControllerSet, human_controller) == 0x20);
static_assert(offsetof(ControllerSet, ai_controller) == 0x28);
static_assert(offsetof(ControllerSet, flags) == 0x30);
static_assert(offsetof(ControllerSet, vehicle) == 0x34);

struct EntrantsManager {
    uint8_t _pad_0x0[0x2b0];
    GameSetupMode *game_setup_mode; // L2 experimental
};
static_assert(offsetof(EntrantsManager, game_setup_mode) == 0x2b0);

struct GameSetupMode {
    uint8_t _pad_0x0[0x1e0];
    void *drs_allowed_lap_group; // L2 experimental
};
static_assert(offsetof(GameSetupMode, drs_allowed_lap_group) == 0x1e0);

struct GameWeatherSystem {
    uint8_t _pad_0x0[0x1d98];
    TimeOfDaySystem *tod; // L2 experimental
    uint8_t _pad_0x1da0[0x8];
    TimeOfDayOwner *tod_owner; // L2 experimental
    uint8_t _pad_0x1db0[0x152];
    uint8_t weekend_valid; // L2 experimental
    uint8_t _pad_0x1f03[0x1];
    uint32_t season_index; // L2 experimental
    uint32_t weekend_index; // L2 experimental
    uint8_t _pad_0x1f0c[0x4];
    uint32_t base_seed; // L2 experimental
    uint32_t forced_seed; // L2 experimental
    uint32_t weekend; // L2 experimental
    uint8_t _pad_0x1f1c[0x101e];
    uint8_t dataset_dynamic[8]; // L2 experimental
};
static_assert(offsetof(GameWeatherSystem, tod) == 0x1d98);
static_assert(offsetof(GameWeatherSystem, tod_owner) == 0x1da8);
static_assert(offsetof(GameWeatherSystem, weekend_valid) == 0x1f02);
static_assert(offsetof(GameWeatherSystem, season_index) == 0x1f04);
static_assert(offsetof(GameWeatherSystem, weekend_index) == 0x1f08);
static_assert(offsetof(GameWeatherSystem, base_seed) == 0x1f10);
static_assert(offsetof(GameWeatherSystem, forced_seed) == 0x1f14);
static_assert(offsetof(GameWeatherSystem, weekend) == 0x1f18);
static_assert(offsetof(GameWeatherSystem, dataset_dynamic) == 0x2f3a);

struct SpectatorDirector {
    uint8_t _pad_0x0[0xd0];
    uint8_t ui_active; // L2 experimental
    uint8_t active; // L2 experimental
    uint8_t _pad_0xd2[0x1];
    uint8_t should_quit; // L2 experimental
    uint8_t _pad_0xd4[0x2];
    uint8_t current_driver; // L2 experimental
    uint8_t _pad_0xd7[0x1];
    int32_t current_camera; // L2 experimental
    uint8_t requested_driver; // L2 experimental
    uint8_t _pad_0xdd[0x3];
    int32_t requested_camera; // L2 experimental
};
static_assert(offsetof(SpectatorDirector, ui_active) == 0xd0);
static_assert(offsetof(SpectatorDirector, active) == 0xd1);
static_assert(offsetof(SpectatorDirector, should_quit) == 0xd3);
static_assert(offsetof(SpectatorDirector, current_driver) == 0xd6);
static_assert(offsetof(SpectatorDirector, current_camera) == 0xd8);
static_assert(offsetof(SpectatorDirector, requested_driver) == 0xdc);
static_assert(offsetof(SpectatorDirector, requested_camera) == 0xe0);

struct TimeOfDayClock {
    uint8_t _pad_0x0[0x68];
    double time_a; // L2 experimental
    double time_b; // L2 experimental
    uint8_t _pad_0x78[0x8];
    float time_scale; // L2 experimental
    float length; // L2 experimental
    uint8_t _pad_0x88[0x200];
};
static_assert(offsetof(TimeOfDayClock, time_a) == 0x68);
static_assert(offsetof(TimeOfDayClock, time_b) == 0x70);
static_assert(offsetof(TimeOfDayClock, time_scale) == 0x80);
static_assert(offsetof(TimeOfDayClock, length) == 0x84);
static_assert(sizeof(TimeOfDayClock) == 0x288);

struct TimeOfDayOwner {
    uint8_t _pad_0x0[0x2d0];
    TimeOfDaySystem *tod; // L2 experimental
};
static_assert(offsetof(TimeOfDayOwner, tod) == 0x2d0);

struct WeatherState {
    uint32_t type; // L2 experimental
    float intensity; // L2 experimental
    float end; // L2 experimental
    float transition; // L2 experimental
};
static_assert(offsetof(WeatherState, type) == 0x0);
static_assert(offsetof(WeatherState, intensity) == 0x4);
static_assert(offsetof(WeatherState, end) == 0x8);
static_assert(offsetof(WeatherState, transition) == 0xc);
static_assert(sizeof(WeatherState) == 0x10);

struct WeatherDataset {
    uint8_t _pad_0x0[0xb90];
    WeatherState states[38]; // L2 experimental
    uint32_t state_total; // L2 experimental
    uint8_t _pad_0xdf4[0x4];
    uint32_t state_count; // L2 experimental
    uint8_t _pad_0xdfc[0x4];
};
static_assert(offsetof(WeatherDataset, states) == 0xb90);
static_assert(offsetof(WeatherDataset, state_total) == 0xdf0);
static_assert(offsetof(WeatherDataset, state_count) == 0xdf8);
static_assert(sizeof(WeatherDataset) == 0xe00);

struct WeatherTypeRange {
    float weight; // L2 experimental
    uint8_t _pad_0x4[0x8];
    float intensity_min; // L2 experimental
    float intensity_max; // L2 experimental
};
static_assert(offsetof(WeatherTypeRange, weight) == 0x0);
static_assert(offsetof(WeatherTypeRange, intensity_min) == 0xc);
static_assert(offsetof(WeatherTypeRange, intensity_max) == 0x10);
static_assert(sizeof(WeatherTypeRange) == 0x14);

struct TimeOfDaySystem {
    uint8_t _pad_0x0[0x2c8];
    TimeOfDayClock clocks[10]; // L2 experimental
    uint8_t _pad_0x1c18[0xa3c];
    uint8_t scripted_scenario; // L2 experimental
    uint8_t _pad_0x2655[0x6f];
    uint8_t active_dataset; // L2 experimental
    uint8_t _pad_0x26c5[0x13];
    WeatherDataset datasets[10]; // L2 experimental
    uint8_t _pad_0xb2d8[0x64];
    uint32_t seed; // L2 experimental
    uint8_t _pad_0xb340[0x10];
    float dataset_length[10]; // L2 experimental
    WeatherTypeRange type_ranges[6]; // L2 experimental
    uint8_t _pad_0xb3f0[0x36];
    uint8_t exact_timing; // L2 experimental
    uint8_t _pad_0xb427[0x2];
    uint8_t ready_a; // L2 experimental
    uint8_t ready_b; // L2 experimental
    uint8_t _pad_0xb42b[0x2];
    uint8_t random_override; // L2 experimental
    uint8_t _pad_0xb42e[0x4];
    uint8_t busy; // L2 experimental
    uint8_t _pad_0xb433[0x3];
    uint8_t override_active; // L2 experimental
};
static_assert(offsetof(TimeOfDaySystem, clocks) == 0x2c8);
static_assert(offsetof(TimeOfDaySystem, scripted_scenario) == 0x2654);
static_assert(offsetof(TimeOfDaySystem, active_dataset) == 0x26c4);
static_assert(offsetof(TimeOfDaySystem, datasets) == 0x26d8);
static_assert(offsetof(TimeOfDaySystem, seed) == 0xb33c);
static_assert(offsetof(TimeOfDaySystem, dataset_length) == 0xb350);
static_assert(offsetof(TimeOfDaySystem, type_ranges) == 0xb378);
static_assert(offsetof(TimeOfDaySystem, exact_timing) == 0xb426);
static_assert(offsetof(TimeOfDaySystem, ready_a) == 0xb429);
static_assert(offsetof(TimeOfDaySystem, ready_b) == 0xb42a);
static_assert(offsetof(TimeOfDaySystem, random_override) == 0xb42d);
static_assert(offsetof(TimeOfDaySystem, busy) == 0xb432);
static_assert(offsetof(TimeOfDaySystem, override_active) == 0xb436);

struct WeatherAndTimeOfDayComponent {
    uint8_t _pad_0x0[0x2a0];
    GameWeatherSystem *weather_system; // L2 experimental
    uint8_t _pad_0x2a8[0x1c];
    uint32_t random_seed; // L2 experimental
};
static_assert(offsetof(WeatherAndTimeOfDayComponent, weather_system) == 0x2a0);
static_assert(offsetof(WeatherAndTimeOfDayComponent, random_seed) == 0x2c4);

struct WeatherSessionSettings {
    uint8_t _pad_0x0[0x4];
    uint32_t session_type; // L2 experimental
    uint8_t _pad_0x8[0xb0c];
    uint32_t mode; // L2 experimental
    uint8_t exact_timing; // L2 experimental
};
static_assert(offsetof(WeatherSessionSettings, session_type) == 0x4);
static_assert(offsetof(WeatherSessionSettings, mode) == 0xb14);
static_assert(offsetof(WeatherSessionSettings, exact_timing) == 0xb18);

using Sess_Apply_ContentAvailability_t = void(void *self, void *availability);
inline constexpr fn<"Sess_Apply_ContentAvailability", Sess_Apply_ContentAvailability_t> Sess_Apply_ContentAvailability{}; // L2 experimental
using Sess_Build_GroupResult_t = void(void *weekend, uint8_t group, void *result);
inline constexpr fn<"Sess_Build_GroupResult", Sess_Build_GroupResult_t> Sess_Build_GroupResult{}; // L2 experimental
inline constexpr address<"Sess_ControllerSystem_ApplyInputs"> Sess_ControllerSystem_ApplyInputs{}; // L2 experimental
inline constexpr address<"Sess_ControllerSystem_BuildSets"> Sess_ControllerSystem_BuildSets{}; // L2 experimental
inline constexpr address<"Sess_ControllerSystem_DestroySets"> Sess_ControllerSystem_DestroySets{}; // L2 experimental
inline constexpr address<"Sess_CopyOptionsToState"> Sess_CopyOptionsToState{}; // L2 experimental
inline constexpr address<"Sess_DoughnutSequence_Enter"> Sess_DoughnutSequence_Enter{}; // L2 experimental
inline constexpr address<"Sess_DriveThroughPenalty_ReceiveData"> Sess_DriveThroughPenalty_ReceiveData{}; // L2 experimental
inline constexpr address<"Sess_DriveThroughPenalty_SendData"> Sess_DriveThroughPenalty_SendData{}; // L2 experimental
inline constexpr address<"Sess_DriveThroughPenalty_SendData_2"> Sess_DriveThroughPenalty_SendData_2{}; // L2 experimental
using Sess_Find_GroupRowPosition_t = int32_t *(void *weekend, void *result, uint32_t driver_key);
inline constexpr fn<"Sess_Find_GroupRowPosition", Sess_Find_GroupRowPosition_t> Sess_Find_GroupRowPosition{}; // L2 experimental
inline constexpr address<"Sess_Flow_ParseXml"> Sess_Flow_ParseXml{}; // L2 experimental
inline constexpr address<"Sess_FormationLapLogic_CheckAllParked"> Sess_FormationLapLogic_CheckAllParked{}; // L2 experimental
inline constexpr address<"Sess_FormationLapLogic_Reset"> Sess_FormationLapLogic_Reset{}; // L2 experimental
inline constexpr address<"Sess_FormationLapType_Update"> Sess_FormationLapType_Update{}; // L2 experimental
using Sess_FormationLap_Update_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_FormationLap_Update", Sess_FormationLap_Update_t> Sess_FormationLap_Update{}; // L2 experimental
inline constexpr address<"Sess_FreezeAllVehicles"> Sess_FreezeAllVehicles{}; // L2 experimental
using Sess_GameWeatherSystem_SetActiveSessionType_t = void(void *gws, int session_type, float time_scale);
inline constexpr fn<"Sess_GameWeatherSystem_SetActiveSessionType", Sess_GameWeatherSystem_SetActiveSessionType_t> Sess_GameWeatherSystem_SetActiveSessionType{}; // L2 experimental
using Sess_GameWeatherSystem_SetupSession_t = void(void *ws, void *settings, void *config);
inline constexpr fn<"Sess_GameWeatherSystem_SetupSession", Sess_GameWeatherSystem_SetupSession_t> Sess_GameWeatherSystem_SetupSession{}; // L2 experimental, GameWeatherSystem::setupSession.
using Sess_GameWeatherSystem_UpdateRandomSeed_t = void(void *ws, uint32_t seed, uint32_t weekend);
inline constexpr fn<"Sess_GameWeatherSystem_UpdateRandomSeed", Sess_GameWeatherSystem_UpdateRandomSeed_t> Sess_GameWeatherSystem_UpdateRandomSeed{}; // L2 experimental, GameWeatherSystem::updateRandomSeed.
using Sess_Get_PlayerVehicleIndex_t = uint8_t(void *ctx, int local_player);
inline constexpr fn<"Sess_Get_PlayerVehicleIndex", Sess_Get_PlayerVehicleIndex_t> Sess_Get_PlayerVehicleIndex{}; // L2 experimental
using Sess_GridRundown_Update_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_GridRundown_Update", Sess_GridRundown_Update_t> Sess_GridRundown_Update{}; // L2 experimental
using Sess_InGamePQ_Update_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_InGamePQ_Update", Sess_InGamePQ_Update_t> Sess_InGamePQ_Update{}; // L2 experimental
using Sess_InGame_UpdateBase_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_InGame_UpdateBase", Sess_InGame_UpdateBase_t> Sess_InGame_UpdateBase{}; // L2 experimental
using Sess_Invalidate_Lap_Thunk_t = void(void *rules, uint32_t vehicle, uint32_t reason, char mode);
inline constexpr fn<"Sess_Invalidate_Lap_Thunk", Sess_Invalidate_Lap_Thunk_t> Sess_Invalidate_Lap_Thunk{}; // L2 experimental
using Sess_Issue_DriveThrough_t = void(void *rules, uint8_t *incident, uint32_t reason);
inline constexpr fn<"Sess_Issue_DriveThrough", Sess_Issue_DriveThrough_t> Sess_Issue_DriveThrough{}; // L2 experimental
inline constexpr address<"Sess_Issue_GridPlacePenalty"> Sess_Issue_GridPlacePenalty{}; // L2 experimental
inline constexpr address<"Sess_Issue_PostRaceTimePenalty"> Sess_Issue_PostRaceTimePenalty{}; // L2 experimental
using Sess_Issue_TimePenalty_t = void(void *rules, uint8_t *incident, uint32_t reason, uint32_t seconds);
inline constexpr fn<"Sess_Issue_TimePenalty", Sess_Issue_TimePenalty_t> Sess_Issue_TimePenalty{}; // L2 experimental
using Sess_Issue_Warning_t = void(void *rules, uint8_t *incident, uint32_t reason);
inline constexpr fn<"Sess_Issue_Warning", Sess_Issue_Warning_t> Sess_Issue_Warning{}; // L2 experimental
inline constexpr address<"Sess_LapTimes_OnNewSector"> Sess_LapTimes_OnNewSector{}; // L2 experimental
inline constexpr address<"Sess_LapTimes_ReportSplit"> Sess_LapTimes_ReportSplit{}; // L2 experimental
inline constexpr address<"Sess_LapTimes_SendLapHistory"> Sess_LapTimes_SendLapHistory{}; // L2 experimental
using Sess_Load_Teams_t = void(void *entrants_system, uint8_t set);
inline constexpr fn<"Sess_Load_Teams", Sess_Load_Teams_t> Sess_Load_Teams{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestCareerHubLoad"> Sess_Loader_RequestCareerHubLoad{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestCareerHubUnload"> Sess_Loader_RequestCareerHubUnload{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestSessionLoad"> Sess_Loader_RequestSessionLoad{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestSessionUnload"> Sess_Loader_RequestSessionUnload{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestTrackLoad"> Sess_Loader_RequestTrackLoad{}; // L2 experimental
inline constexpr address<"Sess_Loader_RequestTrackUnload"> Sess_Loader_RequestTrackUnload{}; // L2 experimental
inline constexpr address<"Sess_Loader_SetupTrackAssets"> Sess_Loader_SetupTrackAssets{}; // L2 experimental
inline constexpr address<"Sess_MarshalLights_Set"> Sess_MarshalLights_Set{}; // L2 experimental
inline constexpr address<"Sess_MarshalLights_UpdateZoneShader"> Sess_MarshalLights_UpdateZoneShader{}; // L2 experimental
inline constexpr address<"Sess_MarshalZoneMonitor_Init"> Sess_MarshalZoneMonitor_Init{}; // L2 experimental
inline constexpr address<"Sess_MarshalZone_CheckFlags"> Sess_MarshalZone_CheckFlags{}; // L2 experimental
inline constexpr address<"Sess_MisconductManager_Ctor"> Sess_MisconductManager_Ctor{}; // L2 experimental
inline constexpr address<"Sess_Misconduct_CheckEnvironmentContact"> Sess_Misconduct_CheckEnvironmentContact{}; // L2 experimental
inline constexpr address<"Sess_Misconduct_CheckWallRiding"> Sess_Misconduct_CheckWallRiding{}; // L2 experimental
inline constexpr address<"Sess_Misconduct_DumpTable"> Sess_Misconduct_DumpTable{}; // L2 experimental
inline constexpr address<"Sess_Misconduct_HandBackIllegalOvertake"> Sess_Misconduct_HandBackIllegalOvertake{}; // L2 experimental
inline constexpr address<"Sess_Misconduct_OnCarToCarCollision"> Sess_Misconduct_OnCarToCarCollision{}; // L2 experimental
inline constexpr address<"Sess_MultipleWarningsPenalty_ReceiveData"> Sess_MultipleWarningsPenalty_ReceiveData{}; // L2 experimental
inline constexpr address<"Sess_MultipleWarningsPenalty_SendData"> Sess_MultipleWarningsPenalty_SendData{}; // L2 experimental
inline constexpr address<"Sess_MultipleWarningsPenalty_SendData_2"> Sess_MultipleWarningsPenalty_SendData_2{}; // L2 experimental
inline constexpr address<"Sess_OsqTeleportToStart"> Sess_OsqTeleportToStart{}; // L2 experimental
inline constexpr address<"Sess_OvertakeLog_SendAllRecords"> Sess_OvertakeLog_SendAllRecords{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_Print"> Sess_PenaltyLog_Print{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_ReceiveAddRecord"> Sess_PenaltyLog_ReceiveAddRecord{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_RemoveVehicle"> Sess_PenaltyLog_RemoveVehicle{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_SendAllRecords"> Sess_PenaltyLog_SendAllRecords{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_SendRecord"> Sess_PenaltyLog_SendRecord{}; // L2 experimental
inline constexpr address<"Sess_PenaltyLog_SendRecord_2"> Sess_PenaltyLog_SendRecord_2{}; // L2 experimental
inline constexpr address<"Sess_PlayerTerminalDamage_Enter"> Sess_PlayerTerminalDamage_Enter{}; // L2 experimental
inline constexpr address<"Sess_PreLights_Update"> Sess_PreLights_Update{}; // L2 experimental
using Sess_PreRaceToRace_Update_t = uint32_t *(void *state, uint32_t *route);
inline constexpr fn<"Sess_PreRaceToRace_Update", Sess_PreRaceToRace_Update_t> Sess_PreRaceToRace_Update{}; // L2 experimental
using Sess_PreRace_Update_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_PreRace_Update", Sess_PreRace_Update_t> Sess_PreRace_Update{}; // L2 experimental
inline constexpr address<"Sess_RC_AheadOfLeadUpdate"> Sess_RC_AheadOfLeadUpdate{}; // L2 experimental
inline constexpr address<"Sess_RC_HandleAheadOfLeadData"> Sess_RC_HandleAheadOfLeadData{}; // L2 experimental
inline constexpr address<"Sess_RC_SendAheadOfLeadData"> Sess_RC_SendAheadOfLeadData{}; // L2 experimental
inline constexpr address<"Sess_RaceRuleset_OnLapCompleted"> Sess_RaceRuleset_OnLapCompleted{}; // L2 experimental
inline constexpr address<"Sess_RaceRuleset_SetTimeLimit"> Sess_RaceRuleset_SetTimeLimit{}; // L2 experimental
inline constexpr address<"Sess_Read_DeploymentPoints"> Sess_Read_DeploymentPoints{}; // L2 experimental
inline constexpr address<"Sess_Read_LoosePartsRules"> Sess_Read_LoosePartsRules{}; // L2 experimental
using Sess_Record_GroupTimes_t = void(void *weekend, uint8_t group, void *entries);
inline constexpr fn<"Sess_Record_GroupTimes", Sess_Record_GroupTimes_t> Sess_Record_GroupTimes{}; // L2 experimental
inline constexpr address<"Sess_Recordable_Register"> Sess_Recordable_Register{}; // L2 experimental
inline constexpr address<"Sess_RedFlagFreezeVehicles_Enter"> Sess_RedFlagFreezeVehicles_Enter{}; // L2 experimental
inline constexpr address<"Sess_RedFlagPerformRepairs_Update"> Sess_RedFlagPerformRepairs_Update{}; // L2 experimental
using Sess_RedFlagRaceStrategy_Enter_t = uint64_t(void *state);
inline constexpr fn<"Sess_RedFlagRaceStrategy_Enter", Sess_RedFlagRaceStrategy_Enter_t> Sess_RedFlagRaceStrategy_Enter{}; // L2 experimental
using Sess_RedFlagRaceStrategy_Update_t = uint32_t *(void *state, uint32_t *route);
inline constexpr fn<"Sess_RedFlagRaceStrategy_Update", Sess_RedFlagRaceStrategy_Update_t> Sess_RedFlagRaceStrategy_Update{}; // L2 experimental
inline constexpr address<"Sess_RedFlagSystem_Deploy"> Sess_RedFlagSystem_Deploy{}; // L2 experimental, RedFlagSystem
inline constexpr address<"Sess_RedFlagSystem_OnSessionBeginGameplay"> Sess_RedFlagSystem_OnSessionBeginGameplay{}; // L2 experimental
inline constexpr address<"Sess_RedFlagSystem_OnStartLightsBegin"> Sess_RedFlagSystem_OnStartLightsBegin{}; // L2 experimental
inline constexpr address<"Sess_RedFlagSystem_ResetCarStatus"> Sess_RedFlagSystem_ResetCarStatus{}; // L2 experimental
inline constexpr address<"Sess_RedFlagSystem_SimulateSessionTime"> Sess_RedFlagSystem_SimulateSessionTime{}; // L2 experimental
inline constexpr address<"Sess_RedFlagSystem_SnapshotDamage"> Sess_RedFlagSystem_SnapshotDamage{}; // L2 experimental
inline constexpr address<"Sess_RedFlagToRestart_Exit"> Sess_RedFlagToRestart_Exit{}; // L2 experimental
inline constexpr address<"Sess_RedFlagToRestart_Update"> Sess_RedFlagToRestart_Update{}; // L2 experimental
inline constexpr address<"Sess_RedFlag_SimulateUnlapLaps"> Sess_RedFlag_SimulateUnlapLaps{}; // L2 experimental
inline constexpr address<"Sess_Register_EntrantSetupMode"> Sess_Register_EntrantSetupMode{}; // L2 experimental
using Sess_Register_RaceRecordables_t = void(void *game);
inline constexpr fn<"Sess_Register_RaceRecordables", Sess_Register_RaceRecordables_t> Sess_Register_RaceRecordables{}; // L2 experimental
inline constexpr address<"Sess_ResetReplay_Update"> Sess_ResetReplay_Update{}; // L2 experimental
inline constexpr address<"Sess_Result_ChangeVehicleStatus"> Sess_Result_ChangeVehicleStatus{}; // L2 experimental
inline constexpr address<"Sess_Result_DecodeVehicleResult"> Sess_Result_DecodeVehicleResult{}; // L2 experimental
inline constexpr address<"Sess_Result_RequestVehicleStatus"> Sess_Result_RequestVehicleStatus{}; // L2 experimental
inline constexpr address<"Sess_RetirementSequence_PlayTerminalDamage"> Sess_RetirementSequence_PlayTerminalDamage{}; // L2 experimental
inline constexpr address<"Sess_RetirementSequence_Update"> Sess_RetirementSequence_Update{}; // L2 experimental
inline constexpr address<"Sess_RuleManager_HandleSessionEvents"> Sess_RuleManager_HandleSessionEvents{}; // L2 experimental
using Sess_RuleManager_Initialise_t = uint32_t(void *self, void *systems, void *components);
inline constexpr fn<"Sess_RuleManager_Initialise", Sess_RuleManager_Initialise_t> Sess_RuleManager_Initialise{}; // L2 experimental
inline constexpr address<"Sess_Rules_ApplyPostSessionPenalties"> Sess_Rules_ApplyPostSessionPenalties{}; // L2 experimental
inline constexpr address<"Sess_Rules_CornerCuttingTimeGained_2"> Sess_Rules_CornerCuttingTimeGained_2{}; // L2 experimental
inline constexpr address<"Sess_Rules_Disqualify"> Sess_Rules_Disqualify{}; // L2 experimental
inline constexpr address<"Sess_Rules_DisqualifyForPenaltyPoints"> Sess_Rules_DisqualifyForPenaltyPoints{}; // L2 experimental
using Sess_Rules_IsPracticeQualifying_t = bool(void *owner_with_session_at_0x1b8);
inline constexpr fn<"Sess_Rules_IsPracticeQualifying", Sess_Rules_IsPracticeQualifying_t> Sess_Rules_IsPracticeQualifying{}; // L2 experimental
inline constexpr address<"Sess_Rules_PaceCarNewLap"> Sess_Rules_PaceCarNewLap{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadCollisionExpectedSpeed"> Sess_Rules_ReadCollisionExpectedSpeed{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadCollisionSettings"> Sess_Rules_ReadCollisionSettings{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadCornerCuttingSettings"> Sess_Rules_ReadCornerCuttingSettings{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadFormationLapMiniGame"> Sess_Rules_ReadFormationLapMiniGame{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadPitWhiteLine"> Sess_Rules_ReadPitWhiteLine{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadRedFlagRepairTimes"> Sess_Rules_ReadRedFlagRepairTimes{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadSafetyCarPacePenalties"> Sess_Rules_ReadSafetyCarPacePenalties{}; // L2 experimental
inline constexpr address<"Sess_Rules_ReadYellowFlagOvertaking"> Sess_Rules_ReadYellowFlagOvertaking{}; // L2 experimental
inline constexpr address<"Sess_Rules_RemoveInvalidOvertake"> Sess_Rules_RemoveInvalidOvertake{}; // L2 experimental
inline constexpr address<"Sess_Rules_SendBlackFlagTimerToOsd"> Sess_Rules_SendBlackFlagTimerToOsd{}; // L2 experimental
inline constexpr address<"Sess_Ruleset_FinaliseResults"> Sess_Ruleset_FinaliseResults{}; // L2 experimental
inline constexpr address<"Sess_SafetyCarDeployment_OnSessionRestart"> Sess_SafetyCarDeployment_OnSessionRestart{}; // L2 experimental
inline constexpr address<"Sess_SafetyCarDeployment_PreSessionRestart"> Sess_SafetyCarDeployment_PreSessionRestart{}; // L2 experimental
inline constexpr address<"Sess_SafetyCarEmergencyGhosting_Init"> Sess_SafetyCarEmergencyGhosting_Init{}; // L2 experimental
inline constexpr address<"Sess_SafetyCarLogic_Subscribe"> Sess_SafetyCarLogic_Subscribe{}; // L2 experimental
using Sess_SafetyCarTether_UpdateProgress_t = void(void *tether_entry);
inline constexpr fn<"Sess_SafetyCarTether_UpdateProgress", Sess_SafetyCarTether_UpdateProgress_t> Sess_SafetyCarTether_UpdateProgress{}; // L2 experimental
using Sess_SafetyCarTether_UpdateTarget_t = void(void *tether_entry, float gap);
inline constexpr fn<"Sess_SafetyCarTether_UpdateTarget", Sess_SafetyCarTether_UpdateTarget_t> Sess_SafetyCarTether_UpdateTarget{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_ChangeState"> Sess_SafetyCar_ChangeState{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_IntersectionTime"> Sess_SafetyCar_IntersectionTime{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_OnLapChanged"> Sess_SafetyCar_OnLapChanged{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_ReadDynamicObstruction"> Sess_SafetyCar_ReadDynamicObstruction{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_ReadMaxRedFlags"> Sess_SafetyCar_ReadMaxRedFlags{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_UnlapGate"> Sess_SafetyCar_UnlapGate{}; // L2 experimental
inline constexpr address<"Sess_SafetyCar_VehicleTransition"> Sess_SafetyCar_VehicleTransition{}; // L2 experimental
inline constexpr address<"Sess_ScCond_HazardDetected_Update"> Sess_ScCond_HazardDetected_Update{}; // L2 experimental
inline constexpr address<"Sess_ScCond_MinLapsOut_Update"> Sess_ScCond_MinLapsOut_Update{}; // L2 experimental
inline constexpr address<"Sess_ScCond_OffTrack_Update"> Sess_ScCond_OffTrack_Update{}; // L2 experimental
inline constexpr address<"Sess_ScCond_PitAreaClear_Update"> Sess_ScCond_PitAreaClear_Update{}; // L2 experimental
inline constexpr address<"Sess_ScCond_TriggerVsc_Update"> Sess_ScCond_TriggerVsc_Update{}; // L2 experimental
using Sess_ScDebrisAccumulator_Update_t = void(void *acc);
inline constexpr fn<"Sess_ScDebrisAccumulator_Update", Sess_ScDebrisAccumulator_Update_t> Sess_ScDebrisAccumulator_Update{}; // L2 experimental
inline constexpr address<"Sess_ScState_Deploy_Enter"> Sess_ScState_Deploy_Enter{}; // L2 experimental
inline constexpr address<"Sess_ScState_Deploy_ReadTunables"> Sess_ScState_Deploy_ReadTunables{}; // L2 experimental
inline constexpr address<"Sess_ScState_Return_Enter"> Sess_ScState_Return_Enter{}; // L2 experimental
inline constexpr address<"Sess_ScState_ReturnedToPit_Enter"> Sess_ScState_ReturnedToPit_Enter{}; // L2 experimental
inline constexpr address<"Sess_ScState_ReturnedToPit_Exit"> Sess_ScState_ReturnedToPit_Exit{}; // L2 experimental
inline constexpr address<"Sess_ScState_VscDeploy_Enter"> Sess_ScState_VscDeploy_Enter{}; // L2 experimental
inline constexpr address<"Sess_ScState_VscEnding_Enter"> Sess_ScState_VscEnding_Enter{}; // L2 experimental
inline constexpr address<"Sess_ScState_VscEnding_Exit"> Sess_ScState_VscEnding_Exit{}; // L2 experimental
inline constexpr address<"Sess_SessionClock_SetTime"> Sess_SessionClock_SetTime{}; // L2 experimental
inline constexpr address<"Sess_Set_RowsAvailable"> Sess_Set_RowsAvailable{}; // L2 experimental
using Sess_Set_SafetyCarDeploymentType_t = void(void *sc_state, const void *sc_deployment_msg);
inline constexpr fn<"Sess_Set_SafetyCarDeploymentType", Sess_Set_SafetyCarDeploymentType_t> Sess_Set_SafetyCarDeploymentType{}; // L2 experimental
using Sess_Setup_VehicleComponents_t = void(void *module);
inline constexpr fn<"Sess_Setup_VehicleComponents", Sess_Setup_VehicleComponents_t> Sess_Setup_VehicleComponents{}; // L2 experimental
inline constexpr address<"Sess_SpectatorDirector_SetActive"> Sess_SpectatorDirector_SetActive{}; // L2 experimental
inline constexpr address<"Sess_SpectatorDirector_Update"> Sess_SpectatorDirector_Update{}; // L2 experimental
inline constexpr address<"Sess_SpectatorHud_Hide"> Sess_SpectatorHud_Hide{}; // L2 experimental
inline constexpr address<"Sess_SpeedLimitPenalty_ReceiveData"> Sess_SpeedLimitPenalty_ReceiveData{}; // L2 experimental
inline constexpr address<"Sess_SpeedLimitPenalty_SendData"> Sess_SpeedLimitPenalty_SendData{}; // L2 experimental
inline constexpr address<"Sess_StartLights_Enter"> Sess_StartLights_Enter{}; // L2 experimental
inline constexpr address<"Sess_StartLights_Init"> Sess_StartLights_Init{}; // L2 experimental
inline constexpr address<"Sess_Start_VehiclesInGarages"> Sess_Start_VehiclesInGarages{}; // L2 experimental
inline constexpr address<"Sess_StopGoPenalty_ReceiveData"> Sess_StopGoPenalty_ReceiveData{}; // L2 experimental
inline constexpr address<"Sess_StopGoPenalty_SendData"> Sess_StopGoPenalty_SendData{}; // L2 experimental
inline constexpr address<"Sess_StopGoPenalty_SendData_2"> Sess_StopGoPenalty_SendData_2{}; // L2 experimental
inline constexpr address<"Sess_StoryPreLights_Update"> Sess_StoryPreLights_Update{}; // L2 experimental
inline constexpr address<"Sess_TimeOfDayOwner_EvalRainIntensity"> Sess_TimeOfDayOwner_EvalRainIntensity{}; // L2 experimental
using Sess_Tod_BuildSessionDataset_Thunk_t = void(void *tod_owner, const uint8_t *types, const float *hold_s, const float *trans_s, const uint8_t *random, uint32_t count, float len_s, uint16_t session, uint8_t use_count);
inline constexpr fn<"Sess_Tod_BuildSessionDataset_Thunk", Sess_Tod_BuildSessionDataset_Thunk_t> Sess_Tod_BuildSessionDataset_Thunk{}; // L2 experimental
inline constexpr address<"Sess_Tod_BuildWeatherScenario"> Sess_Tod_BuildWeatherScenario{}; // L2 experimental
inline constexpr address<"Sess_Tod_RandomiseTransitions"> Sess_Tod_RandomiseTransitions{}; // L2 experimental
inline constexpr address<"Sess_Tod_ResetDatasetStates"> Sess_Tod_ResetDatasetStates{}; // L2 experimental
inline constexpr address<"Sess_Tod_StitchWeatherScenario"> Sess_Tod_StitchWeatherScenario{}; // L2 experimental
using Sess_TyreAllocation_OnSessionStart_t = void(void *component);
inline constexpr fn<"Sess_TyreAllocation_OnSessionStart", Sess_TyreAllocation_OnSessionStart_t> Sess_TyreAllocation_OnSessionStart{}; // L2 experimental
using Sess_TyreClass_FromName_t = void *(void *classes, const void *name, uint8_t check_weekend_format);
inline constexpr fn<"Sess_TyreClass_FromName", Sess_TyreClass_FromName_t> Sess_TyreClass_FromName{}; // L2 experimental
using Sess_Tyres_ClosestAvailableCompound_t = uint32_t(void *tyre_rules, uint8_t vehicle, const uint32_t *preference, float wear_scale);
inline constexpr fn<"Sess_Tyres_ClosestAvailableCompound", Sess_Tyres_ClosestAvailableCompound_t> Sess_Tyres_ClosestAvailableCompound{}; // L2 experimental
inline constexpr address<"Sess_Veh_UpdateWheelMap"> Sess_Veh_UpdateWheelMap{}; // L2 experimental
inline constexpr address<"Sess_VehiclePlacement_BroadcastGrid"> Sess_VehiclePlacement_BroadcastGrid{}; // L2 experimental
inline constexpr address<"Sess_VehiclePlacement_HandleGridRequest"> Sess_VehiclePlacement_HandleGridRequest{}; // L2 experimental
inline constexpr address<"Sess_VehiclePlacement_PlaceOnSlot"> Sess_VehiclePlacement_PlaceOnSlot{}; // L2 experimental
using Sess_WeatherComponent_SetSystemRandomSeed_t = void(void *component);
inline constexpr fn<"Sess_WeatherComponent_SetSystemRandomSeed", Sess_WeatherComponent_SetSystemRandomSeed_t> Sess_WeatherComponent_SetSystemRandomSeed{}; // L2 experimental, WeatherAndTimeOfDayComponent::setWeatherSystemRandomSeed
using Sess_Weekend_Start_t = void(void *weekend, const void *entrants, bool begin);
inline constexpr fn<"Sess_Weekend_Start", Sess_Weekend_Start_t> Sess_Weekend_Start{}; // L2 experimental, game::championship::Weekend::start(const
using Sess_WinnerSequence_Update_t = uint32_t *(void *state, uint32_t *route, double dt);
inline constexpr fn<"Sess_WinnerSequence_Update", Sess_WinnerSequence_Update_t> Sess_WinnerSequence_Update{}; // L2 experimental
inline constexpr data<"g_Sess_TyreRulesActive", uint8_t> g_Sess_TyreRulesActive{}; // L2 experimental
inline constexpr data<"g_Sess_TyreRulesMode", uint8_t> g_Sess_TyreRulesMode{}; // L2 experimental
inline constexpr data<"g_Sess_TyreRulesSessionType", uint32_t> g_Sess_TyreRulesSessionType{}; // L2 experimental
inline constexpr data<"k_Sess_TyreComponentType", void *> k_Sess_TyreComponentType{}; // L2 experimental
inline constexpr call<"site_Sess_AiVehicleRegisterCall"> site_Sess_AiVehicleRegisterCall{}; // L2 experimental
inline constexpr call<"site_Sess_ControllerSystem_ApplyRecordCall"> site_Sess_ControllerSystem_ApplyRecordCall{}; // L2 experimental
inline constexpr call<"site_Sess_InGamePlayerFinishedAiCall"> site_Sess_InGamePlayerFinishedAiCall{}; // L2 experimental
inline constexpr call<"site_Sess_InGameSessionCompleteAiCall"> site_Sess_InGameSessionCompleteAiCall{}; // L2 experimental
inline constexpr vtable<"vt_Sess_AiController", 4> vt_Sess_AiController{}; // L2 experimental
inline constexpr vtable<"vt_Sess_RaceRuleset", 60> vt_Sess_RaceRuleset{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScAheadOfLead", 11> vt_Sess_ScAheadOfLead{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScCanReturn", 10> vt_Sess_ScCanReturn{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScCarsGrouped", 11> vt_Sess_ScCarsGrouped{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScDebugDeploy", 10> vt_Sess_ScDebugDeploy{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScRaceLapsRemaining", 10> vt_Sess_ScRaceLapsRemaining{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScTriggerRedFlag", 10> vt_Sess_ScTriggerRedFlag{}; // L2 experimental
inline constexpr vtable<"vt_Sess_ScTriggerVsc", 10> vt_Sess_ScTriggerVsc{}; // L2 experimental

}
