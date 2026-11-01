// struct: VehicleIdentity
// subsystem: entrants
// size: ? L2 decomp:re/decomp/1425027e0.c
// notes: the per-vehicle identity byte block inside the ~0x1110-byte AI-config vehicle object built
// by Sess_Load_VehicleAiConfig (0x1425027e0) from an entrants.vehicles row. These four bytes carry
// the car's engine-supplier, vehicle-package and performance identity. Each is a stat_id resolved by
// label: vehicle_package -> entrants.vehicles.vehicle_packages.<pkg>.stat_id; engine ->
// entrants.engines.<engine>.stat_id; performance_class / performance_tier are read directly. Offsets
// are from the AI-config object base (rsi in the decomp); the block overlaps the car lane's vehicle
// object. Byte +0xcd8 inits to 0xff, +0xcd9 inits to 0 (dword).
struct VehicleIdentity {
    /* 0xcd8 */ uint8_t vehicle_package_stat_id;   // L2 decomp:re/decomp/1425027e0.c;xref:0x142502c51 entrants.vehicles.vehicle_packages.<pkg>.stat_id (0xff = none)
    /* 0xcd9 */ uint8_t engine_supplier_stat_id;   // L2 decomp:re/decomp/1425027e0.c;xref:0x142502a74 entrants.engines.<engine>.stat_id (engine-supplier mapping; 0 = none)
    /* 0xcda */ uint8_t performance_class;          // L2 decomp:re/decomp/1425027e0.c;xref:0x142503e3d vehicle performance_class byte (car-class filter key in Sess_Build_CandidatePool)
    /* 0xcdb */ uint8_t performance_tier;           // L2 decomp:re/decomp/1425027e0.c;xref:0x142503e63 vehicle performance_tier byte
};
