// struct: StandingsSession
// subsystem: career
// size: 0x28 L2 decomp:re/decomp/1413cd970.c;xref:0x1413cd9c6 (add r15,0x28 while searching the session)
// notes: one session of a StandingsRound: {allocator, records_begin, records_end, capacity, session_id}; records are StandingsResult (results) or StandingsPoints (points). Raised by ce-season with the lead's authorisation.
struct StandingsSession {
    /* 0x008 */ void *records_begin;                 // L2 decomp:re/decomp/1413cd970.c;decomp:re/decomp/1413cdff0.c;xref:0x1413cd9d4 the session's records
    /* 0x010 */ void *records_end;                   // L2 decomp:re/decomp/1413cd970.c;decomp:re/decomp/1413cdff0.c;xref:0x1413cd9d8 end of the records
    /* 0x020 */ uint32_t session_id;                 // L2 decomp:re/decomp/1413cd970.c;decomp:re/decomp/1413cdff0.c;xref:0x1413cd9c0 the Session.session_id it belongs to (0 = none)
};
