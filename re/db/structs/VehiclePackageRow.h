// struct: VehiclePackageRow
// subsystem: car
// size: ? L2 decomp:re/decomp/142504a90.c
// notes: output of Game_Read_VehiclePackageRow(entrants.vehicles vehicle_N). Strings (NeStr-ish) at
// +0x08 resource_path, +0x18 resource_path_specialisation, +0x28 blob_shadow_decal, +0x38/+0x50
// asset_group / ai_package_folder / driver_animation_set.
struct VehiclePackageRow {
    /* 0x0000 */ uint8_t stat_id;                            // L2 decomp:re/decomp/142504a90.c;kb:subsystems/car.md#vehicle-package initialised 0xff
    /* 0x0060 */ uint8_t has_cockpit_halo;                   // L2 decomp:re/decomp/142504a90.c;kb:subsystems/car.md#vehicle-package
    /* 0x0061 */ uint8_t is_story_vehicle_package;           // L2 decomp:re/decomp/142504a90.c;kb:subsystems/car.md#vehicle-package
    /* 0x0062 */ uint8_t is_equal_performance_vehicle_package; // L2 decomp:re/decomp/142504a90.c;kb:subsystems/car.md#vehicle-package equal-performance (ranked/esports) flag
    /* 0x0063 */ uint8_t is_26_aero_regulations;             // L2 decomp:re/decomp/142504a90.c;kb:subsystems/car.md#vehicle-package 2026 aero ruleset flag
};
