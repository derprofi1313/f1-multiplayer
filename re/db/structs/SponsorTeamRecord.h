// struct: SponsorTeamRecord
// subsystem: career
// size: 0x88 L2 decomp:re/decomp/141106870.c (team vector stride 0x88)
// notes: one team's sponsor record in CareerSponsorsComponent. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct SponsorTeamRecord {
    /* 0x008 */ uint32_t team;                       // L2 decomp:re/decomp/141106870.c;xref:0x1411068c0 team key
    /* 0x018 */ SponsorEntry * sponsors_begin;       // L2 decomp:re/decomp/141106870.c;xref:0x1411068d3 sponsors
    /* 0x020 */ SponsorEntry * sponsors_end;         // L2 decomp:re/decomp/141106870.c;xref:0x1411068d7 end of the sponsors
};
