// struct: NeAllocatorVtbl
// subsystem: core
// size: ?
struct NeAllocatorVtbl {
    /* 0x018 */ void *alloc;            // L2 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#1 void *(*)(NeAllocator*, size_t size, size_t align)
    /* 0x038 */ void *free;             // L2 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#1 void (*)(NeAllocator*, void*)
};
