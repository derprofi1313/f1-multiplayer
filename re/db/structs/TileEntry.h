// struct: TileEntry
// subsystem: fe_tiles
// size: 0x10 L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c
// notes: element of TileList.items, pushed by FeTile_ItemVec_PushBack_Thunk
struct TileEntry {
    /* 0x000 */ void *node;             // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 VM node handle of button_N
    /* 0x008 */ Tile *tile;             // L3 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.4;trace:T000/run1:56 core mod reads end[-1].tile
};
