// struct: StandingsResult
// subsystem: career
// size: 0x10 L2 decomp:re/decomp/1413cdff0.c;xref:0x1413ce4bf (add r12,0x10 over the result records)
// notes: one driver's result in a StandingsSession of F1SeasonStandings.results, written by Season::applyResultsToStandings through 0x1413cd970. Raised by ce-season with the lead's authorisation.
struct StandingsResult {
    /* 0x000 */ uint32_t key;                        // L2 decomp:re/decomp/1413cdff0.c;decomp:re/decomp/1413cd970.c;xref:0x1413ce4b0 entrant key (EntrantRecord +0x50: id_lo, driver stat id in the high word)
    /* 0x004 */ int32_t position;                    // L2 decomp:re/decomp/1413cdff0.c;xref:0x1413ce533,0x1413ce5b6 0-based finishing position (index into the points table)
    /* 0x008 */ int32_t fastest_lap_rank;            // L2 decomp:re/decomp/1413cdff0.c;xref:0x1413ce5ae 0 = fastest lap (bonus when position < 10)
    /* 0x00c */ uint32_t status;                     // L2 decomp:re/decomp/1413cdff0.c;xref:0x1413ce4db scores points when < 2
};
