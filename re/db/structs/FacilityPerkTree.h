// struct: FacilityPerkTree
// subsystem: career
// size: 0x28 L2 decomp:re/decomp/140f7cfe0.c;imm:0x28 (allocated with mov edx,0x28 at 0x140f7e34e)
// notes: a facility's upgrade tree: allocator +0x00, top-level node vector begin/end/cap +0x08/+0x10/+0x18, allocator +0x20. Descendants hang off each FacilityNode's child vector (+0x20..+0x28), filled recursively by 0x142ea5ae0; 0x140b537a0 flattens the whole tree depth-first (via 0x142ea60b0) into a weak_ptr vector for the serialiser and the daily pass (ce-verify-7 V0327). Raised by ce-rnd (the lead authorised creating career-lane structs).
struct FacilityPerkTree {
    /* 0x08 */ void *nodes_begin;                   // L2 decomp:re/decomp/140b537a0.c;xref:0x140b537d0 top-level nodes as 16-byte shared_ptrs {FacilityNode *, control block} (the loader bumps the strong count at ctrl+8)
    /* 0x10 */ void *nodes_end;                     // L2 decomp:re/decomp/140b537a0.c;xref:0x140b537d4 end of the node list
};
