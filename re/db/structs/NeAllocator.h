// struct: NeAllocator
// subsystem: core
// size: ?
// notes: allocator object; only the vtable is known. vt+0x18 alloc(self, size, align), vt+0x38 free(self, p)
struct NeAllocator {
    /* 0x000 */ NeAllocatorVtbl *vtable; // L2 decomp:work/exe/decomp/141746710_tile_career.c;doc:research/exe.md#1 alloc(0xd8,0x10) and alloc(0x148,0x10) in every adder
};
