// struct: Tile
// subsystem: fe_tiles
// size: 0xd8 L2 decomp:work/exe/decomp/141746710_tile_career.c alloc(0xd8, 0x10)
// notes: built by FeTile_Construct_Tile; vtables vt_FeTile_TileBase then vt_FeTile_Tile
struct Tile {
    /* 0x000 */ void **vtable;          // L2 decomp:work/exe/decomp/1415a92d0_tile_base_ctor.c;doc:research/exe.md#3.2
    /* 0x008 */ void *node;             // L2 decomp:work/exe/decomp/14173df80_home_buttons_build.c;doc:research/exe.md#3.1 &tile->node is passed to item populate
    /* 0x0c8 */ NeAllocator *allocator; // L2 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#3.2
    /* 0x0d0 */ HomeItem *item;         // L3 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#3.2;trace:T000/run1:56
};
