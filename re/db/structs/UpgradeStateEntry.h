// struct: UpgradeStateEntry
// subsystem: career
// size: 0x10 L2 decomp:re/decomp/1435230e0.c;imm:0x10 (Career_Push_UpgradeState grows the vector by 0x10 per entry, 0x1435231bf)
// notes: one entry of an upgrade instance's state history (UpgradeInstance.states_begin..states_end); the last entry is the current state. Every reader indexes from the end pointer (negative displacements). States: 0 Unavailable, 1 Available, 2 InProgress, 3 Completed, 4 Failed, 5 Invalid, 6 PendingDevelopment, 7 InDevelopment, 8 PendingFitting. Raised by ce-rnd (the lead authorised creating career-lane structs).
struct UpgradeStateEntry {
    /* 0x00 */ uint32_t owner;                      // L2 decomp:re/decomp/1435230e0.c;decomp:re/decomp/1412414c0.c;xref:0x143523136,0x1412414fb owner key (u8 kind | u16 stat id << 16; 0x3fff0000 = none); the purchase handler compares it to the requesting driver
    /* 0x04 */ uint16_t day;                        // L2 decomp:re/decomp/1435230e0.c;xref:0x1435233fe career day of the transition
    /* 0x08 */ uint8_t state;                       // L2 decomp:re/decomp/1435230e0.c;decomp:re/decomp/14352e620.c;xref:0x14352311f,0x1435233a8,0x14352e687 state (k_Career_UpgradeStateNameTable index)
    /* 0x09 */ uint8_t adaptation;                  // L2 decomp:re/decomp/143523cc0.c;decomp:re/decomp/1412414c0.c;xref:0x143523d2b,0x1412415e4 0 none, 1 needs adaptation, 2 adapted (set by 0x143531a90 at 0x143531bbc), 3 adapting (the purchase handler starts/cancels an adaptation on 1/3)
    /* 0x0c */ uint8_t enabled;                     // L2 decomp:re/decomp/143523510.c;decomp:re/decomp/143530f00.c;xref:0x143523549,0x143530fbd enabled flag; Career_Query_EnabledUpgrades and the daily ticks skip disabled instances
    /* 0x0d */ uint8_t facility_locked;             // L2 decomp:re/decomp/143523730.c;decomp:re/decomp/14352e620.c;xref:0x143523769,0x14352e6a7 facility-locked flag (Career_Set_UpgradeFacilityLocked; mode-0 owner pass recomputes it)
};
