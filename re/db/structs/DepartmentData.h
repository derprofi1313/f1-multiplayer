// struct: DepartmentData
// subsystem: car
// size: ? L2 decomp:re/decomp/143514670.c (>=0xc08)
// notes: 2nd arg to Car_Fit_UpgradesToRating (game::upgrades::DepartmentData). Holds the department's
// effect definitions; the apply path groups them by enum bytes and lerps a rating.
struct DepartmentData {
    /* 0x0000 */ void *effect_defs[0x180];  // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply inline UpgradeEffectDef* array (capacity 0x180)
    /* 0x0c00 */ uint64_t effect_def_count;  // L2 decomp:re/decomp/143514670.c;kb:subsystems/car.md#apply valid entries in effect_defs (loop bound)
};
