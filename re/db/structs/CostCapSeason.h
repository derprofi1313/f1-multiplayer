// struct: CostCapSeason
// subsystem: career
// size: ?
// notes: one season's cost cap entry of a team (season_cost_cap_data). Raised by ce-econ (career lane owner career-re is stale; the lead authorised --force).
struct CostCapSeason {
    /* 0x000 */ int32_t season_id;                   // L2 decomp:re/decomp/140c9af30.c;xref:0x140c9b0e3,0x140c9b0f6 key of the season map (the registry's current season)
    /* 0x008 */ int32_t cached_expenses;             // L2 decomp:re/decomp/140c9af30.c;decomp:re/decomp/140c9d650.c;xref:0x140c9b12e spending counted against the cap
    /* 0x00c */ int32_t cost_cap;                    // L2 decomp:re/decomp/140c9d650.c;xref:0x140c9dde3 the cap (serialised third)
    /* 0x010 */ float cached_modifier;               // L2 decomp:re/decomp/140c9af30.c;decomp:re/decomp/140c9d650.c;xref:0x140c9b124 multiplier applied to each counted transaction amount before it is subtracted from cached_expenses (eFacilityRunningCost, eFacilityUpgradeEngineering, eUpgradeDevelopment with check_funds=1; eDamageRefund with check_funds=0)
};
