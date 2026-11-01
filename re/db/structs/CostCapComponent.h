// struct: CostCapComponent
// subsystem: career
// size: ?
// notes: CostCapComponent (vtable 0x145686390, My Team only); per-team flat map at +0x2c8. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct CostCapComponent {
    /* 0x2d8 */ CostCapTeam ** team_slots;           // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9afc7 slot array of the per-team flat map
    /* 0x2e0 */ uint8_t * team_ctrl;                 // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9afce end of the slots = control bytes, one per slot
};
