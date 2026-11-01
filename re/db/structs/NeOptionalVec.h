// struct: NeOptionalVec
// subsystem: core
// size: 0x28 L2 decomp:re/decomp/140ae8040.c;decomp:re/decomp/1413cf090.c (by-value optional vector argument; engaged flag at +0x20)
// notes: optional<vector<T>> passed by value to Career_Rebuild_SeasonStandings and 0x1413cf090; the callee copies it with alloc vt+0x18(count*sizeof T) and frees an engaged caller list with alloc vt+0x38. Raised by ce-season.
struct NeOptionalVec {
    /* 0x000 */ NeAllocator *alloc;                  // L2 decomp:re/decomp/140ae8040.c;decomp:re/decomp/1413cf090.c;xref:0x140ae80fc allocator of the buffer
    /* 0x008 */ void *begin;                         // L2 decomp:re/decomp/140ae8040.c;xref:0x140ae812f first element
    /* 0x010 */ void *end;                           // L2 decomp:re/decomp/140ae8040.c;xref:0x140ae8133 end of the elements
    /* 0x018 */ uint64_t count;                      // L2 decomp:re/decomp/140ae8040.c;xref:0x140ae8107 element count used for the copy allocation
    /* 0x020 */ uint8_t engaged;                     // L2 decomp:re/decomp/140ae8040.c;decomp:re/decomp/1413cf090.c;xref:0x140ae80f1,0x1413cf0da 0 = no list
};
