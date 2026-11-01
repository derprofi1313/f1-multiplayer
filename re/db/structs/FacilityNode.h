// struct: FacilityNode
// subsystem: career
// size: 0xe8 L2 decomp:re/decomp/140f8a5f0.c;imm:0xe8 (allocated at 0x140f8a61c)
// notes: one facility upgrade (vt_Career_FacilityNode 0x1456af450; a perk-tree node). Career_Complete_PerkTreeNode moves state 1 -> 2 when days_remaining == 0 and purchased. Raised by ce-rnd (the lead authorised creating career-lane structs); see addons/career-editor/protocol/facilities.md.
struct FacilityNode {
    /* 0x00 */ void **vtable;                       // L2 decomp:re/decomp/140f8a5f0.c;decomp:re/decomp/140fa5470.c;xref:0x140f8a6e4,0x140fa5489 vt_Career_FacilityNode
    /* 0x08 */ uint8_t state;                       // L2 decomp:re/decomp/142ea5fe0.c;decomp:re/decomp/140f80f60.c;xref:0x142ea5fe8,0x140f811ad 0 locked, 1 unlocked, 2 complete
    /* 0x0c */ uint32_t stat_id;                    // L2 decomp:re/decomp/140f8a5f0.c;xref:0x140f8a694,0x140f8a6d9 node statId
    /* 0x20 */ void *children_begin;              // L2 decomp:re/decomp/142ea5fe0.c;decomp:re/decomp/142ea60b0.c;xref:0x142ea6010,0x142ea60e0 child nodes as 16-byte shared_ptrs {FacilityNode *, control block}; Career_Complete_PerkTreeNode unlocks them, 0x142ea60b0 walks them depth-first
    /* 0x28 */ void *children_end;                // L2 decomp:re/decomp/142ea5fe0.c;decomp:re/decomp/142ea60b0.c;xref:0x142ea6014,0x142ea60e4 end of the child vector
    /* 0x40 */ int32_t phase;                       // L2 decomp:re/decomp/140f8a800.c;xref:0x140f8a842 gamedb phase
    /* 0x80 */ int32_t cost;                        // L2 decomp:re/decomp/140f8a800.c;xref:0x140f8a88b gamedb cost
    /* 0x88 */ char *name_loc_tag;                  // L2 decomp:re/decomp/140f8a800.c;decomp:re/decomp/140f8d8d0.c;xref:0x140f8a9b1,0x140f8d94f language key; logged by the claim-rewards slot
    /* 0xa8 */ int32_t days_to_complete;            // L2 decomp:re/decomp/140f8a800.c;xref:0x140f8a864 gamedb days_to_complete
    /* 0xac */ int32_t days_remaining;              // L2 decomp:re/decomp/140f80f60.c;decomp:re/decomp/140f8e5d0.c;xref:0x140f811b3,0x140f811bf,0x140f8e5da counted down daily while purchased; isComplete needs 0
    /* 0xb0 */ uint8_t purchased;                   // L2 decomp:re/decomp/140f80f60.c;decomp:re/decomp/140f8a1a0.c;xref:0x140f811a4,0x140f8a243 bought (the daily pass only counts purchased nodes)
    /* 0xb3 */ uint8_t seen_completed;              // L2 decomp:re/decomp/140f8a1a0.c;decomp:re/decomp/140f8ece0.c;xref:0x140f8a252,0x140f8ee93 completion ceremony seen
    /* 0xdc */ int32_t ui_level;                    // L2 decomp:re/decomp/140f8a800.c;xref:0x140f8b2a3 gamedb ui_level
    /* 0xe0 */ int32_t ui_max_level;                // L2 decomp:re/decomp/140f8a800.c;xref:0x140f8b2c8 gamedb ui_max_level
};
