// struct: ChampionshipSeasonEntry
// subsystem: career
// size: 0x10 L2 decomp:re/decomp/140ae9650.c;decomp:re/decomp/140ae9b10.c (both walk Championship.seasons_begin..seasons_end in steps of 0x10)
// notes: one element of Championship.seasons; +0x0 is not read by the season code. Raised by ce-season with the lead's authorisation.
struct ChampionshipSeasonEntry {
    /* 0x008 */ Season *season;                      // L2 decomp:re/decomp/140ae9650.c;decomp:re/decomp/140ae9b10.c;xref:0x140ae98ee the Season; its name (+0x220) is what Championship serialise writes
};
