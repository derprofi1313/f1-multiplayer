#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

struct NeAllocator {
    NeAllocatorVtbl *vtable; // L2 experimental
};
static_assert(offsetof(NeAllocator, vtable) == 0x0);

struct NeAllocatorVtbl {
    uint8_t _pad_0x0[0x18];
    void *alloc; // L2 experimental
    uint8_t _pad_0x20[0x18];
    void *free; // L2 experimental
};
static_assert(offsetof(NeAllocatorVtbl, alloc) == 0x18);
static_assert(offsetof(NeAllocatorVtbl, free) == 0x38);

struct NeBagCodec {
    void **vtable; // L3 stable
    void *stream; // L3 stable
};
static_assert(offsetof(NeBagCodec, vtable) == 0x0);
static_assert(offsetof(NeBagCodec, stream) == 0x8);

struct NeCountingStream {
    void **vtable; // L3 stable
    uint8_t _pad_0x8[0x20];
    uint64_t count; // L3 stable
};
static_assert(offsetof(NeCountingStream, vtable) == 0x0);
static_assert(offsetof(NeCountingStream, count) == 0x28);

struct NeLogGate {
    uint32_t channel_hashes[373]; // L2 experimental
    uint8_t enabled; // L2 experimental
};
static_assert(offsetof(NeLogGate, channel_hashes) == 0x0);
static_assert(offsetof(NeLogGate, enabled) == 0x5d4);

struct NeMemWriteStream {
    void **vtable; // L3 stable
    uint8_t *data; // L3 stable
    uint64_t capacity; // L3 stable
    uint64_t position; // L3 stable
    uint64_t size; // L2 experimental
};
static_assert(offsetof(NeMemWriteStream, vtable) == 0x0);
static_assert(offsetof(NeMemWriteStream, data) == 0x8);
static_assert(offsetof(NeMemWriteStream, capacity) == 0x10);
static_assert(offsetof(NeMemWriteStream, position) == 0x18);
static_assert(offsetof(NeMemWriteStream, size) == 0x20);

struct NeMessageBus {
    uint8_t _pad_0x0[0x68];
    uint8_t active; // L2 experimental
};
static_assert(offsetof(NeMessageBus, active) == 0x68);

struct NeOptionalVec {
    NeAllocator *alloc; // L2 experimental
    void *begin; // L2 experimental
    void *end; // L2 experimental
    uint64_t count; // L2 experimental
    uint8_t engaged; // L2 experimental
    uint8_t _pad_0x21[0x7];
};
static_assert(offsetof(NeOptionalVec, alloc) == 0x0);
static_assert(offsetof(NeOptionalVec, begin) == 0x8);
static_assert(offsetof(NeOptionalVec, end) == 0x10);
static_assert(offsetof(NeOptionalVec, count) == 0x18);
static_assert(offsetof(NeOptionalVec, engaged) == 0x20);
static_assert(sizeof(NeOptionalVec) == 0x28);

struct NePropertyTypeVtbl {
    uint8_t _pad_0x0[0x10];
    void *id; // L3 stable
    uint8_t _pad_0x18[0x8];
    void *key_offset; // L3 stable
};
static_assert(offsetof(NePropertyTypeVtbl, id) == 0x10);
static_assert(offsetof(NePropertyTypeVtbl, key_offset) == 0x20);

struct NePropertyVtbl {
    uint8_t _pad_0x0[0x8];
    void *type; // L3 stable
    uint8_t _pad_0x10[0x28];
    void *value; // L3 stable
};
static_assert(offsetof(NePropertyVtbl, type) == 0x8);
static_assert(offsetof(NePropertyVtbl, value) == 0x38);

struct NeSharedCtrl {
    void **vtable; // L2 experimental
    int32_t strong; // L2 experimental
    int32_t weak; // L2 experimental
};
static_assert(offsetof(NeSharedCtrl, vtable) == 0x0);
static_assert(offsetof(NeSharedCtrl, strong) == 0x8);
static_assert(offsetof(NeSharedCtrl, weak) == 0xc);

struct NeStreamVtbl {
    uint8_t _pad_0x0[0x48];
    void *write; // L3 stable
    void *write_length; // L3 stable
};
static_assert(offsetof(NeStreamVtbl, write) == 0x48);
static_assert(offsetof(NeStreamVtbl, write_length) == 0x50);

struct SaveDataContainer {
    void **vtable; // L3 stable
    uint8_t _pad_0x8[0x78];
    const char *name; // L3 stable
    uint8_t _pad_0x88[0x78];
    uint8_t retain_unknown; // L2 experimental
    uint8_t _pad_0x101[0x7];
    uint64_t retain_unknown_key; // L2 experimental
    uint8_t _pad_0x110[0xb8];
    uint64_t retained_count; // L2 experimental
};
static_assert(offsetof(SaveDataContainer, vtable) == 0x0);
static_assert(offsetof(SaveDataContainer, name) == 0x80);
static_assert(offsetof(SaveDataContainer, retain_unknown) == 0x100);
static_assert(offsetof(SaveDataContainer, retain_unknown_key) == 0x108);
static_assert(offsetof(SaveDataContainer, retained_count) == 0x1c8);

inline constexpr address<"Ne_ActorSceneDataResourceFactory_DestroyResource"> Ne_ActorSceneDataResourceFactory_DestroyResource{}; // L2 experimental
inline constexpr address<"Ne_ActorSceneDataResource_DestroyScenes"> Ne_ActorSceneDataResource_DestroyScenes{}; // L2 experimental
inline constexpr address<"Ne_BufferedAssetGroupLoader_GroupSwapAssetGroups"> Ne_BufferedAssetGroupLoader_GroupSwapAssetGroups{}; // L2 experimental, NeBufferedAssetGroupLoader::groupSwapAssetGroups
inline constexpr address<"Ne_CloudGenerator_Destroy"> Ne_CloudGenerator_Destroy{}; // L2 experimental, NeCloudGenerator::~NeCloudGenerator
inline constexpr address<"Ne_CloudUploader_Upload"> Ne_CloudUploader_Upload{}; // L2 experimental
using Ne_Compare_StringNoCase_t = int32_t(const char *a, const char *b);
inline constexpr fn<"Ne_Compare_StringNoCase", Ne_Compare_StringNoCase_t> Ne_Compare_StringNoCase{}; // L2 experimental, _stricmp
inline constexpr address<"Ne_CrowdAnimationLoader_LoadBehavioursInStage"> Ne_CrowdAnimationLoader_LoadBehavioursInStage{}; // L2 experimental, NeCrowdAnimationLoader::loadBehavioursInStage
inline constexpr address<"Ne_CrowdAnimationLoader_OnAnimClipChanged"> Ne_CrowdAnimationLoader_OnAnimClipChanged{}; // L2 experimental, NeCrowdAnimationLoader::onAnimClipChanged
inline constexpr address<"Ne_CrowdPluginConnection_ProcessMessage"> Ne_CrowdPluginConnection_ProcessMessage{}; // L2 experimental, NeCrowdPluginConnection::processMessage
inline constexpr address<"Ne_CrowdPlugin_LoadXml"> Ne_CrowdPlugin_LoadXml{}; // L2 experimental
inline constexpr address<"Ne_CustomisationDecalRenderer_Render"> Ne_CustomisationDecalRenderer_Render{}; // L2 experimental, NeCustomisationDecalRenderer::render
inline constexpr address<"Ne_CustomisationDecalsProjectionAreasSetPlugin_CloseConnection"> Ne_CustomisationDecalsProjectionAreasSetPlugin_CloseConnection{}; // L2 experimental, NeCustomisationDecalsProjectionAreasSetPlugin::closeConnection
inline constexpr address<"Ne_CustomisationDecalsSetPlugin_CloseConnection"> Ne_CustomisationDecalsSetPlugin_CloseConnection{}; // L2 experimental, NeCustomisationDecalsSetPlugin::closeConnection
inline constexpr address<"Ne_CutsceneScriptResourceFactory_CreateResourceFinished"> Ne_CutsceneScriptResourceFactory_CreateResourceFinished{}; // L2 experimental
inline constexpr address<"Ne_CutsceneScriptResourceFactory_DestroyResource"> Ne_CutsceneScriptResourceFactory_DestroyResource{}; // L2 experimental
inline constexpr address<"Ne_CutsceneScriptResourceFactory_StartDestroyResource"> Ne_CutsceneScriptResourceFactory_StartDestroyResource{}; // L2 experimental
using Ne_Detach_ChildNode_t = int(void *parent, void *child);
inline constexpr fn<"Ne_Detach_ChildNode", Ne_Detach_ChildNode_t> Ne_Detach_ChildNode{}; // L2 experimental
inline constexpr address<"Ne_EadpErrorsDataService_UninstallExceptionHandler"> Ne_EadpErrorsDataService_UninstallExceptionHandler{}; // L2 experimental
inline constexpr address<"Ne_EadpGameInviteList_DeleteSessionInvite"> Ne_EadpGameInviteList_DeleteSessionInvite{}; // L2 experimental, neon::NeEadpGameInviteList::deleteSessionInvite
inline constexpr address<"Ne_EadpGameInviteList_OnInviteValidated"> Ne_EadpGameInviteList_OnInviteValidated{}; // L2 experimental, neon::NeEadpGameInviteList::onInviteValidated
inline constexpr address<"Ne_EadpGameInviteList_UpdateInviteLifetime"> Ne_EadpGameInviteList_UpdateInviteLifetime{}; // L2 experimental, neon::NeEadpGameInviteList::updateInviteLifetime
inline constexpr address<"Ne_EadpPresenceService_UnsubscribeFriends"> Ne_EadpPresenceService_UnsubscribeFriends{}; // L2 experimental, neon::NeEadpPresenceService::unsubscribeFriends
using Ne_Entity_FindComponent_t = void *(void *entity, const void *type);
inline constexpr fn<"Ne_Entity_FindComponent", Ne_Entity_FindComponent_t> Ne_Entity_FindComponent{}; // L2 experimental
inline constexpr address<"Ne_EventDispatcher_SendEvent"> Ne_EventDispatcher_SendEvent{}; // L2 experimental
inline constexpr address<"Ne_FogSet_SetFogData"> Ne_FogSet_SetFogData{}; // L2 experimental, NeFogSet::setFogData
using Ne_Format_String_t = int64_t(NeStr **out, const char *fmt, ...);
inline constexpr fn<"Ne_Format_String", Ne_Format_String_t> Ne_Format_String{}; // L2 experimental
using Ne_Get_SettingBool_t = uint64_t(void *node, const char *name, uint8_t *out, uint8_t def);
inline constexpr fn<"Ne_Get_SettingBool", Ne_Get_SettingBool_t> Ne_Get_SettingBool{}; // L2 experimental
using Ne_Get_SettingU32_t = uint64_t(void *node, const char *name, uint32_t *out, uint32_t def);
inline constexpr fn<"Ne_Get_SettingU32", Ne_Get_SettingU32_t> Ne_Get_SettingU32{}; // L2 experimental
inline constexpr address<"Ne_IBLCubeProbe_FreeInterpolatedArrays"> Ne_IBLCubeProbe_FreeInterpolatedArrays{}; // L2 experimental
inline constexpr address<"Ne_InputDevicePadPS_Cleanup"> Ne_InputDevicePadPS_Cleanup{}; // L2 experimental, NeInputDevicePadPS::cleanup
inline constexpr address<"Ne_InputDevicePadPS_Initialise"> Ne_InputDevicePadPS_Initialise{}; // L2 experimental, NeInputDevicePadPS::initialise
inline constexpr address<"Ne_InputLibraryDirectInput_EnumDevicesCallback"> Ne_InputLibraryDirectInput_EnumDevicesCallback{}; // L2 experimental, NeInputLibraryDirectInput::enumDevicesCallback
inline constexpr address<"Ne_InputLibraryDirectInput_IsXInputDevice"> Ne_InputLibraryDirectInput_IsXInputDevice{}; // L2 experimental, NeInputLibraryDirectInput::isXInputDevice
inline constexpr address<"Ne_InputSystem_DeviceConnection"> Ne_InputSystem_DeviceConnection{}; // L2 experimental, NeInputSystem::deviceConnection
inline constexpr address<"Ne_InputSystem_MergePreset"> Ne_InputSystem_MergePreset{}; // L2 experimental
inline constexpr address<"Ne_InputSystem_UnbindDevice"> Ne_InputSystem_UnbindDevice{}; // L2 experimental
inline constexpr address<"Ne_LinkResolver_ResolveLinks"> Ne_LinkResolver_ResolveLinks{}; // L2 experimental, NeLinkResolver::resolveLinks
using Ne_Log_Thunk_t = void(const char *channel, const char *fmt, ...);
inline constexpr fn<"Ne_Log_Thunk", Ne_Log_Thunk_t> Ne_Log_Thunk{}; // L2 experimental
using Ne_Make_StrFromCStr_t = uint32_t(NeStr **out, const char *s, uint32_t flags, uintptr_t len);
inline constexpr fn<"Ne_Make_StrFromCStr", Ne_Make_StrFromCStr_t> Ne_Make_StrFromCStr{}; // L3 stable
inline constexpr address<"Ne_MipTailGeneratorFactory_ObtainGeneratorResource"> Ne_MipTailGeneratorFactory_ObtainGeneratorResource{}; // L2 experimental, NeMipTailGeneratorFactory::obtainGeneratorResource
inline constexpr address<"Ne_MipTailGenerator_Construct"> Ne_MipTailGenerator_Construct{}; // L2 experimental, NeMipTailGenerator::NeMipTailGenerator
inline constexpr address<"Ne_MipTailGenerator_CreateResources"> Ne_MipTailGenerator_CreateResources{}; // L2 experimental, NeMipTailGenerator::createResources
inline constexpr address<"Ne_MipTailGenerator_Destroy"> Ne_MipTailGenerator_Destroy{}; // L2 experimental, NeMipTailGenerator::~NeMipTailGenerator
inline constexpr address<"Ne_MipTailGenerator_Record"> Ne_MipTailGenerator_Record{}; // L2 experimental, NeMipTailGenerator::record
inline constexpr address<"Ne_ModuleSystem_CheckConfig"> Ne_ModuleSystem_CheckConfig{}; // L2 experimental
inline constexpr address<"Ne_ModuleSystem_CheckPlatform"> Ne_ModuleSystem_CheckPlatform{}; // L2 experimental
inline constexpr address<"Ne_NetConnectionQuality_ConnectionArrived"> Ne_NetConnectionQuality_ConnectionArrived{}; // L2 experimental, neon::NeNetConnectionQuality::connectionArrived
inline constexpr address<"Ne_NetConnectionQuality_ConnectionLeft"> Ne_NetConnectionQuality_ConnectionLeft{}; // L2 experimental, neon::NeNetConnectionQuality::connectionLeft
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthEvalPayload"> Ne_NetConnectionQuality_OnBandwidthEvalPayload{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthEvalPayload
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthEvalResultMessage"> Ne_NetConnectionQuality_OnBandwidthEvalResultMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthEvalResultMessage
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthEvalStartMessage"> Ne_NetConnectionQuality_OnBandwidthEvalStartMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthEvalStartMessage
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthEvalStartReplyMessage"> Ne_NetConnectionQuality_OnBandwidthEvalStartReplyMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthEvalStartReplyMessage
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthTesterStateRequestMessage"> Ne_NetConnectionQuality_OnBandwidthTesterStateRequestMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthTesterStateRequestMessage
inline constexpr address<"Ne_NetConnectionQuality_OnBandwidthTesterStateUpdateMessage"> Ne_NetConnectionQuality_OnBandwidthTesterStateUpdateMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onBandwidthTesterStateUpdateMessage
inline constexpr address<"Ne_NetConnectionQuality_OnStatsUpdateMessage"> Ne_NetConnectionQuality_OnStatsUpdateMessage{}; // L2 experimental, neon::NeNetConnectionQuality::onStatsUpdateMessage
inline constexpr address<"Ne_NetConnectionQuality_SendAllStatsUpdates"> Ne_NetConnectionQuality_SendAllStatsUpdates{}; // L2 experimental, neon::NeNetConnectionQuality::sendAllStatsUpdates
inline constexpr address<"Ne_NetConnectionQuality_SendBandwidthTesterStateUpdateMessage"> Ne_NetConnectionQuality_SendBandwidthTesterStateUpdateMessage{}; // L2 experimental, neon::NeNetConnectionQuality::sendBandwidthTesterStateUpdateMessage
inline constexpr address<"Ne_NetConnectionQuality_SendSingleStatsUpdate"> Ne_NetConnectionQuality_SendSingleStatsUpdate{}; // L2 experimental, neon::NeNetConnectionQuality::sendSingleStatsUpdate
inline constexpr address<"Ne_NetConnectionQuality_SetTesterState"> Ne_NetConnectionQuality_SetTesterState{}; // L2 experimental, neon::NeNetConnectionQuality::setTesterState
inline constexpr address<"Ne_NetConnectionStore_Initialise"> Ne_NetConnectionStore_Initialise{}; // L2 experimental, neon::NeNetConnectionStore::initialise
inline constexpr address<"Ne_NetHttpRequestHandler_CreateRequest"> Ne_NetHttpRequestHandler_CreateRequest{}; // L2 experimental, NeNetHttpRequestHandler::createRequest
inline constexpr address<"Ne_NetNullSSLPeer_Update"> Ne_NetNullSSLPeer_Update{}; // L2 experimental, neon::NeNetNullSSLPeer::update
inline constexpr address<"Ne_NetReliableAsyncSocket_DisconnectAll"> Ne_NetReliableAsyncSocket_DisconnectAll{}; // L2 experimental, neon::NeNetReliableAsyncSocket::DisconnectAll
inline constexpr address<"Ne_NetReliableAsyncSocket_OnHandshakeCompleted"> Ne_NetReliableAsyncSocket_OnHandshakeCompleted{}; // L2 experimental, NeNetReliableAsyncSocket::onHandshakeCompleted
inline constexpr address<"Ne_NetReliableAsyncSocket_OnMessageDecrypted"> Ne_NetReliableAsyncSocket_OnMessageDecrypted{}; // L2 experimental, neon::NeNetReliableAsyncSocket::onMessageDecrypted
inline constexpr address<"Ne_NetReliableAsyncSocket_OnMessageEncrypted"> Ne_NetReliableAsyncSocket_OnMessageEncrypted{}; // L2 experimental, neon::NeNetReliableAsyncSocket::onMessageEncrypted
inline constexpr address<"Ne_NetReliableAsyncSocket_Restart"> Ne_NetReliableAsyncSocket_Restart{}; // L2 experimental, NeNetReliableAsyncSocket::Restart
inline constexpr address<"Ne_NetReliableAsyncSocket_SetKeyMaterial"> Ne_NetReliableAsyncSocket_SetKeyMaterial{}; // L2 experimental, neon::NeNetReliableAsyncSocket::setKeyMaterial
inline constexpr address<"Ne_NetSessionF1OS_JoinSession"> Ne_NetSessionF1OS_JoinSession{}; // L2 experimental, neon::NeNetSessionF1OS::joinSession
inline constexpr address<"Ne_NetSessionF1OS_MigrateSessionClient"> Ne_NetSessionF1OS_MigrateSessionClient{}; // L2 experimental, NeNetSessionF1OS::migrateSessionClient
inline constexpr address<"Ne_NetSessionF1OS_MigrateSessionHost"> Ne_NetSessionF1OS_MigrateSessionHost{}; // L2 experimental, neon::NeNetSessionF1OS::migrateSessionHost
inline constexpr address<"Ne_NetSessionF1OS_SetCrossplay"> Ne_NetSessionF1OS_SetCrossplay{}; // L2 experimental, neon::NeNetSessionF1OS::setCrossplay
inline constexpr address<"Ne_NetSessionF1OS_SetSessionName"> Ne_NetSessionF1OS_SetSessionName{}; // L2 experimental, neon::NeNetSessionF1OS::setSessionName
inline constexpr address<"Ne_NetSessionLAN_MigrateSessionClient"> Ne_NetSessionLAN_MigrateSessionClient{}; // L2 experimental, NeNetSessionLAN::migrateSessionClient
inline constexpr address<"Ne_NetSessionLAN_MigrateSessionHost"> Ne_NetSessionLAN_MigrateSessionHost{}; // L2 experimental, NeNetSessionLAN::migrateSessionHost
inline constexpr address<"Ne_NetSessionLAN_OnCreateMigratedSessionFinished"> Ne_NetSessionLAN_OnCreateMigratedSessionFinished{}; // L2 experimental, NeNetSessionLAN::onCreateMigratedSessionFinished
inline constexpr address<"Ne_NetSessionSearcherLAN_SearchForSessions"> Ne_NetSessionSearcherLAN_SearchForSessions{}; // L2 experimental, NeNetSessionSearcherLAN::searchForSessions
inline constexpr address<"Ne_NetSyncClock_IntegrateRemoteHostClock"> Ne_NetSyncClock_IntegrateRemoteHostClock{}; // L2 experimental, NeNetSyncClock::integrateRemoteHostClock
inline constexpr address<"Ne_NetSyncClock_RecalculateClockOffset"> Ne_NetSyncClock_RecalculateClockOffset{}; // L2 experimental, neon::NeNetSyncClock::recalculateClockOffset
inline constexpr address<"Ne_NetSystemClassicMatchmaking_CreateSession"> Ne_NetSystemClassicMatchmaking_CreateSession{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::createSession
inline constexpr address<"Ne_NetSystemClassicMatchmaking_MigrateSession"> Ne_NetSystemClassicMatchmaking_MigrateSession{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::migrateSession
inline constexpr address<"Ne_NetSystemClassicMatchmaking_NotifyPeersMigrationResult"> Ne_NetSystemClassicMatchmaking_NotifyPeersMigrationResult{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::notifyPeersMigrationResult
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnCreateSessionComplete"> Ne_NetSystemClassicMatchmaking_OnCreateSessionComplete{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onCreateSessionComplete
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnHandshakingComplete"> Ne_NetSystemClassicMatchmaking_OnHandshakingComplete{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onHandshakingComplete
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnJoinSessionComplete"> Ne_NetSystemClassicMatchmaking_OnJoinSessionComplete{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onJoinSessionComplete
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnMigrateSessionClient"> Ne_NetSystemClassicMatchmaking_OnMigrateSessionClient{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onMigrateSessionClient
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnSessionError"> Ne_NetSystemClassicMatchmaking_OnSessionError{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onSessionError
inline constexpr address<"Ne_NetSystemClassicMatchmaking_OnSessionSearchComplete"> Ne_NetSystemClassicMatchmaking_OnSessionSearchComplete{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::onSessionSearchComplete
inline constexpr address<"Ne_NetSystemClassicMatchmaking_ResetNetworkSystem"> Ne_NetSystemClassicMatchmaking_ResetNetworkSystem{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::resetNetworkSystem
inline constexpr address<"Ne_NetSystemClassicMatchmaking_ResetNetworkSystemCompleted"> Ne_NetSystemClassicMatchmaking_ResetNetworkSystemCompleted{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::resetNetworkSystemCompleted
inline constexpr address<"Ne_NetSystemClassicMatchmaking_SearchForSessions"> Ne_NetSystemClassicMatchmaking_SearchForSessions{}; // L2 experimental, neon::NeNetSystemClassicMatchmaking::searchForSessions
inline constexpr address<"Ne_NetSystemCommon_HandleSendToConnection"> Ne_NetSystemCommon_HandleSendToConnection{}; // L2 experimental, neon::NeNetSystemCommon::handleSendToConnection
inline constexpr address<"Ne_NetSystemCommon_LeaveSession"> Ne_NetSystemCommon_LeaveSession{}; // L2 experimental, neon::NeNetSystemCommon::leaveSession
inline constexpr address<"Ne_NetSystemCommon_Migrate"> Ne_NetSystemCommon_Migrate{}; // L2 experimental, neon::NeNetSystemCommon::migrate
inline constexpr address<"Ne_NetSystemCommon_OnConnectionArrived"> Ne_NetSystemCommon_OnConnectionArrived{}; // L2 experimental, neon::NeNetSystemCommon::onConnectionArrived
inline constexpr address<"Ne_NetSystemCommon_ResetNetworkSystem"> Ne_NetSystemCommon_ResetNetworkSystem{}; // L2 experimental, neon::NeNetSystemCommon::resetNetworkSystem
inline constexpr address<"Ne_NetSystemCommon_ResetNetworkSystemCompleted"> Ne_NetSystemCommon_ResetNetworkSystemCompleted{}; // L2 experimental, neon::NeNetSystemCommon::resetNetworkSystemCompleted
inline constexpr address<"Ne_NetSystemCommon_SendKeepAliveToConnections"> Ne_NetSystemCommon_SendKeepAliveToConnections{}; // L2 experimental, neon::NeNetSystemCommon::sendKeepAliveToConnections
inline constexpr address<"Ne_NetSystemHelloMessage_Deserialise"> Ne_NetSystemHelloMessage_Deserialise{}; // L2 experimental, neon::NeNetSystemHelloMessage::deserialise
inline constexpr address<"Ne_NetSystemHelloMessage_Serialise"> Ne_NetSystemHelloMessage_Serialise{}; // L2 experimental, neon::NeNetSystemHelloMessage::serialise
inline constexpr address<"Ne_NetSystemServerHelloResponseMessage_Deserialise"> Ne_NetSystemServerHelloResponseMessage_Deserialise{}; // L2 experimental, neon::NeNetSystemServerHelloResponseMessage::deserialise
inline constexpr address<"Ne_NetSystemServerHelloResponseMessage_Serialise"> Ne_NetSystemServerHelloResponseMessage_Serialise{}; // L2 experimental, neon::NeNetSystemServerHelloResponseMessage::serialise
inline constexpr address<"Ne_NetTransportCommon_AddHostConnectionInfoToConnectionStore"> Ne_NetTransportCommon_AddHostConnectionInfoToConnectionStore{}; // L2 experimental, NeNetTransportCommon::addHostConnectionInfoToConnectionStore
inline constexpr address<"Ne_NetTransportCommon_BroadcastNewConnectionToClients"> Ne_NetTransportCommon_BroadcastNewConnectionToClients{}; // L2 experimental, neon::NeNetTransportCommon::broadcastNewConnectionToClients
inline constexpr address<"Ne_NetTransportCommon_HandleClientHello"> Ne_NetTransportCommon_HandleClientHello{}; // L2 experimental, neon::NeNetTransportCommon::handleClientHello
inline constexpr address<"Ne_NetTransportCommon_InitialiseClient"> Ne_NetTransportCommon_InitialiseClient{}; // L2 experimental, neon::NeNetTransportCommon::initialiseClient
inline constexpr address<"Ne_NetTransportCommon_Migrate"> Ne_NetTransportCommon_Migrate{}; // L2 experimental, neon::NeNetTransportCommon::migrate
inline constexpr address<"Ne_NetTransportCommon_ReportClientInitialConnections"> Ne_NetTransportCommon_ReportClientInitialConnections{}; // L2 experimental, neon::NeNetTransportCommon::reportClientInitialConnections
inline constexpr address<"Ne_NetTransportCommon_StartHandshaking"> Ne_NetTransportCommon_StartHandshaking{}; // L2 experimental, neon::NeNetTransportCommon::startHandshaking
inline constexpr address<"Ne_NetTransportLAN_RegisterArrivedConnection"> Ne_NetTransportLAN_RegisterArrivedConnection{}; // L2 experimental, NeNetTransportLAN::registerArrivedConnection
inline constexpr address<"Ne_NetTransportPlayFab_MigrateConnections"> Ne_NetTransportPlayFab_MigrateConnections{}; // L2 experimental, neon::NeNetTransportPlayFab::migrateConnections
inline constexpr address<"Ne_NetTransportPlayFab_SendTo"> Ne_NetTransportPlayFab_SendTo{}; // L2 experimental, neon::NeNetTransportPlayFab::sendTo
inline constexpr address<"Ne_NetTransportPlayFab_SendToConnectionInfo"> Ne_NetTransportPlayFab_SendToConnectionInfo{}; // L2 experimental, neon::NeNetTransportPlayFab::sendToConnectionInfo
inline constexpr address<"Ne_NetVoiceNotificationClient_WaveInAddBuffer"> Ne_NetVoiceNotificationClient_WaveInAddBuffer{}; // L2 experimental, NeNetVoiceNotificationClient::waveInAddBuffer
inline constexpr address<"Ne_Network_Init"> Ne_Network_Init{}; // L2 experimental, neon::NeNetwork::init
inline constexpr address<"Ne_Network_StartAsync"> Ne_Network_StartAsync{}; // L2 experimental, neon::NeNetwork::startAsync
using Ne_Parse_Long_t = int32_t(const char *text, char **end, int32_t base);
inline constexpr fn<"Ne_Parse_Long", Ne_Parse_Long_t> Ne_Parse_Long{}; // L2 experimental, strtol
using Ne_PhysBody_ApplyImpulse_t = void(void *body, const float *J, const float *worldPoint);
inline constexpr fn<"Ne_PhysBody_ApplyImpulse", Ne_PhysBody_ApplyImpulse_t> Ne_PhysBody_ApplyImpulse{}; // L2 experimental
inline constexpr address<"Ne_Phys_ParseMaterials"> Ne_Phys_ParseMaterials{}; // L2 experimental
using Ne_Play_FullscreenVideo_t = void(void *feature_system, NeStr **path, const void *on_end, const void *frame_callbacks, int32_t layer);
inline constexpr fn<"Ne_Play_FullscreenVideo", Ne_Play_FullscreenVideo_t> Ne_Play_FullscreenVideo{}; // L2 experimental, neon::NeUIGenericFeatureSystem::playFullscreenVideo
inline constexpr address<"Ne_PlayerManager_HandleLeadPlayerLogoutOrChangeCallback"> Ne_PlayerManager_HandleLeadPlayerLogoutOrChangeCallback{}; // L2 experimental, NePlayerManager::handleLeadPlayerLogoutOrChangeCallback
inline constexpr address<"Ne_Player_AddPlayerInputProvider"> Ne_Player_AddPlayerInputProvider{}; // L2 experimental, NePlayer::addPlayerInputProvider
inline constexpr address<"Ne_Player_DestroyCustomInputProvider"> Ne_Player_DestroyCustomInputProvider{}; // L2 experimental, NePlayer::destroyCustomInputProvider
inline constexpr address<"Ne_Player_DisableCustomControls"> Ne_Player_DisableCustomControls{}; // L2 experimental, NePlayer::disableCustomControls
inline constexpr address<"Ne_Player_RemovePlayerInputProvider"> Ne_Player_RemovePlayerInputProvider{}; // L2 experimental, NePlayer::removePlayerInputProvider
inline constexpr address<"Ne_Player_SetFFBInputDeviceId"> Ne_Player_SetFFBInputDeviceId{}; // L2 experimental, NePlayer::setFFBInputDeviceId
using Ne_Post_BusMessage_t = uint64_t(void *bus, void *msg);
inline constexpr fn<"Ne_Post_BusMessage", Ne_Post_BusMessage_t> Ne_Post_BusMessage{}; // L2 experimental
using Ne_SaveSystem2Win_Mount_t = uint32_t(void *device, void *op, void *save, uint8_t for_saving, uint8_t a5, uint8_t *created);
inline constexpr fn<"Ne_SaveSystem2Win_Mount", Ne_SaveSystem2Win_Mount_t> Ne_SaveSystem2Win_Mount{}; // L2 experimental
using Ne_SaveSystem2Win_Start_t = uint64_t(void *device);
inline constexpr fn<"Ne_SaveSystem2Win_Start", Ne_SaveSystem2Win_Start_t> Ne_SaveSystem2Win_Start{}; // L2 experimental
inline constexpr address<"Ne_SaveSystem2Win_WriteInfoXml"> Ne_SaveSystem2Win_WriteInfoXml{}; // L2 experimental
inline constexpr address<"Ne_SaveSystem_BuildSaveInfoList"> Ne_SaveSystem_BuildSaveInfoList{}; // L2 experimental
using Ne_SaveSystem_InitDevice_t = int(void *save_system);
inline constexpr fn<"Ne_SaveSystem_InitDevice", Ne_SaveSystem_InitDevice_t> Ne_SaveSystem_InitDevice{}; // L2 experimental
inline constexpr address<"Ne_SequenceAudioStreamPacket_PreloadAudioStream"> Ne_SequenceAudioStreamPacket_PreloadAudioStream{}; // L2 experimental, NeSequenceAudioStreamPacket::preloadAudioStream
inline constexpr address<"Ne_SequenceClipInstance_BindOptional"> Ne_SequenceClipInstance_BindOptional{}; // L2 experimental, NeSequenceClipInstance::bindOptional
inline constexpr address<"Ne_SequenceResource_CreateResource"> Ne_SequenceResource_CreateResource{}; // L2 experimental, NeSequenceResource::createResource
using Ne_StringTable_Lookup_t = const char *(void *table, uint64_t hash);
inline constexpr fn<"Ne_StringTable_Lookup", Ne_StringTable_Lookup_t> Ne_StringTable_Lookup{}; // L3 stable
inline constexpr address<"Ne_TerrainResource_GetMaterialResourcel"> Ne_TerrainResource_GetMaterialResourcel{}; // L2 experimental, NeTerrainResource::getMaterialResourcel
inline constexpr address<"Ne_TextureDecompressionGeneratorFactory_ObtainGeneratorResource"> Ne_TextureDecompressionGeneratorFactory_ObtainGeneratorResource{}; // L2 experimental, NeTextureDecompressionGeneratorFactory::obtainGeneratorResource
inline constexpr address<"Ne_Thread_Destroy"> Ne_Thread_Destroy{}; // L2 experimental, NeThread::~NeThread
inline constexpr address<"Ne_UICrashDumpInfo_SettingsScreenClearSelectedCategory"> Ne_UICrashDumpInfo_SettingsScreenClearSelectedCategory{}; // L2 experimental
inline constexpr address<"Ne_UIPCOptions_PopulateOutputMonitors"> Ne_UIPCOptions_PopulateOutputMonitors{}; // L2 experimental, NeUIPCOptions::populateOutputMonitors
inline constexpr address<"Ne_VideoResource_Destroy"> Ne_VideoResource_Destroy{}; // L2 experimental, NeVideoResource::~NeVideoResource
inline constexpr address<"Ne_WindowWin_MonitorEnumProc"> Ne_WindowWin_MonitorEnumProc{}; // L2 experimental, NeWindowWin::monitorEnumProc
inline constexpr address<"Ne_WindowWin_ResizeTarget"> Ne_WindowWin_ResizeTarget{}; // L2 experimental, NeWindowWin::resizeTarget
inline constexpr address<"Ne_WindowWin_SaveWindowPosition"> Ne_WindowWin_SaveWindowPosition{}; // L2 experimental, NeWindowWin::saveWindowPosition
inline constexpr address<"Ne_WindowWin_SetAspectRatio"> Ne_WindowWin_SetAspectRatio{}; // L2 experimental, NeWindowWin::setAspectRatio
inline constexpr address<"Ne_WindowWin_SetOnMinimised"> Ne_WindowWin_SetOnMinimised{}; // L2 experimental, NeWindowWin::setOnMinimised
inline constexpr address<"Ne_WindowWin_Startup"> Ne_WindowWin_Startup{}; // L2 experimental, NeWindowWin::startup
inline constexpr address<"Ne_WindowWin_ToggleFullScreen"> Ne_WindowWin_ToggleFullScreen{}; // L2 experimental, NeWindowWin::toggleFullScreen
inline constexpr address<"Ne_WindowWin_Update"> Ne_WindowWin_Update{}; // L2 experimental, NeWindowWin::update
inline constexpr address<"Ne_WindowWin_WinProc"> Ne_WindowWin_WinProc{}; // L2 experimental, NeWindowWin::winProc
inline constexpr data<"g_Ne_DefaultAllocator", void *> g_Ne_DefaultAllocator{}; // L2 experimental
inline constexpr data<"g_Ne_LogGate", NeLogGate> g_Ne_LogGate{}; // L2 experimental
inline constexpr data<"g_Ne_StringTable", void *> g_Ne_StringTable{}; // L3 stable
inline constexpr vtable<"k_Ne_BagCodecVtbl", 4> k_Ne_BagCodecVtbl{}; // L3 stable
inline constexpr vtable<"k_Ne_CountingStreamVtbl", 24> k_Ne_CountingStreamVtbl{}; // L3 stable
inline constexpr vtable<"k_Ne_MemWriteStreamVtbl", 33> k_Ne_MemWriteStreamVtbl{}; // L3 stable
inline constexpr data<"k_Ne_PropertyType_Bag", void *> k_Ne_PropertyType_Bag{}; // L3 stable
inline constexpr data<"k_Ne_TypeChainEnd", void *> k_Ne_TypeChainEnd{}; // L2 experimental
inline constexpr data<"k_Ne_TypeRoot", void *> k_Ne_TypeRoot{}; // L2 experimental, NoOtiInfo.
inline constexpr site<"site_Ne_Format_String_Entry"> site_Ne_Format_String_Entry{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_CrowdInShadows"> site_Ne_HwBool_CrowdInShadows{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_GodRaysPost"> site_Ne_HwBool_GodRaysPost{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_GrassFlattening"> site_Ne_HwBool_GrassFlattening{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_GroundCoverEnabled"> site_Ne_HwBool_GroundCoverEnabled{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_GroundCoverMeshes"> site_Ne_HwBool_GroundCoverMeshes{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_LensStreak"> site_Ne_HwBool_LensStreak{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_MotionBlurV2"> site_Ne_HwBool_MotionBlurV2{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_NightShadowCharCast"> site_Ne_HwBool_NightShadowCharCast{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_RainBeads"> site_Ne_HwBool_RainBeads{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_RainHigh"> site_Ne_HwBool_RainHigh{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_RainSheets"> site_Ne_HwBool_RainSheets{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_RainSplashes"> site_Ne_HwBool_RainSplashes{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_SkyShadowLowLod"> site_Ne_HwBool_SkyShadowLowLod{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_SkyShadowMap32Bits"> site_Ne_HwBool_SkyShadowMap32Bits{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_TrackHeatHaze"> site_Ne_HwBool_TrackHeatHaze{}; // L2 experimental
inline constexpr call<"site_Ne_HwBool_VolumetricFog"> site_Ne_HwBool_VolumetricFog{}; // L2 experimental
inline constexpr call<"site_Ne_HwConfigParseCall"> site_Ne_HwConfigParseCall{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_CrowdBillboardDistance"> site_Ne_HwInt_CrowdBillboardDistance{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_NightShadowMapSize"> site_Ne_HwInt_NightShadowMapSize{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_NightShadowSourceCount"> site_Ne_HwInt_NightShadowSourceCount{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_ShadowSampling"> site_Ne_HwInt_ShadowSampling{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_SkyShadowCascadeCount"> site_Ne_HwInt_SkyShadowCascadeCount{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_SkyShadowMapSize"> site_Ne_HwInt_SkyShadowMapSize{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_SpotShadowMapSize"> site_Ne_HwInt_SpotShadowMapSize{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_SpotShadowSampling"> site_Ne_HwInt_SpotShadowSampling{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_TexStreamSizeInMiB"> site_Ne_HwInt_TexStreamSizeInMiB{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_VolFogMapHeight"> site_Ne_HwInt_VolFogMapHeight{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_VolFogMapWidth"> site_Ne_HwInt_VolFogMapWidth{}; // L2 experimental
inline constexpr call<"site_Ne_HwInt_WorldShadowMapSize"> site_Ne_HwInt_WorldShadowMapSize{}; // L2 experimental
inline constexpr call<"site_Ne_HwXmlParseCall"> site_Ne_HwXmlParseCall{}; // L2 experimental
inline constexpr call<"site_Ne_HwXmlWriteCall"> site_Ne_HwXmlWriteCall{}; // L2 experimental
inline constexpr site<"site_Ne_ReplayAfterRecord"> site_Ne_ReplayAfterRecord{}; // L2 experimental
inline constexpr vtable<"vt_Ne_ParticleEffectComponent", 16> vt_Ne_ParticleEffectComponent{}; // L2 experimental
inline constexpr vtable<"vt_Ne_SaveSystem2Win", 16> vt_Ne_SaveSystem2Win{}; // L2 experimental

}
