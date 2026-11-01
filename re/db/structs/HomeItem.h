// struct: HomeItem
// subsystem: fe_tiles
// size: 0x148 L2 decomp:work/exe/decomp/141746710_tile_career.c alloc(0x148, 0x10)
// notes: tile item; base vtable vt_FeTile_ItemBase, per-tile vtable stored by the adder. Nine VmPropHolder bound by FeTile_ItemBase_Populate.
struct HomeItem {
    /* 0x000 */ ItemVtbl *vtable;       // L3 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#3.2;trace:T000/run1:55 Career item carries vt_FeHome_CareerItem
    /* 0x010 */ NeStr *id;              // L2 decomp:work/exe/decomp/14173da50_item_a_getaction.c;doc:research/exe.md#3.2 id and route key ("career"); generic getAction returns it
    /* 0x018 */ VmPropHolder button_node; // L3 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2;trace:T000/run1:57 holder node (item+0x20) non-null after base populate
    /* 0x038 */ VmPropHolder name;      // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 string = id
    /* 0x058 */ VmPropHolder title;     // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 string loc tag
    /* 0x078 */ VmPropHolder title_localised; // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 bool, default true
    /* 0x098 */ VmPropHolder description; // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 string
    /* 0x0b8 */ VmPropHolder description_localised; // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 bool, default true
    /* 0x0d8 */ VmPropHolder enabled;   // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 bool, default true
    /* 0x0f8 */ VmPropHolder notification_type; // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 int
    /* 0x118 */ VmPropHolder tutorial_ref; // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 node ref
    /* 0x138 */ uint64_t unk_0x138;     // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 zeroed
    /* 0x140 */ uint64_t unk_0x140;     // L2 decomp:work/exe/decomp/141697b90_item_base_populate.c;doc:research/exe.md#3.2 zeroed
};
