// struct: TileList
// subsystem: fe_tiles
// size: ?
// notes: 'this' of FeTile_Build_List; r14 in the home case. Handle widths at +0x38/+0x58 unproven.
struct TileList {
    /* 0x038 */ void *buttons_node;     // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 VM 'buttons' node (djb2 0x94eb0e14)
    /* 0x058 */ void *selected_button_node; // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 VM 'selected_button' node
    /* 0x070 */ NeAllocator *items_alloc; // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 vector allocator
    /* 0x078 */ TileEntry *items_begin; // L3 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1;trace:T000/run1:56 items 2 -> 3 computed from begin/end
    /* 0x080 */ TileEntry *items_end;   // L3 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1;trace:T000/run1:56
    /* 0x088 */ TileEntry *items_cap;   // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1
};
