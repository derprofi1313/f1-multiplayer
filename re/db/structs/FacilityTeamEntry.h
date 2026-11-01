// struct: FacilityTeamEntry
// subsystem: career
// size: ?
// notes: one element of MyTeamFacilitiesComponent's team map (key, then the team's facility data). Raised by ce-rnd (the lead authorised creating career-lane structs).
struct FacilityTeamEntry {
    /* 0x08 */ FacilityTeamData *data;              // L2 decomp:re/decomp/140f84230.c;decomp:re/decomp/140f80f60.c;xref:0x140f84475,0x140f810d7 the team's facility data
};
