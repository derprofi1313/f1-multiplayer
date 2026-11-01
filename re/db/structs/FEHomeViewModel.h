// struct: FEHomeViewModel
// subsystem: fe_home
// size: ?
// notes: cxx game::FEHomeViewModel; 'vm' of FeHome_Handle_Command
struct FEHomeViewModel {
    /* 0x028 */ void *router;           // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3 passed to Vm_Fire_Route
    /* 0x358 */ TileList *tile_list;    // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3 passed to FeTile_List_GetSelectedItem_Thunk
    /* 0x3f0 */ uint32_t focused_id_hash; // L2 decomp:work/exe/decomp/141bb3c20_fe_home_dispatch.c;doc:research/exe.md#3.3 item->id->hash on focus_changed, 0x1505 when none
};
