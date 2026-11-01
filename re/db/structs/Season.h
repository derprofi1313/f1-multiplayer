// struct: Season
// subsystem: career
// size: 0x550 L2 decomp:re/decomp/140ae4c70.c (the championship's create-season method allocates 0x550 and builds it inline)
// notes: game::championship::Season (Season:: log strings). Owns the calendar (Weekend* list) and the embedded standings model (F1SeasonStandings at +0x2a8). Raised by ce-season with the lead's authorisation.
struct Season {
    /* 0x000 */ void **vtable;                       // L2 decomp:re/decomp/140ae4c70.c;decomp:re/decomp/140b6eab0.c;xref:0x140ae4f79 vt_Career_Season
    /* 0x220 */ char *name;                          // L2 decomp:re/decomp/140af1980.c;decomp:re/decomp/140ae9650.c;xref:0x140af1aa4,0x140ae98f2 points at the inline name buffer (+0x228); Championship serialise writes it
    /* 0x280 */ NeAllocator *allocator;              // L2 decomp:re/decomp/140ae6050.c;decomp:re/decomp/140af1980.c;xref:0x140ae60f6,0x140af19db allocates and frees the season's Weekends (vt+0x18 / vt+0x38)
    /* 0x288 */ Weekend **weekends_begin;            // L2 decomp:re/decomp/140af1230.c;decomp:re/decomp/140af1980.c;decomp:re/decomp/140ae6160.c;xref:0x140af1420,0x140af1999,0x140ae6194 one Weekend per round, saved as weekend_names[] in this order
    /* 0x290 */ Weekend **weekends_end;              // L2 decomp:re/decomp/140af1230.c;decomp:re/decomp/140af1980.c;decomp:re/decomp/140ae6160.c;xref:0x140af1427,0x140af19a0,0x140ae618a end of the round list
    /* 0x2a0 */ uint32_t current_weekend;            // L2 decomp:re/decomp/140af1230.c;decomp:re/decomp/140af1980.c;decomp:re/decomp/140ae8880.c;xref:0x140af132a,0x140af1a08,0x140ae88c1 saved as current_weekend (0-based round)
    /* 0x2a8 */ void **standings_vtable;             // L2 decomp:re/decomp/140ae8040.c;decomp:re/decomp/140ae8880.c;xref:0x140ae8079 first qword of the embedded F1SeasonStandings (drivers); its address is the standings object
    /* 0x538 */ uint32_t completion_state;           // L2 decomp:re/decomp/140af1230.c;decomp:re/decomp/140af1980.c;xref:0x140af12b5,0x140af1b10 saved as completion_state (u8 in the save)
};
