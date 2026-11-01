#pragma once
#include <swofty/game/base.hpp>

namespace swofty::game {

inline constexpr address<"Render_BackBuffer_CopyToCpu"> Render_BackBuffer_CopyToCpu{}; // L2 experimental
inline constexpr address<"Render_Batch_Submit"> Render_Batch_Submit{}; // L2 experimental
inline constexpr address<"Render_CloudGenerator_Update"> Render_CloudGenerator_Update{}; // L2 experimental
inline constexpr address<"Render_DDGISystem_Render"> Render_DDGISystem_Render{}; // L2 experimental
inline constexpr address<"Render_DamageMaskGenerator_Render"> Render_DamageMaskGenerator_Render{}; // L2 experimental
inline constexpr address<"Render_Gfx2Buffer_Construct"> Render_Gfx2Buffer_Construct{}; // L2 experimental
inline constexpr address<"Render_Gfx2Buffer_Destroy"> Render_Gfx2Buffer_Destroy{}; // L2 experimental
inline constexpr address<"Render_Gfx2Device_Initialise"> Render_Gfx2Device_Initialise{}; // L2 experimental
inline constexpr address<"Render_GroundCoverPlugin_Render"> Render_GroundCoverPlugin_Render{}; // L2 experimental
inline constexpr address<"Render_Hbcn_OnCompressRequest"> Render_Hbcn_OnCompressRequest{}; // L2 experimental
inline constexpr address<"Render_LightListResource_Create"> Render_LightListResource_Create{}; // L2 experimental
inline constexpr address<"Render_Model_ValidateMaterialParams"> Render_Model_ValidateMaterialParams{}; // L2 experimental
inline constexpr address<"Render_OnRaytracingValidationMsg"> Render_OnRaytracingValidationMsg{}; // L2 experimental
inline constexpr address<"Render_ParticleManager_DrawMeshParticles"> Render_ParticleManager_DrawMeshParticles{}; // L2 experimental
inline constexpr address<"Render_Particles_ReadbackVertexBuffer"> Render_Particles_ReadbackVertexBuffer{}; // L2 experimental
inline constexpr address<"Render_RainFile_Load"> Render_RainFile_Load{}; // L2 experimental
inline constexpr address<"Render_RainFile_ParseMaterials"> Render_RainFile_ParseMaterials{}; // L2 experimental
inline constexpr address<"Render_SceneView_SubmitCamera"> Render_SceneView_SubmitCamera{}; // L2 experimental
inline constexpr address<"Render_Screenshot_CaptureAndSave"> Render_Screenshot_CaptureAndSave{}; // L2 experimental
inline constexpr address<"Render_TerrainPlugin_Render"> Render_TerrainPlugin_Render{}; // L2 experimental
inline constexpr address<"Render_Texture_CreateFromResource"> Render_Texture_CreateFromResource{}; // L2 experimental
inline constexpr address<"Render_Write_PngFile"> Render_Write_PngFile{}; // L2 experimental
inline constexpr address<"Render_XeSS_Evaluate"> Render_XeSS_Evaluate{}; // L2 experimental
inline constexpr address<"Render_XeSS_PrepareInputs"> Render_XeSS_PrepareInputs{}; // L2 experimental

}
