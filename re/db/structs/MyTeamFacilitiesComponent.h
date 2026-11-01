// struct: MyTeamFacilitiesComponent
// subsystem: career
// size: ?
// notes: MyTeamFacilitiesComponent (vtable 0x1456aeeb0), My Team only; found through ce::comp::get(MyTeamFacilities). Keeps per-team facility data in a node map: slots (8 bytes each, pointing at a FacilityTeamEntry), then one control byte per slot (non-zero = used) starting where the slot array ends, and the element count. The serialiser 0x140f84230 and the daily pass Career_Tick_FacilityNodes 0x140f80f60 iterate it the same way. Raised by ce-rnd (the lead authorised creating career-lane structs).
struct MyTeamFacilitiesComponent {
    /* 0x2b8 */ FacilityTeamEntry **team_slots;     // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f80f60.c;xref:0x140f8434a,0x140f80fdf slot array of the team map
    /* 0x2c0 */ uint8_t *team_ctrl;                 // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f80f60.c;xref:0x140f84351,0x140f80fe6 control bytes, one per slot; also the end of the slot array
    /* 0x2c8 */ uint64_t team_count;                // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f80f60.c;xref:0x140f84337,0x140f80fd1 number of used slots
};
