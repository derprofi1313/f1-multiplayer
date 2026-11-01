// struct: RnDDepartmentConfig
// subsystem: career
// size: ?
// notes: R&D "tweakables" config object, filled by Career_Load_RnDTweakables (0x14353cea0) at [arg0+0x68]. Holds AI recommended-upgrade chances, cancel-refund percentages and development-delay tables. Offsets are relative to the config object base. See subsystems/career.md#tweakables.
struct RnDDepartmentConfig {
    /* 0x090 */ float target_mass_adjustment_at_max_upgrade; // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables field "target_mass_adjustment_at_max_upgrade"
    /* 0x094 */ float difficulty_mass_adjustment_scalar;     // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables field "difficulty_mass_adjustment_scalar"
    /* 0x098 */ uint8_t difficulty_mass_adjustment_scalar_enabled; // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables field "difficulty_mass_adjustment_scalar_enabled"
    /* 0x09c */ float pct_chance_pick_vehicle_upgrade;       // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables recommended_upgrade.percentage_chance_of_picking_vehicle_upgrade
    /* 0x0a0 */ float pct_chance_pick_durability_upgrade;    // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables recommended_upgrade.percentage_chance_of_picking_durability_upgrade
    /* 0x0a4 */ float min_pct_upgrades_adapted;              // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables recommended_upgrade.min_percentage_of_upgrades_that_can_be_adapted
    /* 0x0a8 */ float max_pct_upgrades_adapted;              // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables recommended_upgrade.max_percentage_of_upgrades_that_can_be_adapted
    /* 0x0ac */ float cancelling_research_refund_pct;        // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables cancelling_research.refund_percentage
    /* 0x0b0 */ float cancelling_development_refund_pct;     // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables cancelling_development.refund_percentage
    /* 0x0b4 */ float cancelling_adaptation_refund_pct;      // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables cancelling_adaptation.refund_percentage
    /* 0x0c0 */ int32_t development_delay_table[16];         // L1 decomp:re/decomp/14353cea0.c {int32 threshold; uint16 days} pairs (stride 8) from development_delay.thresholds/.days; length is development_delay_count
    /* 0x100 */ int64_t development_delay_count;             // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables number of development_delay entries written
    /* 0x110 */ int32_t development_delay_cash_penalty;      // L2 decomp:re/decomp/14353cea0.c;kb:subsystems/career.md#tweakables development_delay.worst_case_cash_penalty
};
