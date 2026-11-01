// struct: AiHandlingConfig
// subsystem: ai
// size: 0x710  L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c (0xe2 read slots x 8 bytes)
// notes: per-vehicle AI race-craft config. Filled by Sess_Parse_AiHandlingConfig (session, 0x1406828a0) as a flat table of 0xe2 Gdb field-node pointers (one per key, Gdb_Get_FieldInt by NAME); the float value of each key is at *(node+8). The read ORDER is the slot index (not grouped). Two 8-slot merge-rate heat maps are filled by Ai_Parse_MergeHeatMap (0x140682580) at slots 0x77 (steering) and 0x7f (vehicle). One field is pre-resolved to a float: exit_fallibility.max_yaw_angle_deg -> radians at 0x318. This is a sub-block of the ~0x1110-byte struct that Sess_Load_VehicleAiConfig (0x1425027e0) builds per vehicle. FULL slot->key table (30 mechanic groups, 221 keys) is in subsystems/ai.md#vocabulary. Landmark slots only below.
struct AiHandlingConfig {
    /* 0x000 */ void *allow_to_pass_begin;      // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c allow_to_pass.* (overtake/let-by: 16 slots 0x00-0x0f)
    /* 0x080 */ void *blocking_begin;           // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c blocking.* (defending: 5 slots 0x10-0x14)
    /* 0x0a8 */ void *buffer_space_begin;       // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c buffer_space_settings.* (follow gap: 5 slots 0x15-0x19)
    /* 0x0d0 */ void *celebration_begin;        // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c celebration.* (3 slots 0x1a-0x1c)
    /* 0x0e8 */ void *doughnut_begin;           // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c doughnut.* (10 slots 0x1d-0x26)
    /* 0x138 */ void *damage_settings_begin;    // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c damage_settings.* (6 slots 0x27-0x2c)
    /* 0x220 */ void *ers_use_begin;            // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c ers_use.* push-to-overtake/defend/build-gap battery levels (26 slots 0x44-0x5d)
    /* 0x318 */ float exit_fallibility_max_yaw_angle_rad; // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c exit_fallibility.max_yaw_angle_deg converted to radians (/57.29578); mistake/slide limit
    /* 0x3b8 */ void *merge_steering_heatmap_begin; // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c merge_rates.steering_heat_map.* (8 slots 0x77-0x7e, via Ai_Parse_MergeHeatMap which=1)
    /* 0x3f8 */ void *merge_vehicle_heatmap_begin;  // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c merge_rates.vehicle_heat_map.* (8 slots 0x7f-0x86, via Ai_Parse_MergeHeatMap which=0)
    /* 0x708 */ void *last_slot;                // L1 decomp:work/exe/decomp/1406828a0_drafting_parse.c slot 0xe1 (traction_control / trailbraking / safety-car / yielding tail block)
};
