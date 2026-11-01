// struct: FacilityTeamData
// subsystem: career
// size: 0x80 L2 decomp:re/decomp/140f84f20.c;imm:0x80 (the deserialiser allocates it, 0x140f8537f)
// notes: one team's facilities. Raised by ce-rnd (the lead authorised creating career-lane structs).
struct FacilityTeamData {
    /* 0x10 */ void *facilities_begin;              // L2 decomp:re/decomp/140f80f60.c;decomp:re/decomp/140f84230.c;xref:0x140f810db,0x140f84568 facilities as 16-byte shared pointers {MyTeamFacility *, control block}
    /* 0x18 */ void *facilities_end;                // L2 decomp:re/decomp/140f80f60.c;decomp:re/decomp/140f84230.c;xref:0x140f810e0,0x140f8456c end of the facility vector
    /* 0x28 */ uint32_t team_key;                   // L2 decomp:re/decomp/140f84f20.c;xref:0x140f853e4 packed team key built from team_stat_id ((id & 0x3fff) << 16 | id >> 14)
};
