// struct: CostCapTeam
// subsystem: career
// size: ?
// notes: one team's cost cap node (team_cost_cap_data); first u16 is the bank account id key. Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct CostCapTeam {
    /* 0x000 */ uint16_t account;                    // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9b014,0x140c9b02a bank account id (kind << 14 | team stat id), the team map key
    /* 0x030 */ CostCapSeason ** season_slots;       // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9b0a4 slot array of the per-season flat map (map base +0x20)
    /* 0x038 */ uint8_t * season_ctrl;               // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9b0a8 end of the slots = control bytes, one per slot
};
