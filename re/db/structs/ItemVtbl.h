// struct: ItemVtbl
// subsystem: fe_tiles
// size: 0x98 L2 pat:19-slots-read-from-the-vtables;doc:research/exe.md#3.2
// notes: slots as function pointers; names of the base implementations are in functions.tsv
struct ItemVtbl {
    /* 0x000 */ void *destroy;          // L2 decomp:work/exe/decomp/14179df70_item_career_dtor.c;doc:research/exe.md#3.2
    /* 0x008 */ void *slot08;           // L1 pat:vtable-slot
    /* 0x010 */ void *slot10;           // L1 pat:vtable-slot
    /* 0x018 */ void *init;             // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 (item, ctx, a3) before populate
    /* 0x020 */ void *populate;         // L3 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1;trace:T000/run1:57 (item, &tile->node, a7)
    /* 0x028 */ void *get_action;       // L3 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3;trace:T000/run1:58 (item, Action *out)
    /* 0x030 */ void *gate_a;           // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3
    /* 0x038 */ void *gate_b;           // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3 returns 2 = allowed while gated
    /* 0x040 */ void *on_activate;      // L1 doc:research/exe.md#3.3
    /* 0x070 */ void *is_selectable;    // L1 doc:research/exe.md#3.3 .data1 body in the base
    /* 0x078 */ void *slot78;           // L1 pat:vtable-slot
    /* 0x080 */ void *slot80;           // L1 pat:vtable-slot
    /* 0x088 */ void *slot88;           // L1 pat:vtable-slot
    /* 0x090 */ void *slot90;           // L1 pat:vtable-slot
};
