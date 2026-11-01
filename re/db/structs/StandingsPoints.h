// struct: StandingsPoints
// subsystem: career
// size: 0x8 L2 decomp:re/decomp/1413cdff0.c;xref:0x1413ce4ba (add [rbp-0x18],8 per points record)
// notes: one driver's points in a StandingsSession of F1SeasonStandings.points, and the element of the extra-points list. Raised by ce-season with the lead's authorisation.
struct StandingsPoints {
    /* 0x000 */ uint32_t key;                        // L2 decomp:re/decomp/1413cdff0.c;decomp:re/decomp/1413ccb90.c;xref:0x1413ce4b4 entrant key
    /* 0x004 */ int32_t points;                      // L2 decomp:re/decomp/1413cdff0.c;decomp:re/decomp/140b2fc20.c;xref:0x1413ce4b6 points for that session (points table + fastest-lap bonus)
};
