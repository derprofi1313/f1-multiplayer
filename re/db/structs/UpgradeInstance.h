// struct: UpgradeInstance
// subsystem: career
// size: 0xe0 L2 decomp:re/decomp/14352e1c0.c;decomp:re/decomp/14352cac0.c;imm:0xe0 (Career_ResetAll_Upgrades steps the instance vector by 0xe0 at 0x14352e228)
// notes: one upgrade of a team's tree. Career_Instantiate_UpgradeTrees copies the 0x80-byte UpgradeRecord into the first 0x80 bytes (same offsets as UpgradeRecord), then the instance keeps its research/development progress and its state history. Raised by ce-rnd (the lead authorised creating career-lane structs); see subsystems/rnd_ui.md#instances and addons/career-editor/protocol/rnd.md.
struct UpgradeInstance {
    /* 0x10 */ char *name;                          // L2 decomp:re/decomp/1435230e0.c;xref:0x14352339d upgrade_name chars (SSO pointer, inline at +0x18 for short names); logged by Career_Push_UpgradeState
    /* 0x58 */ uint32_t node_hash;                  // L2 decomp:re/decomp/1435168d0.c;decomp:re/decomp/1435320f0.c;xref:0x143516d5e,0x143516d98 djb2 node hash; the save keys instances by it and mode 2 mirrors by it
    /* 0x5c */ int32_t cost;                        // L2 decomp:re/decomp/143525b60.c;decomp:re/decomp/143539bc0.c;xref:0x143525b95,0x143539e3a gamedb research cost (RP), read by Career_Calc_UpgradeCost
    /* 0x60 */ int32_t cost_to_develop;             // L2 decomp:re/decomp/143525b60.c;decomp:re/decomp/143539bc0.c;xref:0x143525b95,0x143539e3a gamedb development cost; Career_Calc_UpgradeCost reads [inst + idx*4 + 0x5c] with idx = option & 6 ? 1 : 0
    /* 0x68 */ int32_t num_days_to_research;        // L2 decomp:re/decomp/14352fa70.c;decomp:re/decomp/143539bc0.c;xref:0x14352fbb4,0x143539dd9 gamedb research days
    /* 0x6c */ int32_t num_days_to_develop;         // L2 decomp:re/decomp/143530190.c;xref:0x1435303b7 gamedb development days, read by the development-duration helper
    /* 0x72 */ uint8_t department;                  // L2 decomp:re/decomp/1435230e0.c;xref:0x1435230f3 ENUM A department (0 aero, 1 chassis, 2 durability, 3 powertrain, 4 Fake); state mutators return for > 3
    /* 0x74 */ int32_t unlock_at_facility_level;    // L2 decomp:re/decomp/14352ea40.c;xref:0x14352ea47 facility tier the upgrade needs (lock evaluated against the team's kind-4 modifier)
    /* 0x7c */ uint8_t row_flag;                    // L2 decomp:re/decomp/1435168d0.c;decomp:re/decomp/143525b60.c;xref:0x143516922,0x143525b9a 0xff = team tree row, otherwise an engine-supplier stat id
    /* 0x7e */ uint16_t research_start_day;         // L2 decomp:re/decomp/1435168d0.c;xref:0x143516cce,0x1435170ca day research started (0xffff after reset)
    /* 0x80 */ uint16_t research_end_day;           // L2 decomp:re/decomp/1435168d0.c;decomp:re/decomp/143530f00.c;xref:0x143516cd3,0x1435310f9 day research completes; the daily research tick compares it to today
    /* 0x98 */ uint16_t develop_start_day;          // L2 decomp:re/decomp/1435173e0.c;xref:0x1435175f8,0x1435179d4 day development started
    /* 0x9a */ uint16_t develop_end_day;            // L2 decomp:re/decomp/1435173e0.c;decomp:re/decomp/1435314d0.c;xref:0x143517600,0x14353160e day development completes; the daily development tick compares it to today
    /* 0xc0 */ UpgradeStateEntry *states_begin;     // L2 decomp:re/decomp/1435230e0.c;xref:0x143523108,0x143523193 state history (0x10-byte entries)
    /* 0xc8 */ UpgradeStateEntry *states_end;       // L2 decomp:re/decomp/1435230e0.c;xref:0x14352310f,0x14352319a end of the state history; the last entry is the current state
};
