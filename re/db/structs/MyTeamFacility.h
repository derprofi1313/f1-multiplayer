// struct: MyTeamFacility
// subsystem: career
// size: 0x1f8 L2 decomp:re/decomp/140f7cfe0.c;imm:0x1f8 (the facilities loader allocates it, 0x140f7d01d)
// notes: one My Team facility (gamedb career.my_team_facilities.Facility_N). Raised by ce-rnd (the lead authorised creating career-lane structs).
struct MyTeamFacility {
    /* 0x000 */ uint8_t facility_type;              // L2 decomp:re/decomp/140f7cfe0.c;xref:0x140f7d3ef 0 Engineering, 1 Personnel (ci-FNV 0x945c388d0f38cdc3), 2 Corporate (0xd284591e5041e684)
    /* 0x008 */ FacilityPerkTree *tree;             // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f80f60.c;xref:0x140f8112c,0x140f845f6 the facility's upgrade tree
    /* 0x050 */ int32_t workforce_cap;              // L2 decomp:re/decomp/140f84230.c;xref:0x140f848f6 workforce_cap; the serialiser always writes workforce_cap, as +0x50 when workforce_cap_set (+0x54) is set and 0 otherwise
    /* 0x054 */ uint8_t workforce_cap_set;          // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f88be0.c;xref:0x140f84872,0x140f848f0,0x140f88c18 workforce_cap only applies when set (cmovne at 0x140f88c1c replaces the computed cap); saved as workforce_cap_set, and the serialiser always writes workforce_cap, as +0x50 when set and 0 otherwise
    /* 0x1d8 */ int32_t highest_phase_seen;         // L2 decomp:re/decomp/140f84230.c;xref:0x140f84a23 highest_phase_seen
};
