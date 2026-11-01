// struct: EntrantSetupMode
// subsystem: session
// size: 0x288 L2 decomp:re/decomp/14249f190.c parsed by Sess_Parse_EntrantSetupMode; copied into a 0x290 heap object
struct EntrantSetupMode {
    /* 0x020 */ void *entrant_lists[32]; // L2 decomp:re/decomp/14249f190.c;doc:research/grid24-cap.md#2
    /* 0x120 */ uint64_t entrant_list_count; // L2 decomp:re/decomp/14249f190.c;doc:research/grid24-cap.md#2 qword (V1007)
    /* 0x130 */ void *selectable_lists[32]; // L2 decomp:re/decomp/14249f190.c;decomp:re/decomp/141a79180.c;doc:research/grid24-cap.md#2 Sess_Setup_ChampionshipEntrants uses [0]; the parser appends with no bound check
    /* 0x230 */ uint64_t selectable_list_count; // L2 decomp:re/decomp/14249f190.c;decomp:re/decomp/141a79180.c;doc:research/grid24-cap.md#2 qword; read by Game_F1WorldActivityContainer_SelectEntrantList and the era tile availability check (V1007)
    /* 0x264 */ int32_t max_teams;      // L2 decomp:re/decomp/14249f190.c;xref:0x1424a0351;doc:research/grid24-cap.md#2 -1 = no team prune (compared unsigned)
    /* 0x268 */ int32_t max_entrants;   // L2 decomp:re/decomp/14249f190.c;xref:0x1424a0348,0x1424a3075;doc:research/grid24-cap.md#2 value < 1 stored as -1
    /* 0x270 */ uint8_t selection_order[16]; // L1 doc:research/grid24-cap.md#2 length unknown
    /* 0x280 */ uint8_t reuse_entrants; // L2 decomp:re/decomp/1424a2350.c;xref:0x1424a307c;doc:research/grid24-cap.md#2 0 -> limit = min(max_entrants, candidates)
};
