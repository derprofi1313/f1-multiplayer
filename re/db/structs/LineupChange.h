// struct: LineupChange
// subsystem: career
// size: 0x10 L2 decomp:re/decomp/14123b3f0.c (entries stepped 0x10: lea r13,[rsi+count*16+0x10])
// notes: one entry of a MessageDriverMovesFinalised payload, built by Career_Build_LineupChanges. Raised by ce-drivers (the lead authorised --force).
struct LineupChange {
    /* 0x000 */ uint32_t driver;   // L2 decomp:re/decomp/14123b3f0.c;decomp:re/decomp/142457a90.c;xref:0x14123b475 driver key (u8 kind, u16 stat id at +2)
    /* 0x004 */ uint32_t old_team; // L2 decomp:re/decomp/14123b3f0.c;decomp:re/decomp/142457a90.c;xref:0x14123b48b,0x14123b494 team key before (stat id at +6)
    /* 0x008 */ uint32_t new_team; // L2 decomp:re/decomp/14123b3f0.c;decomp:re/decomp/142457a90.c;xref:0x14123b485,0x14123b4ae team key now (stat id at +0xa; 0x3fff = left the grid)
    /* 0x00c */ uint32_t replaced_driver; // L1 decomp:re/decomp/142457a90.c driver key of the driver this one replaced (ce-verify-8 note)
};
