// struct: TeamRenownData
// subsystem: career
// size: 0x140 L2 decomp:re/decomp/140f95a10.c (allocated 0x140 bytes before the ctor 0x140f97c20)
// notes: one team's renown and form object in MyTeamRFComponent (Career_Get_TeamRenownData hands it out as a weak ref). Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct TeamRenownData {
    /* 0x000 */ uint32_t team;                       // L2 decomp:re/decomp/140f98cb0.c;xref:0x140f98ce1 team key (kind byte at +0, stat id u16 at +2)
    /* 0x004 */ float points;                        // L2 decomp:re/decomp/140f98cb0.c;decomp:re/decomp/140f95a10.c;decomp:re/decomp/1414c5f80.c;xref:0x140f98cf9,0x140f98d3f,0x1414c60d5 renown points (saved as renown_points)
    /* 0x018 */ RenownConfig * config;               // L2 decomp:re/decomp/140f98cb0.c;decomp:re/decomp/1414c5f80.c;xref:0x140f98cff,0x1414c60c4 tier configuration
};
