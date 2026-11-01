// struct: StandingsRound
// subsystem: career
// size: 0x20 L2 decomp:re/decomp/1413cd970.c;xref:0x1413cd9a0 (round index shifted by 5 into F1SeasonStandings.results)
// notes: one round of F1SeasonStandings.points or .results: {allocator, sessions_begin, sessions_end, capacity}. Raised by ce-season with the lead's authorisation.
struct StandingsRound {
    /* 0x008 */ StandingsSession *sessions_begin;    // L2 decomp:re/decomp/1413cd970.c;decomp:re/decomp/1413cdff0.c;xref:0x1413cd9a4 sessions of the round, 0x28 each
    /* 0x010 */ StandingsSession *sessions_end;      // L2 decomp:re/decomp/1413cd970.c;decomp:re/decomp/1413cdff0.c;xref:0x1413cd9a9 end of the sessions
};
