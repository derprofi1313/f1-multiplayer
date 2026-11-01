// struct: UpgradeEffectDef
// subsystem: car
// size: ? L2 decomp:re/decomp/143514670.c
// notes: an effect definition inside DepartmentData.effect_defs; also the param_3 of
// Car_Apply_UpgradeToVehicle. Grouped by two enum bytes against the fitted-upgrade group/item bytes.
struct UpgradeEffectDef {
    /* 0x0064 */ float effect_value;      // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply summed over matched defs to form the rating ratio
    /* 0x0072 */ uint8_t item_group_enum; // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply matched == fitted-upgrade item byte [item+0]
    /* 0x007c */ uint8_t dept_group_enum; // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply matched == upgrade group byte [group+0]; 0xff gates a Car_Apply_UpgradeToVehicle branch
    /* 0x00a4 */ uint32_t applied_value;   // L1 decomp:re/decomp/1435173e0.c;kb:subsystems/car.md#apply written by Car_Apply_UpgradeToVehicle from Car_Read_VehicleAttribute
    /* 0x00a8 */ uint32_t applied_value2;  // L1 decomp:re/decomp/1435173e0.c;kb:subsystems/car.md#apply written by Car_Apply_UpgradeToVehicle
    /* 0x00c8 */ void *variant_tag_ptr;    // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply byte at [variant_tag_ptr-8] is a sub-pass type tag (0/3/6)
};
